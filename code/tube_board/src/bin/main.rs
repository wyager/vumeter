
#![no_std]
#![no_main]

extern crate cortex_m_rt;
extern crate panic_halt;
extern crate samd11_bare as hal;
extern crate vumeter_lib as vu;
extern crate atsamd11c14a;

#[macro_use(block)]
extern crate nb;

use hal::delay::Delay;

use hal::clock::GenericClockController;
use hal::pac::{Peripherals,CorePeripherals};
use hal::prelude::*;
use hal::pac::gclk::clkctrl::GEN_A;
use hal::pac::gclk::genctrl::SRC_A;
use hal::sercom::{PadPin, Sercom0Pad0, Sercom0Pad1, Sercom1Pad2, Sercom1Pad3, UART0, UART1};
use embedded_hal::serial::{Read,Write};
use hal::timer::TimerCounter;
use cortex_m::asm::wfi;
use atsamd_hal::gpio::{Output,PushPull,Pa2,Pa4,Pa5,Pa8,Pa9,PfF,PfE};
use atsamd_hal::pwm::{Channel,Pwm0};

use vu::tube_hw as hw;

use vu::protocol::{ParseState,Brightness};
use vu::shared::{RingBuf};
use hw::PWMPin::{A,B,C,D};

use cortex_m_rt::entry;

struct PowerPin(Pa2<Output<PushPull>>);

impl hw::DigitalOutput for PowerPin {
    fn set(&mut self, on : bool) {
        if on {self.0.set_high().unwrap()} else {self.0.set_low().unwrap()}
    }
}

struct PWMPins {
    pwm:Pwm0, _p1:Pa4<PfF>, _p2:Pa5<PfF>, _p3:Pa8<PfE>, _p4:Pa9<PfE>
}

impl hw::PWMx4 for PWMPins {
    fn get_max_duty(&self) -> u32 { self.pwm.get_max_duty() }
    fn set_duty(&mut self, pin : hw::PWMPin, duty : u32) {
        match pin {
            A => {self.pwm.set_duty(Channel::_1,duty)},
            B => {self.pwm.set_duty(Channel::_0,duty)},
            C => {self.pwm.set_duty(Channel::_2,duty)},
            D => {self.pwm.set_duty(Channel::_3,duty)}
        }
    }
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    // Initialize clocks
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    clocks.configure_gclk_divider_and_source(GEN_A::GCLK2, 1, SRC_A::DFLL48M, false);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    // Initialize 20Hz timer
    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tc1_tc2(&gclk0).unwrap();
    let mut timer = TimerCounter::tc1_(&timer_clock, peripherals.TC1, &mut peripherals.PM);
    timer.start(20u32.hz());

    // Initialize TCC0 PWM subsystem
    let tcc0clk = clocks.tcc0(&gclk0).unwrap();
    let pwm = Pwm0::new(&tcc0clk, 10.khz(), peripherals.TCC0, &mut peripherals.PM);

    let mut parts = peripherals.PORT.split();
    // Enable PWM pins
    let p1 = parts.pa4.into_function_f(&mut parts.port);
    let p2 = parts.pa5.into_function_f(&mut parts.port);
    let p3 = parts.pa8.into_function_e(&mut parts.port);
    let p4 = parts.pa9.into_function_e(&mut parts.port);

    let pwm = PWMPins{pwm,_p1:p1,_p2:p2,_p3:p3,_p4:p4};

    // Initialize both left and right UARTs
    let gclk2 = clocks.get_gclk(GEN_A::GCLK2).expect("Could not get clock 2");
    let mut u1 = { 
        let tx1: Sercom1Pad2<_> = parts.pa24.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let rx1: Sercom1Pad3<_> = parts.pa25.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let uart_clk = clocks.sercom1_core(&gclk2).expect("Could not configure sercom1 clock");
        UART1::new(&uart_clk,460800.hz(),peripherals.SERCOM1,&mut peripherals.PM,(rx1, tx1))
    };

    let mut u2 = { 
        let tx2: Sercom0Pad0<_> = parts.pa14.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let rx2: Sercom0Pad1<_> = parts.pa15.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let uart_clk = clocks.sercom0_core(&gclk2).expect("Could not configure sercom0 clock");
        UART0::new(&uart_clk,460800.hz(),peripherals.SERCOM0,&mut peripherals.PM,(rx2, tx2))
    };

    

    // Initialize HV controller and activate HV boost circuit
    let hv_pwr_en = PowerPin(parts.pa2.into_push_pull_output(&mut parts.port));
    
    let mut state = hw::State::new(hv_pwr_en,pwm);


    use hw::Event::*;

    

    // Forwarding buffers. If there's a message intended for another board,
    // we dump its contents in here and send it as the opportunity arises.
    let mut u1_buf = [0;64];
    let mut u1_buf = RingBuf::new(&mut u1_buf);
    let mut u2_buf = [0;64];
    let mut u2_buf = RingBuf::new(&mut u2_buf);

    
    // Give secondary cathodes time to fire
    for _ in 0..=hw::ENERGIZE_TICKS {
        block!(timer.wait()).unwrap();
        state.update(TimerFired, &mut u1_buf, &mut u2_buf);
    };

    // Display startup animation
    state.display_startup_pattern(&mut || delay.delay_ms(1u8));
    
    
    loop {
        // If there's anything in the forward buffers, dump as much as we can to the UART.
        while let Some(()) = u1_buf.with_front(|byte| u1.write(byte).ok()) {}
        while let Some(()) = u2_buf.with_front(|byte| u2.write(byte).ok()) {}

        // Read a byte from either serial port
        let rx1 = u1.read().ok().map(Rx1);
        let rx2 = u2.read().ok().map(Rx2);
        // See if the timer fired
        let tmr = timer.wait().ok().map(|()| TimerFired);
        
        if rx1.is_none() && rx2.is_none() && tmr.is_none() {
            // wfi() // Doesn't seme to work properly on this board.
        } else {
            for evt in rx1.into_iter().chain(rx2.into_iter()).chain(tmr.into_iter()) {
                state.update(evt, &mut u1_buf, &mut u2_buf);
            }
        }
    }

}




