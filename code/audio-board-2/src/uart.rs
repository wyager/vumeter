use cty;
use nb;

use embedded_hal::serial::{Read,Write};
use nb::Result;
use nb::Error::WouldBlock;

//#[link(name = "libteensy")]

extern "C" {
    // fn copy_from_spdif_buffer(max : cty::uint32_t, l : *mut f32, r : *mut f32) -> cty::uint32_t;
    fn uart4_init(baud : cty::uint32_t);
    pub fn uart_isr();
    fn uart4_read() -> i16;
    fn uart4_is_transmitting() -> bool;
    fn uart4_write(c : u8) -> cty::size_t;
}


pub struct UART{}

#[no_mangle]
static mut UART_TAKEN: bool = false;

impl UART {
    pub fn initialize(baud : u32) -> Option<UART> {
        unsafe {
            if UART_TAKEN {return None};
            UART_TAKEN=true;
            uart4_init(baud);   
        }
        Some(UART{})
    }

}

impl Read<u8> for UART {
    type Error = ();
    fn read(&mut self) -> Result<u8,()> {
        let r = unsafe{uart4_read()};
        if r < 0 {
            Err(WouldBlock)
        } else {
            Ok(r as u8)
        }
    }
}

impl Write<u8> for UART {
    type Error = ();
    fn write(&mut self, word:u8) -> Result<(),()> {
        let r = unsafe{uart4_write(word)};
        if r == 0 {
            Err(WouldBlock)
        } else {
            Ok(())
        }
    }
    fn flush(&mut self) -> Result<(),()> {
        if unsafe{uart4_is_transmitting()} {
            Err(WouldBlock)
        } else {
            Ok(())
        }
    }  
}
