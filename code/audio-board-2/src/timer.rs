use vumeter_lib::audio_hw::TimerEvent;
use TimerEvent::*;

//#[link(name = "libteensy")]

extern "C" {
    pub fn pit_isr();
    fn timer_count() -> u64;
    fn timer_init(cycles : u32) -> bool;
}

pub struct Timer{
    count : u64
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
        let cnt = unsafe{timer_count()};
        let diff = cnt - self.count;
        self.count = cnt;
        if diff > 0 {
            Some(Elapsed(diff))
        } else { None }
    }
}
