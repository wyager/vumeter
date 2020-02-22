extern crate vumeter_lib as vu;

use vu::bandpass as bp;
use vu::protocol as pkt;
use std::collections::VecDeque;

use serialport::prelude::*;
use std::time::Duration;
use std::fs::File;
use rodio::Source;
use std::io::BufReader;
use itertools::Itertools;

use biquad::frequency::*;




fn test_bandpass_1() {
    let sr = 40000i32.hz();
    let lo = 20i32.hz();
    let hi = 12000i32.hz();
    let mut bs = [bp::BandState::default(); 20];
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, ..bp::Params::default()};
    let mut bands = bp::Bands::init(&mut bs, params, sr).unwrap();
    (0..20).map(|i| bands.band(i)).for_each(|(lo,hi)| println!("{:?} {:?}", lo,hi));
    for i in 0..sr.hz() as i32 {
        let t = i as f32 / sr.hz();
        let freq = (lo.hz() * (hi.hz() / lo.hz()).powf(t));
        let sig = (t * 2. * core::f32::consts::PI * freq).cos();
        // let pwr = bs.step(sig);
        bands.step(sig);
        let pwr : Vec<i32> = bands.energies().map(|f| (f * 100.0) as i32).collect();
        if i % 500 == 1 {
            println!("{:04?}",pwr);
        }
    }
}

fn test_bandpass_2() {
    let sr = 40000.hz();
    let lo = 20.hz();
    let hi = 12000.hz();
    let mut bs = [bp::BandState::default(); 16];
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, ..bp::Params::default()};
    let mut bands = bp::Bands::init(&mut bs, params, sr).unwrap();
    (0..16).map(|i| bands.band(i)).for_each(|(lo,hi)| println!("{:?} {:?}", lo,hi));
    let count = (0..100).map(|i| {
        let (low,_) = bp::freq(100,lo,hi,i);
        // println!("{}",low);
        // let mut pwr : Vec<i32> = vec![];
        (0..sr.hz() as i32).map(|i| {
            let t = i as f32 / sr.hz();
            let freq = low.hz();
            let sig = (t * 2. * core::f32::consts::PI * freq).cos();
            bands.step(sig);
            let mut res : Vec<f32> = bands.energies().collect();
            res.sort_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
            let specificity = res[0]/res[1];
            let total = res.iter().sum();
            (total,specificity)
        }).last().unwrap_or((0.,0.))
    }).map(|(total,spec)| {
        println!("{} {}", total, spec);
    }).count();

} 

fn test_bandpass_3() {
    let sr = 40000.hz();
    let lo = 20.hz();
    let hi = 12000.hz();
    let update_rate = 2.hz();
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, ..bp::Params::default()};
    let mut l = [bp::BandState::default(); 16];
    let mut r = [bp::BandState::default(); 16];
    let mut p = [0.; 16];
    let mut s = [0.; 16];
    let mut scratch = bp::Scratch::new(&mut l,&mut r,&mut p,&mut s).unwrap();
    let mut vu = bp::VU::init(scratch, params, sr).unwrap();
    (8..=13).for_each(|i| {
        let (low,_) = bp::freq(20,lo,hi,i);
        println!("{:?}",low);
        // println!("{}",low);
        // let mut pwr : Vec<i32> = vec![];
        (0..(sr.hz()*10.) as i32).for_each(|i| {
            let t = i as f32 / sr.hz();
            let freq = low;
            let sig = [(0.2,0.8),(1.0,2.1),(2.3,0.5)].iter().map(|(mul,mag)| (t * 2. * core::f32::consts::PI * freq.hz() * mul).cos()*mag).sum();
            if let Some(bands) = vu.step(sig,sig) {
                let mut res : Vec<u32> = bands.map(|en| (en * 100.) as u32).collect();
                println!("{:04?}",res);
            }
        })
    })

} 

fn test_bandpass_4() {
    let sr = 40000.hz();
    let lo = 20.hz();
    let hi = 12000.hz();
    let mut bs = [bp::BandState::default(); 16];
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, filter_q:3., percentile_ndb:0.50, band_edge_multiplier:0.9, ..bp::Params::default()};
    let mut bands = bp::Bands::init(&mut bs, params, sr).unwrap();
    // (0..16).map(|i| bands.band(i)).for_each(|(lo,hi)| println!("{:?} {:?}", lo,hi));
    let count = (200..1800).map(|i| {
        let (low,_) = bp::freq(2000,lo,hi,i);
        (0..(sr.hz()*5.) as i32).map(|i| {
            let t = i as f32 / sr.hz();
            let freq = low.hz();
            let sig = (t * 2. * core::f32::consts::PI * freq).cos();
            bands.step(sig);
            let outputs : Vec<f32> = bands.energies().collect();
            (freq, outputs)
        }).last().unwrap()
    }).for_each(|(freq, energies)| {
        println!("{}",freq);
        let max = energies.iter().cloned().fold(0.,f32::max);
        println!("{}",vufmt(energies.iter().map(|x| x/max).collect()));
        // println!("{} {:?}", freq, energies)
    });

} 

fn vufmt(ens: Vec<f32>) -> String {
    let mut res = String::new();
    for cutoff in (0..=20).rev().map(|i| i as f32 / 20.) {
        for en in &ens {
            if en > &cutoff {
                res.push_str("  ## ");
            } else {
                res.push_str("     ");
            }
        }
        res.push('\n');
    }
    res
}


fn vu_meter() {
    let device = rodio::default_output_device().unwrap();
    let file = File::open("Crystals.wav").unwrap();
    let source1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let file = File::open("Crystals.wav").unwrap();
    let source2 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let sr = source1.sample_rate() as f32;
    println!("{}",sr);
    let channels = source1.channels();
    assert!(channels == 2);
    rodio::play_raw(&device, source1.convert_samples());
    let start = std::time::Instant::now();
    let lo = 25.hz();
    let hi = 6000.hz();
    let update_rate = 20.hz();
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, band_edge_multiplier:0.9, filter_q:3., percentile_ndb:0.7, ..bp::Params::default()};
    const N_BANDS : usize = 16;
    let mut l = [bp::BandState::default(); N_BANDS];
    let mut r = [bp::BandState::default(); N_BANDS];
    let mut p = [0.; N_BANDS];
    let mut s = [0.; N_BANDS];
    let scratch = bp::Scratch::new(&mut l,&mut r,&mut p,&mut s).unwrap();
    let mut vu = bp::VU::init(scratch,params,sr.hz()).unwrap();
    let samps : Vec<f32> = source2.convert_samples().collect();
    for (i,val) in samps[..].chunks_exact(2).enumerate() {
        if let Some(bands) = vu.step(val[0],val[1]) {
            let now = std::time::Instant::now();
            let diff = now - start;
            let implied_diff = std::time::Duration::from_millis(((i as f32 / sr)*1000.) as u64);
            if implied_diff > diff {
                let to_wait = implied_diff - diff;
                std::thread::sleep(to_wait);
            }
            let bands : Vec<f32> = bands.collect();
            println!("{}",vufmt(bands));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(10000));
    // println!("done");
}



fn vu_meter_mic() {

    use std::net::{UdpSocket,SocketAddr};
    let mut socket = UdpSocket::bind("0.0.0.0:7895").unwrap();
    let addr = SocketAddr::from(([192,168,1,118], 6776));

    

    let lo = 25.hz();
    let hi = 6000.hz();
    let update_rate = 60.hz();
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, band_edge_multiplier:0.9, filter_q:3., percentile_ndb:0.7, ..bp::Params::default()};
    const N_BANDS : usize = 16;
    let mut l = [bp::BandState::default(); N_BANDS];
    let mut r = [bp::BandState::default(); N_BANDS];
    let mut p = [0.; N_BANDS];
    let mut s = [0.; N_BANDS];
    let scratch = bp::Scratch::new(&mut l,&mut r,&mut p,&mut s).unwrap();
    


    use cpal::traits::{HostTrait,DeviceTrait,EventLoopTrait};
    use cpal::StreamData::Input;
    use cpal::UnknownTypeInputBuffer::*;
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host.default_input_device().unwrap();
    let format = device.default_input_format().unwrap();
    let in_stream_id = event_loop.build_input_stream(&device,&format).unwrap();
    let channels = format.channels as usize;
    println!("channels: {}",channels);
    assert!(channels == 2 || channels == 1);
    let sample_rate = format.sample_rate.0 as f32;

    let mut vu = bp::VU::init(scratch,params,sample_rate.hz()).unwrap();

    event_loop.run(move |stream_id, stream_result| {
        if let Input{buffer} = stream_result.unwrap(){
            let converted : Vec<f32> = match buffer {
                U16(buf) => buf.iter().cloned().map(|i| i as f32 / 65_536.).collect()  ,
                I16(buf) => buf.iter().cloned().map(|i| i as f32 / 32_768.).collect() ,
                F32(buf) => buf.iter().cloned().collect(),
            };

            for (val) in converted[..].chunks_exact(2) {
                let (l,r) = if channels == 1 {(val[0],val[0])} else {(val[0],val[1])};
                if let Some(bands) = vu.step(l,r) {
                    let mut buf = [0u8;120*3];
                    for (i,energy) in bands.enumerate() {
                        let b = (energy.powf(4.)*255.) as u8;
                        for o in (0..7*3) {
                            buf[i*7*3+o] = b;
                            buf[i*7*3+o] = b;
                            buf[i*7*3+o] = b;
                        }
                    }
                    socket.send_to(&buf,addr);
                }
            }
        }
    });
    // let sr = source1.sample_rate() as f32;
    // println!("{}",sr);
    // let channels = source1.channels();
    // assert!(channels == 2);
    // rodio::play_raw(&device, source1.convert_samples());
    // let start = std::time::Instant::now();
    

    // std::thread::sleep(std::time::Duration::from_millis(10000));
    // println!("done");
}

fn test_serial(){
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };
    let mut port = serialport::open_with_settings("/dev/tty.usbserial-AC009X7K", &s).unwrap();
    
    // for b in (0..0x3FF).map(|x| x*0x10) {
    //     for t in (0..8) {
    //         // let brightness = (energy*pkt::Brightness::MAX_BRIGHTNESS as f32) as u16;
    //         let brightness = pkt::Brightness::new(b).unwrap();
    //         let pkt = pkt::Pkt::mk_pkt(t as i8,brightness).unwrap();
    //         port.write_all(&pkt.format()).unwrap();
    //         port.flush();

    //         let dur = std::time::Duration::from_micros(500);
    //         std::thread::sleep(dur);
    //     }
    // }

    loop {

        for i in (0..2) {
            let b = pkt::Brightness::new(0x1234).unwrap();
            let p = pkt::Pkt::mk_pkt(i,b).unwrap();
            port.write_all(&p.format()).unwrap();
            port.flush().unwrap();
        }
        let dur = std::time::Duration::from_millis(4);
        std::thread::sleep(dur);
    }


}

fn vu_serial() {
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };
    let mut port = serialport::open_with_settings("/dev/tty.usbserial-AC009X7K", &s).unwrap();
    let device = rodio::default_output_device().unwrap();
    let file = File::open("Iteration.wav").unwrap();
    let source1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let file = File::open("Iteration.wav").unwrap();
    let source2 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let sr = source1.sample_rate() as f32;
    println!("{}",sr);
    let channels = source1.channels();
    assert!(channels == 2);
    rodio::play_raw(&device, source1.convert_samples());
    let start = std::time::Instant::now();
    let lo = 30.hz();
    let hi = 8000.hz();
    const N_BANDS : usize = 8;
    let update_rate = pkt::refresh_rate(N_BANDS).hz();
    println!("Refresh rate: {:?}",update_rate);
    // let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, band_edge_multiplier:0.9, filter_q:3., percentile_ndb:0.7, ..bp::Params::default()};
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, percentile_ndb:0.7, lowest_ndb:0.0001, ewma_time:Duration::from_millis(50), filter_q:1.5, ..bp::Params::default()};
    let mut l = [bp::BandState::default(); N_BANDS];
    let mut r = [bp::BandState::default(); N_BANDS];
    let mut p = [0.; N_BANDS];
    let mut s = [0.; N_BANDS];
    let scratch = bp::Scratch::new(&mut l,&mut r,&mut p,&mut s).unwrap();
    let mut vu = bp::VU::init(scratch,params,sr.hz()).unwrap();
    let samps : Vec<f32> = source2.convert_samples().collect();
    for (t,val) in samps[..].chunks_exact(2).enumerate() {
        if let Some(bands) = vu.step(val[0],val[1]) {
            let now = std::time::Instant::now();
            let diff = now - start;
            let implied_diff = std::time::Duration::from_millis(((t as f32 / sr)*1000.) as u64);
            if implied_diff > diff {
                let to_wait = implied_diff - diff;
                std::thread::sleep(to_wait);
                
            } else {
                println!("Lagging: {:?}", diff - implied_diff);
            }
            
            // Writing bands in reverse order ensures that all tubes will update within ~2ms of each other
            for (i,energy) in bands.enumerate().rev() {
                // let energy = 1.0;
                // let energy  = (i as f32) * 0.1 + 0.1;
                // let energy = (t as f32) / 40000.;
                // let energy = (energy * 2. * 3.14 / 4.).sin()/2. + 0.5;
                // let energy = 0.;
                let brightness = (energy*pkt::Brightness::MAX_BRIGHTNESS as f32) as u16;
                let brightness = pkt::Brightness::new(brightness).unwrap();
                let pkt = pkt::Pkt::mk_pkt(i as i8,brightness).unwrap();
                port.write_all(&pkt.format()).unwrap();
                // let dur = std::time::Duration::from_micros(100);
                // std::thread::sleep(dur);
                port.flush().unwrap(); 
            }
        }
    }
}

fn vu_udp() {
    use std::net::{UdpSocket,SocketAddr};
    let mut socket = UdpSocket::bind("0.0.0.0:7895").unwrap();
    let addr = SocketAddr::from(([192,168,1,118], 6776));
    let device = rodio::default_output_device().unwrap();
    let file = File::open("Kingdom.mp3").unwrap();
    let source1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let file = File::open("Kingdom.mp3").unwrap();
    let source2 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let sr = source1.sample_rate() as f32;
    println!("{}",sr);
    let channels = source1.channels();
    assert!(channels == 2);
    rodio::play_raw(&device, source1.convert_samples());
    let start = std::time::Instant::now();
    let lo = 20.hz();
    let hi = 12000.hz();
    const N_BANDS : usize = 120;
    let update_rate = 60.hz();
    println!("Refresh rate: {:?}",update_rate);
    // let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, band_edge_multiplier:0.9, filter_q:3., percentile_ndb:0.7, ..bp::Params::default()};
    let params = bp::Params{lowest_freq:lo, highest_freq:hi, render_rate:update_rate, percentile_ndb:0.7, lowest_ndb:0.0001, ewma_time:Duration::from_millis(50), filter_q:1.5, ..bp::Params::default()};
    let mut l = [bp::BandState::default(); N_BANDS];
    let mut r = [bp::BandState::default(); N_BANDS];
    let mut p = [0.; N_BANDS];
    let mut s = [0.; N_BANDS];
    let scratch = bp::Scratch::new(&mut l,&mut r,&mut p,&mut s).unwrap();
    let mut vu = bp::VU::init(scratch,params,sr.hz()).unwrap();
    let samps : Vec<f32> = source2.convert_samples().collect();
    let mut buf = [0u8;120*3];
    for (t,val) in samps[..].chunks_exact(2).enumerate() {
        if let Some(bands) = vu.step(val[0],val[1]) {
            let now = std::time::Instant::now();
            let diff = now - start;
            let implied_diff = std::time::Duration::from_millis(((t as f32 / sr)*1000.) as u64);
            if implied_diff > diff {
                let to_wait = implied_diff - diff;
                std::thread::sleep(to_wait);
                
            } else {
                println!("Lagging: {:?}", diff - implied_diff);
            }
            
            for (i,energy) in bands.enumerate().rev() {
                // let energy = 1.0;
                // let energy  = (i as f32) * 0.1 + 0.1;
                // let energy = (t as f32) / 40000.;
                // let energy = (energy * 2. * 3.14 / 4.).sin()/2. + 0.5;
                // let energy = 0.;
                let b = (energy.powf(4.)*255.) as u8;
                buf[i*3+0] = b;
                buf[i*3+1] = b;
                buf[i*3+2] = b;
                // let dur = std::time::Duration::from_micros(100);
                // std::thread::sleep(dur);
            }
            socket.send_to(&buf,addr);
        }
    }
}



fn test_pkt() {
    let ps = 4;
    use pkt::*;
    let mut queues : Vec<VecDeque<u8>> = vec![VecDeque::new();ps+1];
    let mut parsers : Vec<pkt::ParseState> = vec![ParseState::new(); ps];
    for message in (0..20).map(|i| Pkt::mk_pkt(i as i8,Brightness::new(i as u16*100).unwrap()).unwrap()) {
    // for message in (0..20).map(|i| pkt::Pkt::mk_pkt(i as i8,0x4000-1).unwrap()) {
    // for message in (0..20).map(|i| pkt::Pkt::Forward{val:0xBEF}) {
        queues[0].extend(message.format().iter());
    }
    #[derive(Clone,Copy)]
    enum PWM {A,B,C,D};
    use PWM::*;
    while queues[0..ps].iter().any(|v| v.len() > 0) {
        for i in 0..ps {
            if let Some(byte) = queues[i].pop_front() {
                let evt = parsers[i].step(&[A,B,C,D],byte);
                println!("{}", i);
                use pkt::Event::*;
                match evt {
                    Some(Send{pkt}) => {
                        println!("Send");
                        queues[i+1].extend(pkt.format().iter());
                    },
                    Some(Update{which,brightness}) => {
                        let which_c = match which {
                            A => 'a', B => 'b', C => 'c', D => 'd'
                        };
                        println!("PWM {} {:?}", which_c, brightness);
                    },
                    None => println!("Do nothing")
                }
            }
        }
        println!("{:?}",queues[ps]);
    }
    println!("{:?}", Pkt::parse(Pkt::format(Pkt::mk_pkt(27,Brightness::new(1022).unwrap()).unwrap())));
}

fn test_remapper() {
    use pkt::Brightness;


    for x in &[0u16,100,200,1000,2000,0x3FF0,0x3FFF] {
        let brightness = Brightness::new(*x).unwrap();
        println!("{} {}", x, brightness.map_to(0,100));
    }
}

fn test_ringbuf() {
    use vu::shared::RingBuf;
    let mut arr : [u8;4] = [0;4];
    let mut rb = RingBuf::new(&mut arr);
    let inputs = vec![7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26];
    let write_sizes = [2,1,3,5,7,0,5,2,1,3,4,5,3,2];
    let read_sizes = [3,1,7,0,2,6,1,3,4,2,0,5];
    let mut r_head = 0;
    let mut result : Vec<u8> = vec![];
    for (write_upto,read_upto) in write_sizes.iter().zip(read_sizes.iter()) {
        for _ in (0..*write_upto) {
            if r_head == inputs.len() {break};
            match rb.push_end(inputs[r_head]) {
                None => {break},
                Some(()) => {r_head += 1}
            }
        }
        for _ in (0..*read_upto) {
            match rb.with_front(Some) {
                None => {break},
                Some(val) => result.push(val)
            }
        }
    }
    if inputs != result {
        panic!("not equal")
    }
}

struct FakePin {
    name : String
}

impl vu::tube_hw::DigitalOutput for FakePin {
    fn set(&mut self, val : bool) {
        println!("Pin {} set to: {}", self.name, val);
    }
}

struct FakePWM {
    name : String
}

impl vu::tube_hw::PWMx4 for FakePWM {
    fn get_max_duty(&self) -> u32 {100}
    fn set_duty(&mut self, pin : vu::tube_hw::PWMPin, duty : u32) {
        println!("PWM {} Pin {:?} set to: {}", self.name, pin, duty);
    }
}

struct UARTSim {
    rx : Vec<u8>,
    tx : Vec<u8>,
    bufsize : usize
}

impl UARTSim {
    fn new(bufsize : usize) -> UARTSim {
        let mut rx = vec![0;bufsize];
        let mut tx = vec![0;bufsize];
        UARTSim{bufsize,rx,tx}
    }
}

struct TubeSim {
    u1 : UARTSim,
    u2 : UARTSim,
    logic : vu::tube_hw::State<FakePin,FakePWM>,
}

impl TubeSim {
    fn new(name : String) -> TubeSim {
        let fake_pin = FakePin{name:format!("pwr_{}",name)};
        let fake_pwm = FakePWM{name:format!("pwm_{}",name)};
        let logic = vu::tube_hw::State::new(fake_pin,fake_pwm);
        TubeSim{u1:UARTSim::new(64),u2:UARTSim::new(64),logic}
    }
}

struct AudioSim<'a> {
    u : UARTSim,
    logic : vu::audio_hw::DeviceState<'a>
}

impl<'a> AudioSim<'a> {
    fn new(scratch : bp::Scratch<'a>) -> AudioSim<'a> {
        AudioSim{u:UARTSim::new(512), logic:vu::audio_hw::DeviceState::new(scratch)}
    }
}

struct SimState<'a> {
    audio : AudioSim<'a>,
    tubes : Vec<TubeSim>
}



impl SimState<'_> {
    fn new<'a>(tube_boards : usize, scratch : bp::Scratch<'a>) -> SimState<'a> {
        let tubes : Vec<TubeSim> = (0..tube_boards).map(|i| TubeSim::new(format!("{}",i))).collect();
        let audio = AudioSim::new(scratch);
        SimState{audio,tubes}
    }
}

#[derive(Clone,Copy,PartialEq,PartialOrd,Debug)]
enum Event {
    AudioTimer, AudioUART, AudioSamps([(f32,f32);16]),
    TubeTimer(usize), TubeUART(usize)
}
use Event::*;

use std::collections::BTreeMap;
use rand::random;

fn random_offset(scale : i128) -> i128 {
    let rand = random::<f64>() - 0.5;
    (rand * 0.01 * scale as f64) as i128
}


fn times(spacing : Duration, time : Duration) -> impl Iterator<Item = i128> {
    let count = time.as_nanos() / spacing.as_nanos();
    let spacing = spacing.as_nanos();
    (0..count).map(move |i| (i * spacing) as i128 + random_offset(spacing as i128))
}





trait KV {
    type K;
    type V;
    fn split(self:Self) -> (Self::K,Self::V);
    fn unsplit(k:Self::K,v:Self::V) -> Self;
}


struct Merger<T> where T : Iterator, T::Item : KV, <<T as std::iter::Iterator>::Item as KV>::K: Ord {
    its : BTreeMap<<<T as std::iter::Iterator>::Item as KV>::K,(<<T as std::iter::Iterator>::Item as KV>::V,T)>
}

impl<K,V> KV for (K,V) {
    type K = K;
    type V = V;
    fn split(self:Self) -> (Self::K,Self::V) {self}
    fn unsplit(k:Self::K,v:Self::V) -> Self  {(k,v)}
}

impl<T> Merger<T> where T : Iterator, T::Item : KV, <<T as std::iter::Iterator>::Item as KV>::K : Ord + Copy {
    fn empty() -> Merger<T> {
        Merger{its:BTreeMap::new()}
    }
    fn add(&mut self, mut iterator : T) {
        iterator.next().map(|item| {
            let (key,val) = item.split();
            self.its.insert(key,(val,iterator));
        });
    }
}



impl<T> Iterator for Merger<T> where T : Iterator, T::Item : KV, <<T as std::iter::Iterator>::Item as KV>::K : Ord + Copy {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let smallest = self.its.keys().next().map(|x| *x)?;
        let (item, mut iterator) = self.its.remove(&smallest)?;
        self.add(iterator);
        Some(T::Item::unsplit(smallest, item))
    }
}



// fn merge<T>(iterators : &[T]) -> impl Iterator<Item=T::Item> where T : Iterator, T::Item : Ord {
//     // let mut map = BTreeMap::new();
//     // iterators.iterfor_each(|iterator| iterator.next().map(|item| map.insert(item,iterator)));
//     // Merger{its:map}
// }

fn events(time : Duration, tube_boards : usize) -> impl Iterator<Item = Event> {
    let mut merger : Merger<Box<dyn Iterator<Item = (i128, Event)>>>  = Merger::empty();
    let aud_tmr = times(Duration::from_millis(25),time).map(|t| (t,AudioTimer));
    merger.add(Box::new(aud_tmr));
    let tube_tmr = times(Duration::from_millis(50),time).map(|t| (t,TubeTimer(0)));
    merger.add(Box::new(tube_tmr));
    // let evts = its.iter().fold(std::iter::empty, |it1, it2| it1.merge(it2));
    // let evts = aud_tmr.merge(tube_tmr);
    // evts.map(|(t,e)| e)
    merger.map(|(t,e)| e)
}

// fn events(time : Duration, tube_boards : usize) -> BTreeMap<i128, Vec<Event>> {
//     let mut map = BTreeMap::new();
//     let mut events = vec![(AudioTimer, Duration::from_millis(25))];
//     for i in 0..tube_boards {
//         events.push((TubeTimer(i), Duration::from_millis(50)));
//     }
//     for t in std::iter::once(AudioUART).chain((0..tube_boards).map(TubeUART)) {
//         let byte_freq = 460800. / 10.;
//         let byte_period = 1. / byte_freq;
//         events.push((t, Duration::from_secs_f64(byte_period)));
//     }
//     for (evt,spacing) in events.iter().cloned() {
//         times(spacing, time).for_each(|time| {
//             let entry = map.entry(time).or_insert(vec![]);
//             entry.push(evt)
//         });
//     }
//     map
// }

// fn uart_times(byte_spacing : Duration, count : 




fn main() {
    // events(Duration::from_millis(200), 1).for_each(|event| println!("{:?}",event));
    // println!("{:?}",bp::freqs::<typenum::U4>(100.,1000.));
    // test_bandpass_1();
    // test_bandpass_2();
    // test_bandpass_3();
    // test_bandpass_4();
    vu_meter();
    // vu_serial();
    // vu_udp();
    // vu_meter_mic();
    // test_serial();
    // test_pkt();
    // test_remapper()
    // test_ringbuf();
}
