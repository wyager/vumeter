use libm::powf;
use biquad::DirectForm2Transposed;
use biquad::coefficients::Coefficients;
use biquad::coefficients::Type;
use biquad::Biquad;
// use core::f32::consts::PI;
use core::time::Duration;
use biquad::frequency::*;


#[derive(Debug,Clone,Copy)]
pub struct Params {
    pub lowest_freq : Hertz<f32>, // What's the lowest frequency we process?
    pub highest_freq : Hertz<f32>, // And the highest?
    pub ewma_cycles : f32, // At least how many cycles of the lowest frequency in this band should the EWMA take to decay to 1/2?
    pub ewma_time : Duration, // At least how many seconds should the EWMA take to decay to 1/2?
    pub filter_q : f32, // Q factor of bandpass filters
    pub percentile_ndb : f32, // What percentile of samples should be used to calculate the ndb reference point?
    pub decay_time_ndb : Duration, // How many seconds should it take the ndb reference point to decay to 1/2
    pub increase_time_ndb : Duration, // How many seconds should it take the ndb reference point to increase to 1/2 the instantaneous max?
    pub lowest_ndb : f32, // How low should the ndb tracker be allowed to go, in energy output terms?
    pub sine_remap_power : f32, // Exponent on sine function used to map [0,40] (dB) to [0,1]
    pub render_rate : Hertz<f32>, // How frequently to recalculate VU values
    pub lowest_to_center : f32, // Determines the N in N db. The ratio we use relative to the center to chop to zero.
    pub highest_to_center : f32, // Determines the N in N db. The ratio we use relative to the center to chop to zero.
    pub band_edge_multiplier : f32, // [0,1]. Determines how far away from the nominal band edge the high/low pass filters are set to
}

impl Params {
    pub fn default() -> Params {
        Params {
            lowest_freq : 40.hz(), // What's the lowest frequency we process?
            highest_freq : 10_000.hz(), // And the highest?
            ewma_cycles : 1., // At least how many cycles of the lowest frequency in this band should the EWMA take to decay to 1/2?
            ewma_time : Duration::from_millis(30), // At least how many seconds should the EWMA take to decay to 1/2?
            filter_q : 2., // Q factor of bandpass filters
            percentile_ndb : 0.5, // What percentile of samples should be used to calculate the ndb reference point?
            decay_time_ndb : Duration::from_secs(10), // How many seconds should it take the ndb reference point to decay to 1/2
            lowest_ndb : 0.01, // How low should the ndb tracker be allowed to go, in energy output terms?
            increase_time_ndb : Duration::from_secs(1), // How many seconds should it take the ndb reference point to increase to 1/2 the instantaneous max?
            sine_remap_power : 4., // Exponent on sine function used to map [0,log_10(lowest_to_center)*2] (dB) to [0,1]
            render_rate : 144.hz(), // How frequently to recalculate VU values
            lowest_to_center : 10., // Determines our dynamic range. 
            highest_to_center : 20., // Determines our dynamic range. ltc = 10, htc = 20 => 30db dynamic range, with midpoint 1/3 up the tube
            band_edge_multiplier : 0.9,
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct BandState {
    lp_lopass : DirectForm2Transposed<f32>,
    lp_hipass : DirectForm2Transposed<f32>,
    energy : f32,
    alpha : f32 // EWMA on the energy
}

impl BandState {
    fn new(q : f32, ewma_cycles : f32, ewma_time : Duration, sr : Hertz<f32>, flo : Hertz<f32>, fhi : Hertz<f32>) -> Result<BandState, biquad::Errors>  {
        let cycle_steps = sr.hz() / flo.hz();
        let ewma_halftime_cycles = cycle_steps * ewma_cycles;
        let ewma_halftime_time = ewma_time.as_secs_f32() * sr.hz();
        let ewma_halftime = if ewma_halftime_cycles > ewma_halftime_time {ewma_halftime_cycles} else {ewma_halftime_time};
        let alpha = powf(0.5, 1. / ewma_halftime); 
        let coeffs_hipass = Coefficients::<f32>::from_params(Type::HighPass, sr, flo, q)?;
        let coeffs_lopass = Coefficients::<f32>::from_params(Type::LowPass, sr, fhi, q)?;
        let lp_hipass = DirectForm2Transposed::<f32>::new(coeffs_hipass);
        let lp_lopass = DirectForm2Transposed::<f32>::new(coeffs_lopass);
        Ok(BandState {alpha, energy:0., lp_lopass, lp_hipass})
    }
    fn step(&mut self, samp : f32) {
        let o_lo = self.lp_lopass.run(samp);
        let o_hi = self.lp_hipass.run(o_lo);
        self.energy = self.alpha * self.energy + (1.0 - self.alpha) * (o_hi * o_hi);
    }
    pub fn default() -> BandState {
        BandState::new(1., 1., Duration::from_millis(10), 1000.hz(),1.hz(),2.hz()).unwrap()
    }
}


pub fn freq(num : usize, lo : Hertz<f32>, hi : Hertz<f32>, i : usize) -> (Hertz<f32>, Hertz<f32>) {
    let (hi,lo) = (hi.hz(), lo.hz());
    let ratio = hi / lo;
    let f1 = lo * powf(ratio, i as f32 / num as f32);
    let f2 = lo * powf(ratio, (i+1) as f32 / num as f32);
    (f1.hz(),f2.hz())
}

pub struct Bands<'a> {
    bands : &'a mut[BandState],
    lo : Hertz<f32>,
    hi : Hertz<f32>
} 

impl Bands<'_> {
    pub fn init<'a>(bands : &'a mut [BandState], params : Params, sr : Hertz<f32>) -> Result<Bands<'a>, biquad::Errors> {
        let len = bands.len();
        let lo = params.lowest_freq;
        let hi = params.highest_freq;
        for (i,bs) in bands.iter_mut().enumerate() {
            let (flo, fhi) = freq(len,lo,hi,i);
            let flo = (flo.hz() / params.band_edge_multiplier).hz();
            let fhi = (fhi.hz() * params.band_edge_multiplier).hz();
            *bs = BandState::new(params.filter_q, params.ewma_cycles, params.ewma_time, sr,flo,fhi)?;
        }
        Ok(Bands{bands,lo,hi})
    }
    pub fn step(&mut self, samp : f32) {
        self.bands.iter_mut().for_each(|band| band.step(samp));
    }
    pub fn band(&self, i : usize) -> (Hertz<f32>,Hertz<f32>) {
        freq(self.bands.len(),self.lo,self.hi,i)
    }
    pub fn energies<'a>(&'a mut self) -> impl Iterator<Item=f32> + 'a {
        self.bands.iter().map(|b| b.energy)
    }
}

pub fn sum<'a, L : Iterator<Item=f32> + 'a , R : Iterator<Item=f32> + 'a >(l : L, r : R) -> impl Iterator<Item=f32> + 'a{
    l.zip(r).map(|(l,r)| l + r)
}




// Keeps track of a reference point, decaying slowly or bumping up quickly
struct BumpTracker {
    val : f32,
    minimum : f32,
    up_alpha : f32,
    down_alpha : f32
}

impl BumpTracker {
    fn new(minimum : f32, up_alpha : f32, down_alpha : f32) -> BumpTracker {
        BumpTracker{val:minimum,minimum,up_alpha,down_alpha}
    }
    fn step(&mut self, samp : f32) {
        if samp > self.val {
            self.val = self.up_alpha * self.val + (1. - self.up_alpha) * samp;
        } else {
            self.val = self.down_alpha * self.val;
        }
        if self.val < self.minimum {
            self.val = self.minimum;
        }
    }
}


pub struct Scratch<'a> {
    l : &'a mut [BandState],
    r : &'a mut [BandState],
    p : &'a mut [f32], // Percentile calc
    s : &'a mut [f32], // Sum calc
}

impl<'a> Scratch<'a> {
    pub fn new(l : &'a mut [BandState], r : &'a mut [BandState], p : &'a mut[f32], s : &'a mut[f32]) ->  Option<Self>{
        if l.len() != r.len() || l.len() != p.len() || l.len() != s.len() {None}
        else {Some(Scratch{l,r,p,s})}
    }
    pub fn length(&self) -> usize {
        self.l.len()
    }
    pub fn trim(self,new_len : usize) -> Option<Self> {
        if new_len <= self.length() {
            Some(Scratch {
                l : &mut self.l[0..new_len],
                r : &mut self.r[0..new_len],
                p : &mut self.p[0..new_len],
                s : &mut self.s[0..new_len],
            })
        } else {
            None
        }
    }
}

pub struct VU<'a> {
    l : Bands<'a>,
    r : Bands<'a>,
    ctr : usize,
    decimation : usize,
    pub sr : Hertz<f32>,
    percentile_scratch : &'a mut[f32],
    percentile_tracker : BumpTracker,
    sum_scratch : &'a mut[f32],
    params : Params
}

impl<'t> VU<'t> {
    pub fn init<'a>(scratch : Scratch<'a>, params : Params, sr : Hertz<f32>) -> Result<VU<'a>, biquad::Errors>{
        let l = Bands::init(scratch.l,params,sr)?;
        let r = Bands::init(scratch.r,params,sr)?;
        let decimation = (sr.hz() / params.render_rate.hz()) as usize;
        let down_alpha = powf(0.5, 1.0 / (params.render_rate.hz() * params.decay_time_ndb.as_secs_f32()));
        let up_alpha = powf(0.5, 1.0 / (params.render_rate.hz() * params.increase_time_ndb.as_secs_f32()));
        let bump_tracker = BumpTracker::new(params.lowest_ndb, up_alpha, down_alpha);
        Ok(VU{l,r,ctr:0
            ,decimation,percentile_scratch:scratch.p
            ,percentile_tracker:bump_tracker,sum_scratch:scratch.s
            ,params,sr})
    }

    // Process:
    // L/R => bandpass => energy => EWMA => decimate => sum L/R => map Mth %ile to ndb. Cut off below 0db => clamp to 40db w/sine quarter-wave to the 4th
    //                                                   \=> track Mth percentile =/^
    pub fn step<'a>(&'a mut self, ls : f32, rs : f32) -> Option<impl Iterator<Item=f32> + DoubleEndedIterator + ExactSizeIterator + 'a> {
        self.l.step(ls);
        self.r.step(rs);
        self.ctr += 1;
        if self.ctr == self.decimation {
            self.ctr = 0;
            let sorted = &mut self.percentile_scratch;
            let bands = &mut self.sum_scratch;
            for (i,energy) in sum(self.l.energies(), self.r.energies()).enumerate() {
                sorted[i] = energy;
                bands[i] = energy;
            }
            sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            let floating_index = self.params.percentile_ndb * sorted.len() as f32;
            let hi_index = floating_index as usize;
            let lo_index = hi_index - 1;
            let lo_mult = hi_index as f32 - floating_index;
            let hi_mult = 1. - lo_mult;
            let interpolated = lo_mult * sorted[lo_index] + hi_mult * sorted[hi_index];
            self.percentile_tracker.step(interpolated);
            let zero_point = self.percentile_tracker.val / self.params.lowest_to_center; // Mth percentile = ndB
            let res = bands.iter().map(move |en| en / zero_point).map(|en| if en < 1. {1.} else {en});
            let midpoint = libm::logf(self.params.lowest_to_center);
            // let multiplier = PI / (2. * 2. * midpoint);
            // let power = self.params.sine_remap_power;
            // let res = res.map(libm::logf).map(move |x| if x > 2.*midpoint {1.} else {libm::powf(libm::sinf(x * multiplier),power)});
            let max = libm::logf(self.params.highest_to_center) + midpoint;
            let res = res.map(libm::logf).map(move |x| if x > max {1.} else {x / max});//.enumerate().rev();
            Some(res)
        }
        else {None}
        // 
    }

    pub fn teardown(self) -> Scratch<'t> {
        Scratch{l:self.l.bands, r:self.r.bands, p:self.percentile_scratch, s:self.sum_scratch}
    }
    pub fn num_bands(&self) -> usize {
        self.percentile_scratch.len()
    }
}
