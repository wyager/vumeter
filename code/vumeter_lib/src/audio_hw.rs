use crate::shared::Transition::{TransitioningTo,In};
use crate::shared::{Consumer,BlackHole,Transition};
use crate::protocol::Pkt;
use crate::protocol as pkt;
use biquad::frequency::{Hertz,ToHertz};
use core::time::Duration;


pub enum AudioEvent<T> {
    LockLost,
    LockAcquired(f32),
    Samples(T)
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Pin {A,B,C,D}

#[derive(Clone,Copy)]
pub enum SwitchEvent {
    Activated(Pin)
}

#[derive(Clone,Copy)]
pub enum TimerEvent {
    Elapsed(u64)
}

pub enum Event<Samps> {
    Audio(AudioEvent<Samps>),
    Switch(SwitchEvent),
    Timer(TimerEvent),
    Rx(u8)
}

use Event::*;


pub enum AudioState<'a> {
    Disconnected{scratch : crate::bandpass::Scratch<'a>},
    Running{vu : crate::bandpass::VU<'a>}
}

impl AudioState<'_> {
    pub fn num_bands(&self) -> usize {
        match self {
            Disconnected{scratch} => scratch.length(),
            Running{vu} => vu.num_bands()
        }
    }
}

impl AudioState<'_> {
    fn disconnect(&mut self) -> () {
        take(self, |audio| match audio {
            Disconnected{scratch} => Disconnected{scratch},
            Running{vu} => Disconnected{scratch:vu.teardown()}
        });
    }
    fn initialize(&mut self, sr : Hertz<f32>, params : crate::bandpass::Params) {
        take(self, |audio| match audio {
            Disconnected{scratch} => {
                let vu = crate::bandpass::VU::init(scratch, params, sr).unwrap();
                Running{vu}
            }
            Running{vu} => {
                let scratch = vu.teardown();
                let vu = crate::bandpass::VU::init(scratch, params, sr).unwrap();
                Running{vu} 
            }
        });
    }
    fn change_params(&mut self, params : crate::bandpass::Params) {
        take(self, |audio| match audio {
            Disconnected{scratch} => Disconnected{scratch},
            Running{vu} => {
                let sr = vu.sr;
                let scratch = vu.teardown();
                let vu = crate::bandpass::VU::init(scratch, params, sr).unwrap();
                Running{vu} 
            }
        });
    }
    fn sample<'a>(&'a mut self, l : f32, r : f32) -> Option<impl Iterator<Item=(usize,f32)> + 'a> {
         match self {
            Disconnected{..} => None, // Received sample while disconnected. That'd be a bug
            Running{vu} => vu.step(l,r).map(|output| output.enumerate().rev()) // Reverse so that all tubes update in the same ~2ms
        }
    }
}

use AudioState::*;

// We have to use this unpleasantness thanks to the inability
// to temporarily take a mut ref in Rust. Makes it hard to write
// state machines otherwise. Copied from take_mut
pub fn take<T, F>(mut_ref: &mut T, f: F)
  where F: FnOnce(T) -> T {
    use core::ptr;

    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = f(old_t);
        ptr::write(mut_ref, new_t);
    }
}



#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum TubeMode {
    Narrow, Wide
}
use TubeMode::*;

#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum DeviceMode {
    TubesOff,
    Calibrating,
    TubesOn(TubeMode)
}
use DeviceMode::*;



pub struct ConnectedState<'a> {
    audio_state : AudioState<'a>,
    mode : Transition<DeviceMode>
}



impl ConnectedState<'_> {
    pub fn step<Output : Consumer<Pkt>,Samps : Iterator<Item=(f32,f32)>>
        (&mut self, 
            evt : Event<Samps>, 
            output : &mut Output) {
        
        match evt {
            Timer(TimerEvent::Elapsed(i)) => {
                for _ in 0..i {
                    let mode = self.mode.step();
                    if let TransitioningTo{..} = self.mode {
                        if let In(mode) = mode {
                            log::info!("Transitioned to mode: {:?}",mode);
                        }
                    }
                    self.mode = mode;
                }
                if let TransitioningTo{..} = self.mode {self.const_tubes(0.0,output)}
                if let In(Calibrating) = self.mode {self.const_tubes(1.0, output)}
            },
            Switch(SwitchEvent::Activated(pin)) => {
                use Pin::*;
                let next_mode = match pin {
                    A => TubesOff,
                    B => {
                        log::info!("Updating params to Narrow mode");
                        self.audio_state.change_params(ConnectedState::narrow_params(self.audio_state.num_bands()));
                        TubesOn(Narrow)
                    },
                    C => {
                        log::info!("Updating params to normal mode");
                        self.audio_state.change_params(ConnectedState::default_params(self.audio_state.num_bands()));
                        TubesOn(Wide)
                    },
                    D => Calibrating,
                };
                log::info!("Setting mode to {:?}",next_mode);
                self.mode = TransitioningTo{state:next_mode,ticks:2};
            },
            Audio(spdif_evt) => {
                if let In(TubesOn(_)) = self.mode {
                    self.step_spdif(spdif_evt, output)
                } else {
                    self.step_spdif(spdif_evt, &mut BlackHole)
                }      
            },
            Rx(_) => log::info!("Received UART byte during operation")
        }
        
          
    }
    fn const_tubes<Output : Consumer<Pkt>>(&mut self, e : f32, output : &mut Output) {
        let num_bands = self.audio_state.num_bands();
        // let tx_buf = &mut self.tx_buf;
        // if tx_buf.space() > num_bands * Pkt::PKT_LEN {
        let e = (crate::protocol::Brightness::MAX_BRIGHTNESS as f32 * e) as u16;
        for i in (0..num_bands).rev() {
            let e = crate::protocol::Brightness::new(e).unwrap();
            let pkt = Pkt::mk_pkt(i as i8, e).unwrap();
            output.consume(pkt);      
        }
        // }
    }
    pub fn new<'a>(audio_state : AudioState<'a>) -> ConnectedState<'a>{
        ConnectedState{
            audio_state,
            mode : In(TubesOff)
        }
    }
    pub fn default_params(num_bands : usize) -> crate::bandpass::Params {
        let lo = 30.hz();
        let hi = 8000.hz();
        let update_rate : Hertz<f32> = crate::protocol::refresh_rate(num_bands).hz();
        crate::bandpass::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, percentile_ndb:0.6, lowest_ndb:0.00001, ewma_time:Duration::from_millis(50), filter_q:1.5, ..crate::bandpass::Params::default()}
    }
    pub fn narrow_params(num_bands : usize) -> crate::bandpass::Params {
        crate::bandpass::Params{filter_q:2., band_edge_multiplier:0.75, ..ConnectedState::default_params(num_bands)}
    }
    pub fn step_spdif<Output,Samps>(&mut self, event : AudioEvent<Samps>, output : &mut Output) 
        where Output : Consumer<Pkt>, Samps : Iterator<Item=(f32,f32)>  {
        use AudioEvent::*;
        match event {
            LockLost => {
                log::info!("S/PDIF lock Lost");
                self.audio_state.disconnect();
            },
            LockAcquired(sr) => {
                let actual = sr * 600. / 528.;
                log::info!("S/PDIF lock Acquired. Reported {}. Adjusting to {}", sr, actual);
                let params = match self.mode {
                    In(TubesOn(Narrow)) | TransitioningTo{state:TubesOn(Narrow),..} => {
                        log::info!("Initializing with Narrow params.");
                        ConnectedState::narrow_params(self.audio_state.num_bands())
                    },
                    _ => {
                        log::info!("Initializing with Wide params.");
                        ConnectedState::default_params(self.audio_state.num_bands())
                    }
                };
                self.audio_state.initialize(actual.hz(),params);
            },
            Samples(samps) => {
                for (l,r) in samps {
                    if let Some(outputs) = self.audio_state.sample(l,r) {
                        outputs.for_each(|(i,e)| {
                            let e : u16 = (crate::protocol::Brightness::MAX_BRIGHTNESS as f32 * e) as u16;
                            let e = crate::protocol::Brightness::new(e).unwrap();
                            let pkt = Pkt::mk_pkt(i as i8, e).unwrap();
                            output.consume(pkt);
                        });
                    }
                };
            }
        }
    }
}

#[derive(Clone,Copy)]
pub enum InitializerStage {
    Waiting{ticks:usize},
    Parsing{state: crate::protocol::ParseState, ticks:usize}
}
use InitializerStage::*;

pub struct InitializerState<'a> {
    scratch : crate::bandpass::Scratch<'a>,
    stage : InitializerStage,
    audio_connection : Option<Hertz<f32>>,
    switch : Option<Pin>
}

pub struct InitResult<'a> {
    trimmed_scratch : crate::bandpass::Scratch<'a>,
    audio_connection : Option<Hertz<f32>>,
    switch : Option<Pin>
}

impl<'a> InitializerState<'a> {
    fn step<Output:Consumer<Pkt>, Samps>(mut self : InitializerState<'a>, evt : Event<Samps>, output : &mut Output) -> Result<InitializerState<'a>, InitResult<'a>> {
        let reset = Waiting{ticks:4};
        match evt {
            Timer(TimerEvent::Elapsed(n)) => {
                for _ in 0..n {
                    match self.stage {
                        Waiting{ticks:0} => {
                            log::info!("Sending probe packet.");
                            output.consume(pkt::Pkt::probe_pkt());
                            self.stage = Parsing{state:pkt::ParseState::new(), ticks:4}
                        },
                        Waiting{ticks:n} => {
                            self.stage = Waiting{ticks:n-1}
                        },
                        Parsing{state:_,ticks:0} => {
                            log::info!("Parsing timed out.");
                            self.stage = reset
                        },
                        Parsing{state,ticks:n} => {
                            self.stage = Parsing{state,ticks:n-1}
                        }
                    }
                }
            }
            Rx(byte) => {
                log::info!("Recieved UART byte: {}", byte);
                match &mut self.stage {
                    Waiting{ticks:_} => {
                        log::info!("Received byte while waiting to send.");
                    },
                    Parsing{state,ticks:_} => {
                        if let Some(pkt) = state.step_(byte) {
                            log::info!("Packet parsed: {:?}",pkt);
                            match pkt {
                                Pkt::Set{ttl,brightness:pkt::Brightness::ZERO} => {
                                    let diff = (Pkt::PROBE_TTL - ttl) as usize;
                                    let num_tubes = diff / 2;
                                    if (diff & 1) == 1 {
                                        log::info!("Got odd tube length response.");
                                        self.stage = reset;
                                    } else if num_tubes > self.scratch.length() {
                                        log::info!("Counted too many tubes. Using max number.");
                                        return Err(InitResult{trimmed_scratch:self.scratch, audio_connection:self.audio_connection, switch:self.switch});
                                    } else {
                                        log::info!("Tubes detected: {}", num_tubes);
                                        return Err(InitResult{trimmed_scratch:self.scratch.trim(num_tubes).unwrap(), audio_connection:self.audio_connection, switch:self.switch});
                                    }
                                },
                                _ => {
                                    log::info!("Invalid probe response");
                                    self.stage = reset;
                                }
                            }
                        } 
                    }
                };
            },
            Audio(evt) => match evt {
                AudioEvent::LockLost => {self.audio_connection = None},
                AudioEvent::LockAcquired(sr) => {self.audio_connection = Some(sr.hz())},
                AudioEvent::Samples(_) => ()
            },
            Switch(SwitchEvent::Activated(pin)) => {
                self.switch = Some(pin)
            }
        };
        Ok(self)
    }
}

pub enum DeviceState<'a> {
    Connected(ConnectedState<'a>), // Tubes acquired
    Initializing(InitializerState<'a>)
}
use DeviceState::*;

impl DeviceState<'_> {
    pub fn new<'a>(scratch : crate::bandpass::Scratch<'a>) -> DeviceState<'a> {
        Initializing(InitializerState{stage:Waiting{ticks:4}, scratch, audio_connection:None, switch:None})
    }
    pub fn step<Output : Consumer<Pkt>,Samps : Iterator<Item=(f32,f32)>>
        (&mut self, evt : Event<Samps>, output : &mut Output) {
        take(self, |s| match s {
            Connected(mut state) => {
                state.step(evt,output);
                Connected(state)
            },
            Initializing(state) => {
                match state.step(evt, output) {
                    Ok(state) => Initializing(state),
                    Err(init_res) => {
                        let mut conn_state = ConnectedState::new(AudioState::Disconnected{scratch:init_res.trimmed_scratch});
                        if let Some(pin) = init_res.switch {
                            conn_state.step::<Output,Samps>(Switch(SwitchEvent::Activated(pin)), output);
                        }
                        if let Some(sr) = init_res.audio_connection {
                            conn_state.step::<Output,Samps>(Audio(AudioEvent::LockAcquired(sr.hz())), output);
                        }
                        Connected(conn_state)
                    }
                }
            }
        });
    }
}
