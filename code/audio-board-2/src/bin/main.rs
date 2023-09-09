#![no_std]
#![no_main]

extern crate vumeter_lib as vu;

use teensy4_panic as _;

use teensy4_bsp as bsp;
use bsp::{board, interrupt, rt::{interrupt, entry}};

use vu::bandpass as bp;
use vu::audio_hw::*;
use vu::shared::*;

use cortex_m::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};

use boardlib::{spdif,switch,timer};
use cortex_m::asm::{wfi, delay};

use core::fmt::Write;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

//const SPDIF_SIC_LOCKLOSS: u32 = 1 << 2;
//const SPDIF_SIC_LOCK: u32 = 1 << 20;

#[interrupt]
fn SPDIF() {
   //let spdif_local = unsafe{ bsp::ral::spdif::SPDIF::instance()};
   //spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCKLOSS);
   //spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCK);
   unsafe{spdif::spdif_isr()};
}

#[interrupt]
fn DMA3_DMA19() {
    unsafe{spdif::spdif_dma_isr()};
}

#[interrupt]
fn PIT() {
    unsafe{timer::pit_isr()};
}

#[entry]
fn main() -> ! {
    // These are peripheral instances. Let the board configure these for us.
    // This function can only be called once!
    let instances = board::instances();
    // Driver resources that are configured by the board. For more information,
    // see the `board` documentation.
    let board::Resources {
        // `pins` has objects that represent the physical pins. The object
        // for pin 13 is `p13`.
        pins,
        // This is a hardware timer. We'll use it for blocking delays.
        mut gpt1,
        // This is the GPIO2 port. We need this to configure the LED as a
        // GPIO output.
        mut gpio2,
        //use the bsp implementation to replace the c code from lib
        lpuart3,
        //use the bsp implementation to log info to OLED
        lpi2c1,
        ..
    } = board::t40(instances);

    let lpi2c: board::Lpi2c1 = board::lpi2c(
        lpi2c1,
        pins.p19,
        pins.p18,
        board::Lpi2cClockSpeed::KHz400,
    );
    let interface = I2CDisplayInterface::new(lpi2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
    .into_terminal_mode();

    let mut spdif = spdif::SPDIF::initialize().unwrap();
    let mut uart: board::Lpuart3 = board::lpuart(lpuart3, pins.p17, pins.p16, 460800);
    let mut switch = switch::Switch::initialize().unwrap();
    let mut pwr = switch::Pwr::initialize().unwrap();
    let mut sleep_pin = switch::SleepPin::initialize().unwrap();

    // This configures the LED as a GPIO output.
    let led = board::led(&mut gpio2, pins.p13);

    pwr.set(true);
    
    // Configures the GPT1 timer to run at GPT1_FREQUENCY. See the
    // constants below for more information.
    gpt1.disable();
    gpt1.set_divider(GPT1_DIVIDER);
    gpt1.set_clock_source(GPT1_CLOCK_SOURCE);

    // Convenience for blocking delays.
    //let mut delay = hal::timer::Blocking:: <_,GPT1_FREQUENCY> ::from_gpt(gpt1);

    delay(200);
    
    display.init().unwrap();

    let _ = display.clear().unwrap();

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
    let mut counter: u32 = 0;
    loop {
        // Flush major serial buffer to minor buffer until we run out of space in minor buffer
        while let Some(()) = tx_buf.with_front(|byte| uart.write(byte).ok()) {}
        // Check for hardware events
        let switch = switch.check().map(Event::Switch);
        let spdif = spdif.read().map(Event::Audio);
        let timer = timer.elapsed().map(Event::Timer);
        let uart = uart.read().ok().map(Event::Rx);

        let _ = display.set_column(1);
        let _ = display.write_fmt(format_args!("Counter: {}", counter));

        let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[10]) });
        let _ = display.set_column(1);
        let _ = display.write_fmt(format_args!("Counter2: {}", counter));

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
        let _ = display.write_str("                                                                               ");
        let _ = display.clear().unwrap();
        counter = counter.wrapping_add(1);
    }
}

// We're responsible for configuring our timers.
// This example uses PERCLK_CLK as the GPT1 clock source,
// and it configures a 1 KHz GPT1 frequency by computing a
// GPT1 divider.
use bsp::hal::gpt::ClockSource;
/// The intended GPT1 frequency (Hz).
const GPT1_FREQUENCY: u32 = 1_000;
/// Given this clock source...
const GPT1_CLOCK_SOURCE: ClockSource = ClockSource::HighFrequencyReferenceClock;
/// ... the root clock is PERCLK_CLK. To configure a GPT1 frequency,
/// we need a divider of...
const GPT1_DIVIDER: u32 = board::PERCLK_FREQUENCY / GPT1_FREQUENCY;
