
pub trait Consumer<T> {
    fn consume(&mut self, t : T);
}

pub struct BlackHole;

impl<T> Consumer<T> for BlackHole {
    fn consume(&mut self, _ : T) {}
}


pub struct RingBuf<'a,T> {
    entries : &'a mut [T],
    start : usize,
    len : usize
}

impl<T:Copy> RingBuf<'_,T> {
    pub fn new<'a>(entries : &'a mut [T]) -> RingBuf<'a,T> {
        RingBuf{entries, start:0, len:0}
    }
    pub fn space(&self) -> usize {
        self.entries.len() - self.len
    }
    pub fn len(&self) -> usize {self.len}
    fn nth<'a>(&'a mut self, n : usize) -> &'a mut T {
        if self.start + n < self.entries.len() {
            &mut self.entries[self.start + n]
        } else {
            &mut self.entries[self.start + n - self.entries.len()]
        }
    }
    pub fn push_end(&mut self, val : T) -> Option<()> {
        if self.space() > 0 {
            *self.nth(self.len) = val;
            self.len += 1;
            Some(())
        } else {
            None
        }
    }
    fn peek_front(&mut self) -> Option<T> {
        if self.len > 0 {
            Some(*self.nth(0))
        } else {
            None
        }
    }
    fn drop_front(&mut self) {
        if self.len > 0 {
            self.start += 1;
            if self.start == self.entries.len() {self.start = 0};
            self.len -= 1;
        } else {
            panic!("Dropping front from empty buf");
        }
    }
    pub fn with_front<F,U>(&mut self, f:F) -> Option<U> where F:FnOnce(T) -> Option<U> {
        let front = self.peek_front()?;
        let result = f(front)?;
        self.drop_front();
        Some(result)
    }
}

impl<W:Copy> Consumer<&[W]> for RingBuf<'_,W> {
    fn consume(&mut self, slice:&[W]) {
        if self.space() >= slice.len() {
            slice.iter().for_each(|w| self.push_end(*w).unwrap());
        }
    }
}


#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum Transition<State> {
    In(State),
    TransitioningTo{state:State,ticks:u64}
}
use Transition::*;

impl<State> Transition<State> {
    pub fn step(self) -> Transition<State> {
        match self {
            In(state) => In(state),
            TransitioningTo{state,ticks} => {
                if ticks == 0 {
                    In(state)
                } else {
                    TransitioningTo{state,ticks:ticks-1}
                }
            }
        }
    }
}





