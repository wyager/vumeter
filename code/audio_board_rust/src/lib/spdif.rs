use cortex_m::interrupt::{enable,disable};
use vumeter_lib::audio_hw::AudioEvent;
//use AudioEvent::*;
use teensy4_bsp as bsp;
use cortex_m::peripheral::NVIC;
use bsp::ral::{dma,dmamux,spdif,ccm};
use bsp::pins::imxrt_iomuxc::{configure, Config, OpenDrain, PullKeeper, Speed, SlewRate, DriveStrength, Hysteresis};
use core::ptr;

pub struct SPDIF {
    lbuf : [f32;128],
    rbuf : [f32;128],
    sr : Option<f32>
}

fn nvic_set_priority_spdif(irqnum: u32, priority: u8) {
    let addr = 0xE000E400 as *mut u8;
    unsafe {
        ptr::write_volatile(addr.offset(irqnum as isize), priority);
    }
}

#[no_mangle]
static mut SPDIF_TAKEN: bool = false;

const BUFFERLEN: u32 = 1536;
static mut BUFFER_R: [f32; BUFFERLEN as usize] = [0.0; BUFFERLEN as usize];
static mut BUFFER_L: [f32; BUFFERLEN as usize] = [0.0; BUFFERLEN as usize];
static mut BUFFER_OFFSET: u32 = 0;
static mut BUFFER_READ_OFFSET: u32 = 0;
const SPDIF_RX_BUFFER_LENGTH: usize = 128;

#[link_section = ".dmabuffers"]
static mut SPDIF_RX_BUFFER: [i32; SPDIF_RX_BUFFER_LENGTH] = [0; SPDIF_RX_BUFFER_LENGTH];

const CCM_CCGR_ON: u32 = 3;
const DMA_CR_GRP1PRI: u32 = 1 << 10;
const DMA_CR_EMLM: u32 = 1 << 7;
const DMA_CR_EDBG: u32 = 1 << 1;

const DMA_TCD_NBYTES_SMLOE: u32 = 1 << 31;
const DMA_TCD_CSR_INTHALF: u16 = 0x0004;
const DMA_TCD_CSR_INTMAJOR: u16 = 0x0002;

const DMAMUX_SOURCE_SPDIF_RX: u32 = 85;
const DMAMUX_CHCFG_ENBL: u32 = 1 << 31;


const SPDIF_SIC_LOCKLOSS: u32 = 1 << 2;
const SPDIF_SIC_LOCK: u32 = 1 << 20;

const SPDIF_SIE_LOCKLOSS: u32 = 1 << 2;
const SPDIF_SIE_LOCK: u32 = 1 << 20;

const SPDIF_SCR_RXFIFO_OFF_ON: u32 = 1 << 22;
const SPDIF_SCR_RXFIFO_CTR: u32 = 1 << 23;
const SPDIF_SCR_RXAUTOSYNC: u32 = 1 << 18;
const SPDIF_SCR_DMA_RX_EN: u32 = 1 << 9;

const CHANNEL_DMA: u8 = 3;

pub fn ccm_ccgr5_dma(n: u32) -> u32 {
    (n & 0x03) << 6
}

fn dma_tcd_attr_ssize(n: u16) -> u16 {
    (n & 0x7) << 8
}

fn dma_tcd_attr_dsize(n: u16) -> u16 {
    (n & 0x7) << 0
}

fn dma_tcd_nbytes_mloffyes_nbytes(n: u32) -> u32 {
    n & 0x3FF
}

fn dma_tcd_nbytes_mloffyes_mloff(n: i32) -> u32 {
    ((n & 0x3FF)<<10).try_into().unwrap()
}

pub fn ccm_ccgr5_spdif(n: u32) -> u32 {
    (n & 0x03) << 14
}

pub fn spdif_scr_rxfifofull_sel(n: u32) -> u32 {
    (n & 0x03) << 19
}

pub fn spdif_scr_usrc_sel(n: u32) -> u32 {
    (n & 0x03) << 0
}

pub fn spdif_srpc_gainsel(n: u32) -> u32 {
    (n & 0x07) << 3
}

fn copy_from_spdif_buffer(max: u32, out_l: &mut [f32], out_r: &mut [f32]) -> u32 {
    disable();
    let b_o = unsafe { BUFFER_OFFSET };
    let mut b_r_o = unsafe { BUFFER_READ_OFFSET };
    unsafe { enable() };
    let remaining = if b_r_o <= b_o {
        b_o - b_r_o
    } else {
        BUFFERLEN - b_r_o + b_o
    };
    let to_copy = remaining.min(max);
    for i in 0..to_copy {
        out_l[i as usize] = unsafe { BUFFER_L[b_r_o as usize] };
        out_r[i as usize] = unsafe { BUFFER_R[b_r_o as usize] };
        let _ = b_r_o.wrapping_add(1);
        if b_r_o == BUFFERLEN {
            b_r_o = 0;
        };
    }
    disable();
    unsafe { BUFFER_READ_OFFSET = b_r_o };
    unsafe { enable() };
    to_copy
}

pub fn spdif_isr() {
    let spdif_local = unsafe{ bsp::ral::spdif::SPDIF::instance()};
    spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCKLOSS);
    spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCK);
}

pub fn spdif_dma_isr() {
    let dma_local = unsafe {dma::DMA::instance()};
    let spdif_rx_buffer_address: *const i32 = unsafe { SPDIF_RX_BUFFER.as_ptr() };
    let spdif_rx_buffer_length: usize = unsafe { SPDIF_RX_BUFFER.len() };
    let mut core = cortex_m::Peripherals::take().unwrap();
    // Clear interrupts
    dma_local.CINT.write(CHANNEL_DMA);
    let daddr: u32 = dma_local.TCD[3].TCD_DADDR.read();

    let mut src: &[i32] = unsafe { &SPDIF_RX_BUFFER };
    if daddr < (spdif_rx_buffer_address as u32 + (spdif_rx_buffer_length/2) as u32) {
        src = unsafe { &SPDIF_RX_BUFFER [ spdif_rx_buffer_length/2..]};
    }
    else {
        src = unsafe { &SPDIF_RX_BUFFER[..]};
    }

    if unsafe { BUFFER_OFFSET >= BUFFER_READ_OFFSET || (BUFFER_OFFSET + (spdif_rx_buffer_length/4) as u32) < BUFFER_READ_OFFSET } {
        core.SCB.clean_dcache_by_address(src.as_ptr() as usize, (spdif_rx_buffer_length/2)*4);
    
        for i in 0..spdif_rx_buffer_length/4 {
            let max: f32 = 32768.0;
        
            let buff_offset_i: usize = unsafe { BUFFER_OFFSET as usize } + i;
            // There's a bit of a weird sign conversion thing going on here;
            // we have a *signed* 24-bit value in the *lower* 24 bits of the 
            // 32-bit locus. One solution is to just toss the bottom 8 bits 
            // and then interpret this as a signed 16-bit int.
            let n_l_index: usize = (2 * i) + 0;
            let n_l: u32 = src[n_l_index] as u32;
            unsafe {
            let i_l: i16 = (n_l >> 8).try_into().unwrap();
            BUFFER_L[buff_offset_i] = i_l as f32 / max;
            }
        
            let n_r_index: usize = (2 * i) + 1;
            let n_r: u32 = src[n_r_index] as u32;
            unsafe {
            let i_r: i16 = (n_r >> 8).try_into().unwrap();
            BUFFER_R[buff_offset_i] = i_r as f32 / max;
            }
        }
        let quarter_spdif_rx_buffer_length: u32 = spdif_rx_buffer_length as u32/4;
        unsafe { BUFFER_OFFSET = ( BUFFER_OFFSET + quarter_spdif_rx_buffer_length) % BUFFERLEN };
    }
}

// Written to by ISR, read by "userspace" code
pub fn config_spdif() {
    let spdif_local = unsafe{ spdif::SPDIF::instance()};
    let ccm_local = unsafe {ccm::CCM::instance()};
    let dma_local = unsafe {dma::DMA::instance()};
    let dmamux_local = unsafe {dmamux::DMAMUX::instance()};

    //let iomuxc_local = unsafe { ral::iomuxc::IOMUXC::instance() };

    let spdif_rx_buffer_address: *const i32 = unsafe { SPDIF_RX_BUFFER.as_ptr() };
    let spdif_rx_buffer_length: usize = unsafe { SPDIF_RX_BUFFER.len() };
    let sizeof_spdif_rx_buffer :u16 = spdif_rx_buffer_length as u16;

    let no_byte_minor_loop = 2*4;

    ccm_local.CCGR5.write(ccm_local.CCGR5.read() | ccm_ccgr5_dma(CCM_CCGR_ON));
    dma_local.CR.write(DMA_CR_GRP1PRI | DMA_CR_EMLM | DMA_CR_EDBG);
    dma_local.CERQ.write(CHANNEL_DMA);
    dma_local.CERR.write(CHANNEL_DMA);
    dma_local.CEEI.write(CHANNEL_DMA);
    dma_local.CINT.write(CHANNEL_DMA);

    // Source address offset. Probably to read SRL then SRR.
    dma_local.TCD[3].TCD_SOFF.write(4); 
    //Source data transfer/dest size (32 bit)
    dma_local.TCD[3].TCD_ATTR.write(dma_tcd_attr_ssize(2) | dma_tcd_attr_dsize(2)); 
    //Minor loop byte count (I'm guessing two)
    dma_local.TCD[3].TCD_NBYTES_MLNO.write(dma_tcd_nbytes_mloffyes_nbytes(no_byte_minor_loop) | DMA_TCD_NBYTES_SMLOE | dma_tcd_nbytes_mloffyes_mloff(-8));
    //Add to source address after major iteration
    dma_local.TCD[3].TCD_SLAST.write(2^32 - 8); 
    //Destination offset change per write
    dma_local.TCD[3].TCD_DOFF.write(4);
    //Major iteration count
    dma_local.TCD[3].TCD_CITER_ELINKNO.write( sizeof_spdif_rx_buffer / no_byte_minor_loop as u16); 
    //major iteration destination offset
    let local_negative: u32 = 2^32 - sizeof_spdif_rx_buffer as u32;
    dma_local.TCD[3].TCD_DLASTSGA.write(local_negative);
    //Starting major iteration count (this is used to reset CITER_ELINKNO when it hits zero)
    dma_local.TCD[3].TCD_BITER_ELINKNO.write(sizeof_spdif_rx_buffer);
    //Enable interrupt when major counter is half complete. Enable an interrupt when major counter is complete 
    dma_local.TCD[3].TCD_CSR.write(DMA_TCD_CSR_INTHALF | DMA_TCD_CSR_INTMAJOR);
    //Source address. In this case, left RX FIFO address.
    dma_local.TCD[3].TCD_SADDR.write(spdif_local.SRL.read());
    //Dest addr
    dma_local.TCD[3].TCD_DADDR.write(spdif_rx_buffer_address as u32);

    // Trigger DMA at SPDIF RX
    dmamux_local.CHCFG[3].write(0);
    dmamux_local.CHCFG[3].write((DMAMUX_SOURCE_SPDIF_RX & 0x7F) | DMAMUX_CHCFG_ENBL);

    dma_local.SERQ.write(3); // enable DMA

    unsafe { NVIC::unmask(bsp::Interrupt::DMA3_DMA19) };

    // SPDIF HARDWARE SETUP
    //turn spdif clock on - necessary for receiver!
    ccm_local.CCGR5.write(ccm_local.CCGR5.read() | ccm_ccgr5_spdif(CCM_CCGR_ON));
    //turn receive fifo off 1->off, 0->on
    spdif_local.SCR.write(spdif_local.SCR.read() | SPDIF_SCR_RXFIFO_OFF_ON);
    //reset rx fifo control: normal opertation
    spdif_local.SCR.write(spdif_local.SCR.read() & !(SPDIF_SCR_RXFIFO_CTR));   
    //reset rx full select
    spdif_local.SCR.write(spdif_local.SCR.read() & !(spdif_scr_rxfifofull_sel(3)));
    //full interrupt if at least 8 sample in Rx left and right FIFOs
    spdif_local.SCR.write(spdif_local.SCR.read() | spdif_scr_rxfifofull_sel(2));
    //Rx FIFO auto sync on
    spdif_local.SCR.write(spdif_local.SCR.read() | SPDIF_SCR_RXAUTOSYNC);
    //No embedded U channel
    spdif_local.SCR.write(spdif_local.SCR.read() & !(spdif_scr_usrc_sel(3)));  
    
    //pin 15 set to alt3 -> spdif input
    unsafe { bsp::pins::t40::P15::set_alternate(3) };
    // from eval board sample code
    //   IOMUXC_SetPinConfig(
    //       IOMUXC_GPIO_AD_B1_03_SPDIF_IN,        /* GPIO_AD_B1_03 PAD functional properties : */
    //       0x10B0u);                               /* Slew Rate Field: Slow Slew Rate
    //                                                  Drive Strength Field: R0/6
    //                                                  Speed Field: medium(100MHz)
    //                                                  Open Drain Enable Field: Open Drain Disabled
    //                                                  Pull / Keep Enable Field: Pull/Keeper Enabled
    //                                                  Pull / Keep Select Field: Keeper
    //                                                  Pull Up / Down Config. Field: 100K Ohm Pull Down
    //                                                  Hyst. Enable Field: Hysteresis Disabled */
    const CONFIG: Config = Config::zero()
        .set_slew_rate(SlewRate::Slow)
        .set_drive_strength(DriveStrength::R0_6)
        .set_speed(Speed::Medium)
        .set_open_drain(OpenDrain::Disabled)
        .set_pull_keeper(Some(PullKeeper::Pulldown100k))
        .set_hysteresis(Hysteresis::Disabled);

    let mut pad = unsafe { bsp::pins::imxrt_iomuxc::imxrt1060::gpio_ad_b1::GPIO_AD_B1_03::new() };

    configure(&mut pad, CONFIG);
   
    //receive fifo is turned on again
    spdif_local.SCR.write(spdif_local.SCR.read() & !(SPDIF_SCR_RXFIFO_OFF_ON));
    //SPDIF_SRPC |=SPDIF_SRPC_CLKSRC_SEL(6);        //if (DPLL Locked) SPDIF_RxClk else tx_clk (SPDIF0_CLK_ROOT)
    //page 2129: FrequMeas[23:0]=FreqMeas_CLK / BUS_CLK * 2^10 * GAIN
    spdif_local.SRPC.write(spdif_local.SRPC.read() & !(spdif_srpc_gainsel(7)));    //reset gain select 0 -> gain = 24*2^10
    //SPDIF_SRPC |= SPDIF_SRPC_GAINSEL(3);  //gain select: 8*2^10
    //==============================================

    //interrupts
    //enable spdif receiver lock interrupt
    spdif_local.SIE.write(spdif_local.SIE.read() | SPDIF_SIE_LOCK);
    spdif_local.SIE.write(spdif_local.SIE.read() | SPDIF_SIE_LOCKLOSS);

    nvic_set_priority_spdif(bsp::Interrupt::SPDIF as u32, 208);
    unsafe { 
 //       NVIC::set_priority(&mut self, bsp::Interrupt::SPDIF, 208);
        NVIC::unmask(bsp::Interrupt::SPDIF) };

    //clear SPDIF_SIC_LOCKLOSS interrupt
    spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCKLOSS);
     //clear SPDIF_SIC_LOCK interrupt
    spdif_local.SIC.write(spdif_local.SIC.read() | SPDIF_SIC_LOCK);

    spdif_local.SRCD.write(0);
    //DMA Receive Request Enable    
    spdif_local.SCR.write(spdif_local.SCR.read() | SPDIF_SCR_DMA_RX_EN);

}    

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

const SPDIF_SRPC_LOCK: u32 = 1 << 6;
static mut F_BUS_ACTUAL: f32 = 132000000.0;
fn get_sample_rate() -> f32 {
    let spdif_local = unsafe{ spdif::SPDIF::instance()};
    
    if spdif_local.SRPC.read() & SPDIF_SRPC_LOCK != 0 { // Lock acquired
        let freq_meas = spdif_local.SRFM.read() & 0xFFFFFF;
        let f = unsafe { F_BUS_ACTUAL } / (1024. * 1024. * 24. * 128.); // bit clock = 128 * sampling frequency
        let freq_meas_clk = freq_meas as f32 * f;
        freq_meas_clk
    } else {
        -1.0
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
        let sr = get_sample_rate();
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
                    let copied = copy_from_spdif_buffer(128, &mut self.lbuf, &mut self.rbuf);
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

