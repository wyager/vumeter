//use cty;
use vumeter_lib::audio_hw::TimerEvent;
use TimerEvent::*;
use teensy4_bsp as bsp;
use bsp::ral::{pit, ccm};
use cortex_m::peripheral::NVIC;

#[link(name = "libteensy")]

pub struct Timer{
    count : u64
}
static mut PIT_COUNT: u64 = 0;
fn timer_count() -> u64 {
    unsafe { PIT_COUNT }
}

pub fn pit_isr() {
    let pit_local = unsafe{ pit::PIT::instance()};
    let timer0_tctrl = pit_local.TIMER[0].TCTRL.read();
    if timer0_tctrl==1 {
        unsafe { PIT_COUNT.wrapping_add(1) };
        pit_local.TIMER[0].TFLG.write(1);
    }
}
const CCM_CCGR_ON: u32 = 3;
pub fn ccm_ccgr1_pit(n: u32) -> u32 {
    (n & 0x03) << 12
}
fn timer_init(cycles : u32) -> bool {
    let ccm_local = unsafe {ccm::CCM::instance()};
    let pit_local = unsafe{ pit::PIT::instance()};
    ccm_local.CCGR1.write(ccm_local.CCGR1.read() | ccm_ccgr1_pit(CCM_CCGR_ON));
    pit_local.MCR.write(1);

    let channel_index = 0;
    /*
    loop {
        let timer0_tctrl = pit_local.TIMER[channel_index].TCTRL.read();
        if (timer0_tctrl == 0) {
            break;
        } 
        if()
    }
*/
    pit_local.TIMER[channel_index].LDVAL.write(cycles);
    pit_local.TIMER[channel_index].TCTRL.write(3);
    unsafe { 
 //              NVIC::set_priority(&mut NVIC::PTR, bsp::Interrupt::PIT, 208);
               NVIC::unmask(bsp::Interrupt::PIT) };
    true
}


#[no_mangle]
static mut TIMER_TAKEN: bool = false;
impl Timer {
    pub fn initialize(micros : u32) -> Option<Timer> {
        let cycles : u32 = (24000000 / 1000000) * micros - 1;
        let ok = unsafe {
            if TIMER_TAKEN  {return None};
            TIMER_TAKEN = true;
            timer_init(cycles) 
        };
        if ok {Some(Timer{count:0})} else {None}
    }
    pub fn elapsed(&mut self) -> Option<TimerEvent> {
        let cnt = timer_count();
        let diff = cnt - self.count;
        self.count = cnt;
        if diff > 0 {
            Some(Elapsed(diff))
        } else { None }
    }
}
