//! The starter code slowly blinks the LED, sets up
//! USB logging, and creates a UART driver using pins
//! 14 and 15. The UART baud rate is [`UART_BAUD`].
//!
//! Despite targeting the Teensy 4.0, this starter code
//! also works on the Teensy 4.1.

#![no_std]
#![no_main]

extern crate vumeter_lib as vu;

use teensy4_panic as _;

use cortex_m::asm;
use teensy4_bsp as bsp;
use bsp::board;
use bsp::hal::timer::Blocking;

use vu::bandpass as bp;
use vu::audio_hw::*;
use vu::shared::*;

use cortex_m::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};

use boardlib::{spdif,uart,switch,timer};
use cortex_m::asm::wfi;


use core::fmt::Write as _;

/// CHANGE ME to vary the baud rate.
const UART_BAUD: u32 = 115200;
/// Milliseconds to delay before toggling the LED
/// and writing text outputs.
const DELAY_MS: u32 = 500;

#[bsp::rt::entry]
fn main() -> ! {
    // These are peripheral instances. Let the board configure these for us.
    // This function can only be called once!
    let instances = board::instances();
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
        // This is a hardware timer. We'll use it for blocking delays.
        mut gpt1,
        // These are low-level USB resources. We'll pass these to a function
        // that sets up USB logging.
        usb,
        // This is the GPIO2 port. We need this to configure the LED as a
        // GPIO output.
        mut gpio2,
        // This resource is for the UART we're creating.
        lpuart2,
        ..
    } = board::t40(instances);

    // When this returns, you can use the `log` crate to write text
    // over USB. Use either `screen` (macOS, Linux) or PuTTY (Windows)
    // to visualize the messages from this example.
    bsp::LoggingFrontend::default_log().register_usb(usb);

    // This configures the LED as a GPIO output.
    let led = board::led(&mut gpio2, pins.p13);

    // Configures the GPT1 timer to run at GPT1_FREQUENCY. See the
    // constants below for more information.
    gpt1.disable();
    gpt1.set_divider(GPT1_DIVIDER);
    gpt1.set_clock_source(GPT1_CLOCK_SOURCE);

    // Convenience for blocking delays.
    let mut delay = Blocking::<_, GPT1_FREQUENCY>::from_gpt(gpt1);

    // Create the UART driver using pins 14 and 15.
    // Cast it to a embedded_hal trait object so we can
    // use it with the write! macro.
    let mut lpuart2: board::Lpuart2 = board::lpuart(lpuart2, pins.p14, pins.p15, UART_BAUD);
    let lpuart2: &mut dyn embedded_hal::serial::Write<u8, Error = _> = &mut lpuart2;

    let mut counter: u32 = 0;
    
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
        led.toggle();
        log::info!("Hello from the USB logger! The count is {counter}");
        write!(
            lpuart2,
            "Hello from the UART driver! The count is {counter}\r\n"
        )
        .ok();

        delay.block_ms(DELAY_MS);
        counter = counter.wrapping_add(1);
	
	
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
