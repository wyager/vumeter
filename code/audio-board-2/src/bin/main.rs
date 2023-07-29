#![no_std]
#![no_main]

extern crate vumeter_lib as vu;

use teensy4_panic as _;

use cortex_m::asm;
use teensy4_bsp as bsp;
use bsp::board;

use vu::bandpass as bp;
use vu::audio_hw::*;
use vu::shared::*;

use teensy4_bsp::interrupt;
use cortex_m::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};

use boardlib::{spdif,uart,switch,timer};
use cortex_m::asm::wfi;

#[bsp::rt::interrupt]
fn SPDIF() {
    unsafe{spdif::spdif_isr()};
}

#[bsp::rt::interrupt]
fn DMA3_DMA19() {
    unsafe{spdif::spdif_dma_isr()};
}

#[bsp::rt::interrupt]
fn LPUART3() {
    unsafe{uart::uart_isr()};
}

#[bsp::rt::interrupt]
fn PIT() {
    unsafe{timer::pit_isr()};
}

#[bsp::rt::entry]
fn main() -> ! {
    // These are peripheral instances. Let the board configure these for us.
    // This function can only be called once!
    //let resources = board::t40(board::instances());
    let mut led = switch::Led::initialize().unwrap();
    led.set(true);
    let mut spdif = spdif::SPDIF::initialize().unwrap();
    let mut uart = uart::UART::initialize((460800. *  600. / 528.) as u32).unwrap();
    let mut switch = switch::Switch::initialize().unwrap();
    let mut pwr = switch::Pwr::initialize().unwrap();
    let mut sleep_pin = switch::SleepPin::initialize().unwrap();

    // Driver resources that are configured by the board. For more information,
    // see the `board` documentation.
    let board::Resources {
        // `pins` has objects that represent the physical pins. The object
        // for pin 13 is `p13`.
        pins,
        // This is the GPIO2 port. We need this to configure the LED as a
        // GPIO output.
        mut gpio2,
        ..
    } = board::t40(board::instances());

    // This configures the LED as a GPIO output.
    let led = board::led(&mut gpio2, pins.p13);

    pwr.set(true);

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
            led.toggle();
            sleep_pin.set(false);
        } else {
            for evt in switch.into_iter().chain(spdif.into_iter()).chain(timer.into_iter()).chain(uart.into_iter()) {
                state.step(evt,&mut tx_buf)
            }
        }
    }
}
