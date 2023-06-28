use cty;
use vumeter_lib::audio_hw::{SwitchEvent,Pin};
use SwitchEvent::*;
use Pin::*;

#[link(name = "libteensy")]

extern "C" {
    // fn copy_from_spdif_buffer(max : cty::uint32_t, l : *mut f32, r : *mut f32) -> cty::uint32_t;
    fn switch_pins() -> u8;
    fn switch_init();
    pub fn set_pwr_en(_ : bool);
    pub fn pwr_init();
    pub fn set_led_en(_ : bool);
    pub fn led_init();
    fn set_sleep_pin(_ : bool);
    fn sleep_pin_init();
}

pub struct SleepPin{}

#[no_mangle]
static mut SLP_TAKEN: bool = false;
impl SleepPin {
    pub fn initialize() -> Option<SleepPin> {
        unsafe {
            if SLP_TAKEN  {return None};
            SLP_TAKEN = true;
            sleep_pin_init();   
        }
        Some(SleepPin{})
    }
    pub fn set(&mut self, on : bool) {
        unsafe{set_sleep_pin(on)};
    }
}

pub struct Pwr{}

#[no_mangle]
static mut PWR_TAKEN: bool = false;
impl Pwr {
    pub fn initialize() -> Option<Pwr> {
        unsafe {
            if PWR_TAKEN  {return None};
            PWR_TAKEN = true;
            pwr_init();   
        }
        Some(Pwr{})
    }
    pub fn set(&mut self, on : bool) {
        unsafe{set_pwr_en(on)};
    }
}

pub struct Led{}

#[no_mangle]
static mut LED_TAKEN: bool = false;
impl Led {
    pub fn initialize() -> Option<Led> {
        unsafe {
            if LED_TAKEN  {return None};
            LED_TAKEN = true;
            led_init();   
        }
        Some(Led{})
    }
    pub fn set(&mut self, on : bool) {
        unsafe{set_led_en(on)};
    }
}



pub struct Switch{
    prev : Option<Pin> // "None" means the wires were in some weird state
}
 


#[no_mangle]
static mut SWITCH_TAKEN: bool = false;



impl Switch {
    pub fn initialize() -> Option<Switch> {
        unsafe {
            if SWITCH_TAKEN {return None};
            SWITCH_TAKEN=true;
            switch_init();   
        }
        Some(Switch{prev:None})
    }
    pub fn check(&mut self) -> Option<SwitchEvent> {
        use SwitchEvent::*;
        use Pin::*;
        let state = unsafe{switch_pins()};
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
    pub fn current(&self) -> Option<Pin> {
        self.prev
    }
}
