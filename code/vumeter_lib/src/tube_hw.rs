use crate::protocol::{Brightness,Pkt,ParseState};
use crate::shared::{Consumer};


#[derive(PartialEq,Eq)]
enum PowerState {Energized,Deenergized,EnergizingFor{ticks:usize}}
use PowerState::*;
#[derive(PartialEq,Eq)]
pub enum HVTransition {NoChange, JustBecameEnergized}
use HVTransition::*;
pub struct HV<Pin> {
    state : PowerState,
    pin : Pin,
}

pub trait DigitalOutput {
    fn set(&mut self, _ : bool);
}

pub const ENERGIZE_TICKS : usize = 5; // How long to allow the secondary cathodes to fire before enabling primary cathodes (250ms)

impl<Pin> HV<Pin> where Pin : DigitalOutput {
    pub fn init(mut pin : Pin) -> Self {
        pin.set(false);
        HV{state:Deenergized,pin}
    }
    pub fn energize(&mut self) -> () {
        if self.state == Deenergized {
            self.pin.set(true); 
            self.state = EnergizingFor{ticks:0};
        }
    }
    pub fn deenergize(&mut self) -> () {
        if self.state != Deenergized {self.pin.set(false); self.state = Deenergized;}
    }
    pub fn tick(&mut self) -> HVTransition {
        match &self.state {
            EnergizingFor{ticks} => {
                if *ticks < ENERGIZE_TICKS {
                    self.state = EnergizingFor{ticks:ticks+1};
                    NoChange
                } else {
                    self.state = Energized;
                    JustBecameEnergized
                }
            },
            _ => NoChange
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub enum PWMPin {A,B,C,D}
use PWMPin::*;

pub trait PWMx4 {
    fn get_max_duty(&self) -> u32;
    fn set_duty(&mut self, _ : PWMPin, duty : u32);
}




pub struct PwmManager<PowerPin,PWM> {
    hv : HV<PowerPin>,
    duties : [u32;4],
    zero_ticks : usize,
    update_ticks : usize,
    pwm : PWM,
}

const ZERO_TICKS : usize = 10*20; // 10 seconds of silence before turning off HV
const DISCONNECT_TICKS : usize = 3 * 20; // 3 seconds of no input before turning off HV
impl<PowerPin,PWM> PwmManager<PowerPin,PWM> where PowerPin : DigitalOutput, PWM : PWMx4 {
    pub fn new(mut hv : HV<PowerPin>, mut pwm : PWM) -> Self {
        [A,B,C,D].iter().cloned().for_each(|tube| pwm.set_duty(tube, 0));
        hv.energize();
        PwmManager{hv,duties:[0;4],zero_ticks:0,update_ticks:0,pwm}
    }
    pub fn set_pwm(&mut self, tube : PWMPin, brightness : Brightness) {
        self.update_ticks = 0;
        let duty = brightness.map_to(0,self.pwm.get_max_duty());
        if duty > 0 {
            self.hv.energize();
            self.zero_ticks = 0;
        }
        self.duties[tube as usize] = duty;
        if self.hv.state == Energized {
            self.pwm.set_duty(tube,duty);
        }
    }
    pub fn tick(&mut self) {
        let zeroed = self.duties.iter().all(|d| *d == 0);
        if zeroed {self.zero_ticks += 1};
        self.update_ticks += 1;
        if self.zero_ticks >= ZERO_TICKS || self.update_ticks >= DISCONNECT_TICKS {
            self.hv.deenergize();
            [A,B,C,D].iter().cloned().for_each(|tube| self.pwm.set_duty(tube, 0));
        }
        match self.hv.tick() {
            NoChange => (),
            JustBecameEnergized => [A,B,C,D].iter().cloned().for_each(|tube| self.pwm.set_duty(tube, self.duties[tube as usize]))
        }
    }
}


pub enum Event {
    Rx1(u8),
    Rx2(u8),
    TimerFired
}
use Event::*;

pub struct State<PowerPin,PWM> {
    p1 : ParseState,
    p2 : ParseState,
    pwm : PwmManager<PowerPin,PWM>
}

impl<PowerPin,PWM> State<PowerPin,PWM> where PowerPin : DigitalOutput, PWM : PWMx4 {
    pub fn new(hv_pwr_en : PowerPin, pwm : PWM) -> State<PowerPin,PWM> {
        let hv = HV::init(hv_pwr_en);
        // Initialize protocol parsers
        let p1 = crate::protocol::ParseState::new();
        let p2 = crate::protocol::ParseState::new();
        State{p1,p2,pwm:PwmManager::new(hv,pwm)}
    }
    pub fn update<Out:Consumer<Pkt>>(&mut self, evt : Event, u1_out : &mut Out, u2_out : &mut Out){
        use crate::protocol as pkt;
        match evt {
            Event::Rx1(byte) => {
                self.p1.step(&[D,C,B,A], byte).map(|evt| match evt {
                    pkt::Event::Update{which,brightness} => self.pwm.set_pwm(which,brightness),
                    pkt::Event::Send{pkt} => u2_out.consume(pkt)
                });
            },
            Event::Rx2(byte) => {
                self.p2.step(&[A,B,C,D], byte).map(|evt| match evt {
                    pkt::Event::Update{which,brightness} => self.pwm.set_pwm(which,brightness),
                    pkt::Event::Send{pkt} => u1_out.consume(pkt)
                });
            },
            TimerFired => self.pwm.tick()
        }
    }
    pub fn display_startup_pattern<F:FnMut()->()>(&mut self, sleep : &mut F) {
        let init_pattern = (0..0x3FFF/0x10).chain((0..0x3FFF/0x10).rev()).map(|x| Brightness::new(x*0x10).unwrap());
        for brightness in init_pattern {
            [A,B,C,D].iter().cloned().for_each(|tube| {
                self.pwm.set_pwm(tube,brightness);
            });
            sleep();
        }
    }
}

