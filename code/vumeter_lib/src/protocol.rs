

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Brightness(u16);
impl Brightness {
    pub const ZERO : Brightness = Brightness(0);
    pub const MAX_BRIGHTNESS : u16 = (1<<14) - 1;
    pub fn new(val:u16) -> Option<Brightness> {
        if val < (1 << 14) {
            Some(Brightness(val))
        } else {
            None
        }
    }
    pub fn map_to(&self, out_lo : u32, out_hi : u32) -> u32 {
        let Brightness(x) = self;
        let in_lo = 0;
        let in_hi = (1 << 14) - 1;
        let in_diff = in_hi - in_lo;
        let out_diff = out_hi - out_lo; 
        let x = x - in_lo;
        let x = ((x as u64 * out_diff as u64) / in_diff as u64) as u32;
        x + out_lo
    }
}




#[derive(Clone,Copy,Debug)]
pub enum Pkt {
    Set {ttl : i8, brightness : Brightness},
    Forward {val : u16}
}

#[derive(Debug)]
pub enum PktErr {
    BadHeader,
    TooBig,
    SumCheckFailed,
    XorCheckFailed
}

impl Pkt {
    pub const PROBE_TTL : i8 = 126;
    pub const FWD_TTL : i8 = 127;
    pub const PKT_LEN : usize = 6; // 1 byte header, 1 byte ttl, 2 byte length, 2 byte checksum
    pub fn probe_pkt() -> Pkt {
        Pkt::Set{ttl:Pkt::PROBE_TTL,brightness:Brightness::ZERO}
    }
    pub fn mk_pkt(ttl : i8, brightness: Brightness) -> Option<Pkt> {
        if ttl >= 0&& ttl < Pkt::PROBE_TTL {
            Some(Pkt::Set {ttl,brightness})
        } else {
            None
        }
    }
    fn checksums(data:&[u8]) -> (u8,u8) {
        let (sum, xor) = data.iter().fold((0b01010101u8, 0b00101010u8), |(sum,xor), b| {
            (sum.overflowing_add(*b).0, xor ^ b)
        });
        let mask = 0b111_1111;
        (sum & mask, xor & mask)
    }
    pub fn format(self) -> [u8; Pkt::PKT_LEN] {
        use Pkt::*;
        let data : [u8;3] = match self {
            Set{ttl,brightness} => [ttl as u8, (brightness.0 >> 7) as u8, (brightness.0 & 0b111_1111) as u8],
            Forward{val} => [Pkt::FWD_TTL as u8, (val >> 7) as u8, (val & 0b111_1111) as u8]
        };
        let (sum,xor) = Self::checksums(&data[..]);
        [0xFF, data[0], data[1], data[2], sum, xor]
    }
    
    pub fn parse(buf : [u8; Pkt::PKT_LEN]) -> Result<Pkt,PktErr> {
        use PktErr::*;
        if buf[0] != 0xFF {return Err(BadHeader)}
        if buf[1..].iter().any(|x| *x > 127) {return Err(TooBig)}
        let (sum,xor) = Self::checksums(&buf[1..=3]);
        if sum != buf[4] {return Err(SumCheckFailed)}
        if xor != buf[5] {return Err(XorCheckFailed)}
        let data = ((buf[2] as u16) << 7) + (buf[3] as u16);
        let brightness = Brightness::new(data).unwrap();
        let ttl = buf[1] as i8;
        let pkt = match ttl {
            Pkt::FWD_TTL => Pkt::Forward {val:data},
            _ => Pkt::Set {ttl:ttl,brightness}
        };
        Ok(pkt)
    }
}

pub struct PktIter {
    bytes : [u8;6],
    i : usize
}
impl Iterator for PktIter {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.i < self.bytes.len() {
            let byte = self.bytes[self.i];
            self.i += 1;
            Some(byte)
        } else {
            None
        }
    }
}
impl ExactSizeIterator for PktIter {
    fn len(&self) -> usize {
        Pkt::PKT_LEN - self.i
    }
}

impl<'a> core::iter::IntoIterator for &'a Pkt {
    type Item = u8;
    type IntoIter =  PktIter;
    fn into_iter(self) -> PktIter {
        PktIter{bytes:self.format(),i:0}
    }
}

use crate::shared::{Consumer,RingBuf};

// This sucks, but something is messed up with the constraint inference
// when I try to do it properly
impl Consumer<Pkt> for RingBuf<'_,u8> {
    fn consume(&mut self, pkt : Pkt) {
        let slice : &[u8] = &pkt.format();
        self.consume(slice);
    }
}


#[derive(Clone,Copy)]
pub struct ParseState {
    buffer : [u8; Pkt::PKT_LEN],
    received : usize
}

pub enum Event<A> {Send {pkt:Pkt}, Update {which:A, brightness:Brightness}}

impl ParseState {
    pub fn new() -> ParseState {
        ParseState{buffer:[0;Pkt::PKT_LEN], received : 0}
    }
    
    pub fn step_(&mut self, x : u8) -> Option<Pkt> {
        match x {
            0xFF => {
                self.buffer[0] = 0xFF;
                self.received = 1;
                None
            },
            x if x > 127 => {
                self.received = 0;
                None // Invalid
            }, 
            x => {
                self.buffer[self.received] = x;
                self.received += 1;
                if self.received == Pkt::PKT_LEN {
                    self.received = 0;
                    Pkt::parse(self.buffer).ok()
                } else {None}
            }
        }
    }

    pub fn step<A : Copy>(&mut self, events:&[A], x:u8) -> Option<Event<A>> {
        use Pkt::*;
        use Event::*;
        match self.step_(x) {
            None => None,
            Some(pkt) => match pkt {
                Forward {val} => Some(Send {pkt:(Forward {val})}),
                Set {ttl,brightness} => {
                    let len = events.len() as i8;
                    if ttl >= len {
                        Some(Send {pkt:(Set {ttl:ttl-len,brightness})})
                    } else {
                        Some(Update {which:events[ttl as usize], brightness})
                    }
                }
            }
        }
    }
}

pub fn refresh_rate(tubes : usize) -> f32 {
    let tubes = tubes as f32;
    let baud = 460800.;
    let baud = baud * 0.9; // Allow a little bit of overhead
    let bits_per_cycle = tubes * 10. * 6.;
    let rate = baud / bits_per_cycle;
    if rate > 240. {240.} else {rate} // Not much point going above this
}
