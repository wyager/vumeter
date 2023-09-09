use bsp::hal::{self, gpio};
use teensy4_pins;
use cty;
use teensy4_pins::t40::P6;
use vumeter_lib::audio_hw::SwitchEvent;
use SwitchEvent::*;
//use Pin::*;
use teensy4_bsp as bsp;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::PinState;

#[link(name = "libteensy")]

pub struct SleepPin{}

#[no_mangle]
static mut SLP_TAKEN: bool = false;
impl SleepPin {
    pub fn initialize(gpio2: &mut hal::gpio::Port<2>, sleep_pin: teensy4_pins::common::P7) -> Option<SleepPin> {
        unsafe {
            if SLP_TAKEN  {return None};
            SLP_TAKEN = true;
            gpio2.output(sleep_pin);
        }
        Some(SleepPin{})
    }
    pub fn set(gpio2: &mut hal::gpio::Port<2>, sleep_pin: teensy4_pins::common::P7, on : bool) {
        if on == true {
            let _ = gpio2.output(sleep_pin).set_high();
        }
        else {
            let _ = gpio2.output(sleep_pin).set_low();
        }
    }
}
use bsp::hal::gpio::Port;
pub struct Pwr{}

#[no_mangle]
static mut PWR_TAKEN: bool = false;
impl Pwr {
    pub fn initialize(gpio2: &mut hal::gpio::Port<2>, power_pin: teensy4_pins::common::P6) -> Option<Pwr> {
        unsafe {
            if PWR_TAKEN  {return None};
            PWR_TAKEN = true;  
        }
        gpio2.output(power_pin);
        Some(Pwr{})
    }
    pub fn set(gpio2: &mut hal::gpio::Port<2>, power_pin: teensy4_pins::common::P6, on : bool) {
        if on == true {
            let _ = gpio2.output(power_pin).set_high();
        }
        else {
            let _ = gpio2.output(power_pin).set_low();
        }
    }
}




pub struct Switch{
    prev : Option<vumeter_lib::audio_hw::Pin> // "None" means the wires were in some weird state
}
 
#[no_mangle]
static mut SWITCH_TAKEN: bool = false;

impl Switch {
    pub fn initialize(gpio2: &mut hal::gpio::Port<2>, pin12: teensy4_pins::common::P12, pin11: teensy4_pins::common::P11, pin10: teensy4_pins::common::P10, pin9: teensy4_pins::common::P9) -> Option<Switch>{
        unsafe {
            if SWITCH_TAKEN {return None};
            SWITCH_TAKEN=true;    
        }
        gpio2.input(pin12);
        gpio2.input(pin11);
        gpio2.input(pin10);
        gpio2.input(pin9);
        Some(Switch{prev:None})
    }
    pub fn check(&mut self, gpio2: &mut hal::gpio::Port<2>, pin12: teensy4_pins::common::P12, pin11: teensy4_pins::common::P11, pin10: teensy4_pins::common::P10, pin9: teensy4_pins::common::P9) -> Option<SwitchEvent> {
        use SwitchEvent::*;
        use vumeter_lib::audio_hw::Pin::*;
        let mut state: u8 = 0;

        state |= (gpio2.input(pin9).is_set() as u8) << 0;
        state |= (gpio2.input(pin10).is_set() as u8) << 1;
        state |= (gpio2.input(pin11).is_set() as u8) << 2;
        state |= (gpio2.input(pin12).is_set() as u8) << 3;

       // let state = unsafe{switch_pins()};
        let state = match state {
            0b1 => Some(A),
            0b10 => Some(B),
            0b100 => Some(C),
            0b1000 => Some(D),
            _ => None
        };
        if self.prev != state {
            self.prev = state;
            match state {
                None => None,
                Some(pin) => Some(Activated(pin))
            }
        } else {
            None
        }
    }
    pub fn current(&self) -> Option<vumeter_lib::audio_hw::Pin> {
        self.prev
    }
}
