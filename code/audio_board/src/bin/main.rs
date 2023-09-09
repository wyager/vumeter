#![no_std]
#![no_main]

extern crate panic_halt;
extern crate vumeter_lib as vu;

#[macro_use(block)]
extern crate nb;

use imxrt_rt::entry;
use cortex_m::asm;
use teensy4_bsp as bsp;
use bsp::board;

use imxrt_rt::interrupt;
use imxrt1062_pac::interrupt;

use vu::bandpass as bp;
use vu::audio_hw::*;
use vu::shared::*;

use embedded_hal::serial::{Read,Write};
use embedded_hal::digital::v2::OutputPin;

use boardlib::{spdif,uart,switch,timer};
use cortex_m::asm::wfi;

#[interrupt]
unsafe fn SPDIF() {
    unsafe{boardlib::spdif::spdif_isr()};
}

#[interrupt]
unsafe fn DMA3_DMA19() {
    unsafe{boardlib::spdif::spdif_dma_isr()};
}

#[interrupt]
unsafe fn LPUART3() {
    unsafe{uart::uart_isr()};
}

#[interrupt]
unsafe fn PIT() {
    unsafe{timer::pit_isr()};
}

#[entry]
fn main() -> ! {
    let resources = board::t40(board::instances());
    let mut led = switch::Led::initialize().unwrap();
    led.set(true);
    let mut spdif = spdif::SPDIF::initialize().unwrap();
    let mut uart = uart::UART::initialize((460800. *  600. / 528.) as u32).unwrap();
    let mut switch = switch::Switch::initialize().unwrap();
    let mut pwr = switch::Pwr::initialize().unwrap();
    let mut sleep_pin = switch::SleepPin::initialize().unwrap();
 
    // Enable the interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(imxrt_ral::Interrupt::SPDIF);
        cortex_m::peripheral::NVIC::unmask(imxrt_ral::Interrupt::DMA3_DMA19);
        cortex_m::peripheral::NVIC::unmask(imxrt_ral::Interrupt::LPUART3);
        cortex_m::peripheral::NVIC::unmask(imxrt_ral::Interrupt::PIT);
    }
    pwr.set(true);

    // Enabling logging increases current usage by about 10mA at 528MHz
    // peripherals.log.init(Default::default());
    asm::delay(200);
    
    // Set up some memory for signal processing
    const MAX_BANDS : usize = 32;
    let mut l = [bp::BandState::default(); MAX_BANDS];
    let mut r = [bp::BandState::default(); MAX_BANDS];
    let mut p = [0.; MAX_BANDS];
    let mut s = [0.; MAX_BANDS];
    let scratch = bp::Scratch::new(&mut l[..],&mut r[..],&mut p[..],&mut s[..]).unwrap();
    // let mut state = ConnectedState::new(AudioState::Disconnected{scratch});
    let mut state = DeviceState::new(scratch);

    // Buffer outgoing serial data
    let mut tx_buf : [u8;512] = [0;512];
    let mut tx_buf = RingBuf::new(&mut tx_buf);

    let mut timer = timer::Timer::initialize(25_000).unwrap(); // 25ms
    
    loop {
        // Flush major serial buffer to minor buffer until we run out of space in minor buffer
        while let Some(()) = tx_buf.with_front(|byte| uart.write(byte).ok()) {}
        // Check for hardware events
        let switch = switch.check().map(Event::Switch);
        let spdif = spdif.read().map(Event::Audio);
        let timer = timer.elapsed().map(Event::Timer);
        let uart = uart.read().ok().map(Event::Rx);
        
        if !timer.is_none() {

        }
        if switch.is_none() && spdif.is_none() && timer.is_none() && uart.is_none() {
            sleep_pin.set(true);
            wfi();
            sleep_pin.set(false);
        } else {
            for evt in switch.into_iter().chain(spdif.into_iter()).chain(timer.into_iter()).chain(uart.into_iter()) {
                state.step(evt,&mut tx_buf)
            }
        }
    }

}









