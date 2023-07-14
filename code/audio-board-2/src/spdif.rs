//use cty;
use vumeter_lib::audio_hw::AudioEvent;
//use AudioEvent::*;

//#[link(name = "libteensy")]
extern "C" {
    fn copy_from_spdif_buffer(max : cty::uint32_t, l : *mut f32, r : *mut f32) -> cty::uint32_t;
    fn config_spdif();
    fn get_sample_rate() -> f32;
    pub fn spdif_dma_isr();
    pub fn spdif_isr();
}

pub struct SPDIF {
    lbuf : [f32;128],
    rbuf : [f32;128],
    sr : Option<f32>
}

#[no_mangle]
static mut SPDIF_TAKEN: bool = false;



fn very_different(a : Option<f32>, b : Option<f32>) -> bool {
    match a {
        None => match b {
            None => false,
            Some(_) => true
        },
        Some(af) => match b {
            None => true,
            Some(bf) => {
                let ratio = af / bf;
                ratio > 1.001 || ratio < 0.999
            }
        }
    }
}

impl SPDIF {
    pub fn initialize() -> Option<SPDIF> {
        unsafe {
            if SPDIF_TAKEN {return None};
            SPDIF_TAKEN=true;
            config_spdif();   
        }
        Some(SPDIF{lbuf:[0.;128],rbuf:[0.;128],sr:None})
    }
    pub fn read<'a>(&'a mut self) -> Option<AudioEvent<impl Iterator<Item=(f32,f32)> + 'a>> {
        use AudioEvent::*;
        let sr = unsafe { get_sample_rate() };
        let sr = if sr < 0. {None} else {Some(sr)};
        if very_different(sr, self.sr) {
            self.sr = sr;
            match sr {
                None => Some(LockLost),
                Some(freq) => Some(LockAcquired(freq))
            }
        } else {
            match self.sr {
                None => None, // Waiting for SPDIF to get plugged in
                Some(_) => {
                    let copied = unsafe {
                        copy_from_spdif_buffer(128, self.lbuf.as_mut_ptr(), self.rbuf.as_mut_ptr())
                    };
                    if copied > 0 {
                        Some(Samples(self.lbuf[0..copied as usize].iter().cloned().zip(self.rbuf[0..copied as usize].iter().cloned())))
                    } else {
                        None
                    }
                }
        }
        }
    }

}

