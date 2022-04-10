
#![no_std]
#![no_main]

extern crate cortex_m_rt;
extern crate panic_halt;
extern crate vumeter_lib as vu;
extern crate atsamd11c;

#[macro_use(block)]
extern crate nb;



use atsamd_hal::clock::GenericClockController;
use samd11_bare::pac::{Peripherals,CorePeripherals};
use atsamd_hal::prelude::*;
use samd11_bare::pac::gclk::clkctrl::GEN_A;
use samd11_bare::pac::gclk::genctrl::SRC_A;
use atsamd_hal::sercom::{Sercom0, Sercom1};//{PadPin, Sercom0Pad0, Sercom0Pad1, Sercom1Pad2, Sercom1Pad3, UART0, UART1};
use embedded_hal::serial::{Read,Write};
use atsamd_hal::timer::TimerCounter;
use cortex_m::asm::wfi;
use atsamd_hal::gpio::{Output,PushPull};//Pa2,Pa4,Pa5,Pa8,Pa9,PfF,PfE};
use atsamd_hal::pwm::{Channel,Pwm0};
use atsamd_hal::time::{Hertz,KiloHertz};
use atsamd_hal::delay::Delay;
use atsamd_hal::gpio::{AlternateE, AlternateF, AlternateC};
use atsamd_hal::gpio::pin::{PA02, PA04, PA05, PA08, PA09, PA14, PA15, PA24, PA25};
use atsamd_hal::gpio::Pin;
use atsamd_hal::sercom::uart as uart;
use atsamd_hal::sercom::uart::BaudMode;
use atsamd_hal::sercom::uart::Oversampling;
use atsamd_hal::gpio::PushPullOutput;

use vu::tube_hw as hw;

use vu::protocol::{ParseState,Brightness};
use vu::shared::{RingBuf};
use hw::PWMPin::{A,B,C,D};

use cortex_m_rt::entry;

use biquad::frequency::ToHertz;



struct PowerPin(Pin<PA02, PushPullOutput>);

impl hw::DigitalOutput for PowerPin {
    fn set(&mut self, on : bool) {
        if on {self.0.set_high().unwrap()} else {self.0.set_low().unwrap()}
    }
}

struct PWMPins {
    pwm:Pwm0, _p1:Pin<PA04,AlternateF>, _p2:Pin<PA05,AlternateF>, _p3:Pin<PA08,AlternateE>, _p4:Pin<PA09,AlternateE>
}
//
//impl hw::PWMx4 for PWMPins {
//    fn get_max_duty(&self) -> u32 { self.pwm.get_max_duty() }
//    fn set_duty(&mut self, pin : hw::PWMPin, duty : u32) {
//        match pin {
//            A => {self.pwm.set_duty(Channel::_1,duty)},
//            B => {self.pwm.set_duty(Channel::_0,duty)},
//            C => {self.pwm.set_duty(Channel::_2,duty)},
//            D => {self.pwm.set_duty(Channel::_3,duty)}
//        }
//    }
//}

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
    timer.start(Hertz(20));

    // Initialize TCC0 PWM subsystem
    let tcc0clk = clocks.tcc0(&gclk0).unwrap();
    let pwm = Pwm0::new(&tcc0clk, KiloHertz(10), peripherals.TCC0, &mut peripherals.PM);

    let mut pins = samd11_bare::Pins::new(peripherals.PORT);
    //// Enable PWM pins
    let p1 : Pin<PA04, AlternateF> = pins.d14.into_mode(); // parts.pa4.into_function_f(&mut parts.port);
    let p2 : Pin<PA05, AlternateF> = pins.d1.into_mode(); //parts.pa5.into_function_f(&mut parts.port);
    let p3 : Pin<PA08, AlternateE> = pins.d2.into_mode(); //parts.pa8.into_function_e(&mut parts.port);
    let p4 : Pin<PA09, AlternateE> = pins.d3.into_mode(); //parts.pa9.into_function_e(&mut parts.port);

    let pwm = PWMPins{pwm,_p1:p1,_p2:p2,_p3:p3,_p4:p4};

    // Initialize both left and right UARTs
    let gclk2 = clocks.get_gclk(GEN_A::GCLK2).expect("Could not get clock 2");
    let mut u1 = { 
        //let tx1: Sercom1Pad2<_> = parts.pa24.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let tx1 : Pin<PA24, AlternateC> = pins.d9.into();
        //let rx1: Sercom1Pad3<_> = parts.pa25.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        let rx1 : Pin<PA25, AlternateC> = pins.d10.into();
        let uart_clk = clocks.sercom1_core(&gclk2).expect("Could not configure sercom1 clock");
        let pads = uart::Pads::<Sercom1>::default().rx(rx1).tx(tx1);
        //UART1::new(&uart_clk,Hertz(460800),peripherals.SERCOM1,&mut peripherals.PM,(rx1, tx1))
        let config = uart::Config::new(&peripherals.PM, peripherals.SERCOM1, pads, uart_clk.freq());
        let config = config.baud(Hertz(460800), BaudMode::Fractional(Oversampling::Bits16));
        config.enable()
    };

    let mut u2 = { 
        // let tx2: Sercom0Pad0<_> = parts.pa14.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        // let rx2: Sercom0Pad1<_> = parts.pa15.into_pull_down_input(&mut parts.port).into_pad(&mut parts.port);
        // let uart_clk = clocks.sercom0_core(&gclk2).expect("Could not configure sercom0 clock");
        // UART0::new(&uart_clk,460800u32.hz(),peripherals.SERCOM0,&mut peripherals.PM,(rx2, tx2))
        let tx2 : Pin<PA14, AlternateC> = pins.d4.into();
        let rx2 : Pin<PA15, AlternateC> = pins.d5.into();
        let uart_clk = clocks.sercom0_core(&gclk2).expect("Could not configure sercom1 clock");
        let pads = uart::Pads::<Sercom0>::default().rx(rx2).tx(tx2);
        let config = uart::Config::new(&peripherals.PM, peripherals.SERCOM0, pads, uart_clk.freq());
        let config = config.baud(Hertz(460800), BaudMode::Fractional(Oversampling::Bits16));
        config.enable()
    };



    //// Initialize HV controller and activate HV boost circuit
    let hv_pwr_en = PowerPin(pins.d13.into_push_pull_output());
    //
    //let mut state = hw::State::new(hv_pwr_en,pwm);


    //use hw::Event::*;

    //

    //// Forwarding buffers. If there's a message intended for another board,
    //// we dump its contents in here and send it as the opportunity arises.
    //let mut u1_buf = [0;64];
    //let mut u1_buf = RingBuf::new(&mut u1_buf);
    //let mut u2_buf = [0;64];
    //let mut u2_buf = RingBuf::new(&mut u2_buf);

    //
    //// Give secondary cathodes time to fire
    //for _ in 0..=hw::ENERGIZE_TICKS {
    //    block!(timer.wait()).unwrap();
    //    state.update(TimerFired, &mut u1_buf, &mut u2_buf);
    //};

    //// Display startup animation
    //state.display_startup_pattern(&mut || delay.delay_ms(1u8));
    //
    //
    //loop {
    //    // If there's anything in the forward buffers, dump as much as we can to the UART.
    //    while let Some(()) = u1_buf.with_front(|byte| u1.write(byte).ok()) {}
    //    while let Some(()) = u2_buf.with_front(|byte| u2.write(byte).ok()) {}

    //    // Read a byte from either serial port
    //    let rx1 = u1.read().ok().map(Rx1);
    //    let rx2 = u2.read().ok().map(Rx2);
    //    // See if the timer fired
    //    let tmr = timer.wait().ok().map(|()| TimerFired);
    //    
    //    if rx1.is_none() && rx2.is_none() && tmr.is_none() {
    //        // wfi() // Doesn't seme to work properly on this board.
    //    } else {
    //        for evt in rx1.into_iter().chain(rx2.into_iter()).chain(tmr.into_iter()) {
    //            state.update(evt, &mut u1_buf, &mut u2_buf);
    //        }
    //    }
    //}

    loop {}
}




