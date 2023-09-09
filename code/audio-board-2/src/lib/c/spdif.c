
#include "stdint.h"
#include "core_pins.h"
#include "stdbool.h"
#include "imxrt.h"
#include "avr/pgmspace.h"

typedef struct __attribute__((packed, aligned(4))) {
        volatile const void * volatile SADDR;
        int16_t SOFF;
        union { uint16_t ATTR;
            struct { uint8_t ATTR_DST; uint8_t ATTR_SRC; }; };
        union { uint32_t NBYTES; uint32_t NBYTES_MLNO;
            uint32_t NBYTES_MLOFFNO; uint32_t NBYTES_MLOFFYES; };
        int32_t SLAST;
        volatile void * volatile DADDR;
        int16_t DOFF;
        union { volatile uint16_t CITER;
            volatile uint16_t CITER_ELINKYES; volatile uint16_t CITER_ELINKNO; };
        int32_t DLASTSGA;
        volatile uint16_t CSR;
        union { volatile uint16_t BITER;
            volatile uint16_t BITER_ELINKYES; volatile uint16_t BITER_ELINKNO; };
} TCD_t;


volatile uint32_t F_BUS_ACTUAL = 132000000;
#define SPDIF_RX_BUFFER_LENGTH 128
#define BUFFERLEN 1536

// Written to by DMA, read by ISR
static volatile __attribute__((section(".dmabuffers"), used)) int32_t spdif_rx_buffer[SPDIF_RX_BUFFER_LENGTH] = {};
// Written to by ISR, read by "userspace" code
static volatile float bufferR[BUFFERLEN] = {};
static volatile float bufferL[BUFFERLEN] = {};
volatile uint32_t buffer_offset = 0; // Write to write next
volatile uint32_t buffer_read_offset = 0; // Where to read next




uint32_t copy_from_spdif_buffer(uint32_t max, float* restrict out_l, float* restrict out_r) {
    __disable_irq();
    uint32_t b_o = buffer_offset;
    uint32_t b_r_o = buffer_read_offset;
    __enable_irq();
    uint32_t remaining = (b_r_o <= b_o) ? (b_o - b_r_o) : (BUFFERLEN - b_r_o + b_o);
    uint32_t to_copy = remaining < max ? remaining : max;
    for(uint32_t i = 0; i < to_copy; i++){
        out_l[i] = bufferL[b_r_o];
        out_r[i] = bufferR[b_r_o];
        b_r_o += 1;
        if(b_r_o == BUFFERLEN) {b_r_o = 0;}
    }
    __disable_irq();
    buffer_read_offset = b_r_o;
    __enable_irq();
    return to_copy;

}

float get_sample_rate() {
    if(SPDIF_SRPC & SPDIF_SRPC_LOCK) { // Lock aqcuired
        const uint32_t freqMeas=(SPDIF_SRFM & 0xFFFFFF);
        const double f=(float)F_BUS_ACTUAL/(1024.*1024.*24.*128.); // bit clock = 128 * sampling frequency
        const double freqMeas_CLK= (float)(freqMeas)*f;
        return freqMeas_CLK;
    } else {
        return -1.;
    }
}

void spdif_isr(){
    SPDIF_SIC |= SPDIF_SIC_LOCKLOSS;//clear SPDIF_SIC_LOCKLOSS interrupt
    SPDIF_SIC |= SPDIF_SIC_LOCK;    //clear SPDIF_SIC_LOCK interrupt
}

const int channel = 3; // Careful changing. Search for CH3
TCD_t* TCD = 0;

void spdif_dma_isr() {
    
    uint32_t daddr = (uint32_t)(TCD->DADDR); // Where is it writing now?
    DMA_CINT = channel; // Clear interrupts

    uint32_t* src;
    if (daddr < (uint32_t)spdif_rx_buffer + sizeof(spdif_rx_buffer) / 2) {
        // DMA is filling the first half of the buffer
        // need to remove data from the second half
        src = (uint32_t *)&spdif_rx_buffer[SPDIF_RX_BUFFER_LENGTH/2];
    } else {
        src = (uint32_t *)&spdif_rx_buffer[0];
    }
        


    // Ensure we have room to write to the buffer without clobbering old samples
    if(buffer_offset >= buffer_read_offset || (buffer_offset + SPDIF_RX_BUFFER_LENGTH/4) < buffer_read_offset) {
        // digitalWriteFast(13,1);
        // We have to blow out the cache or else we just get the same few samples over and over again.
        arm_dcache_delete(src, SPDIF_RX_BUFFER_LENGTH/2*sizeof(uint32_t));
        for(uint32_t i = 0; i < SPDIF_RX_BUFFER_LENGTH/4; i++) {

            float max = 32768.;

            // There's a bit of a weird sign conversion thing going on here;
            // we have a *signed* 24-bit value in the *lower* 24 bits of the 
            // 32-bit locus. One solution is to just toss the bottom 8 bits 
            // and then interpret this as a signed 16-bit int.
            uint32_t nL = src[(2*i) + 0];
            int16_t iL = (int16_t) (nL >> 8);
            bufferL[buffer_offset+i] = (float)iL / max;

            uint32_t nR = src[(2*i) + 1];
            int16_t iR = (int16_t) (nR >> 8);
            bufferR[buffer_offset+i] = (float)iR / max;
        }
        buffer_offset = (buffer_offset + SPDIF_RX_BUFFER_LENGTH/4) % BUFFERLEN;
    } else {
        // digitalWriteFast(13,0);
        // overflow
    }


}

// This function is copied piece-by-piece from a bunch of different places.
// Probably some extraneous stuff in here. 
void config_spdif() {
    /// DMA SETUP ///

    // Begin DMA operation
    CCM_CCGR5 |= CCM_CCGR5_DMA(CCM_CCGR_ON);    
    DMA_CR = DMA_CR_GRP1PRI | DMA_CR_EMLM | DMA_CR_EDBG;
    DMA_CERQ = channel;
    DMA_CERR = channel;
    DMA_CEEI = channel;
    DMA_CINT = channel;
    
    TCD = (TCD_t *)(0x400E9000 + channel * 32);
    uint32_t *p = (uint32_t *)TCD;
    *p++ = 0; *p++ = 0; *p++ = 0; *p++ = 0; *p++ = 0; *p++ = 0; *p++ = 0; *p++ = 0;

    const uint32_t noByteMinorLoop=2*4; 
    TCD->SOFF = 4;  // Source address offset. Probably to read SRL then SRR. 
    TCD->ATTR = DMA_TCD_ATTR_SSIZE(2) | DMA_TCD_ATTR_DSIZE(2);  //Source data transfer/dest size (32 bit)
    TCD->NBYTES_MLNO = DMA_TCD_NBYTES_MLOFFYES_NBYTES(noByteMinorLoop) | DMA_TCD_NBYTES_SMLOE | DMA_TCD_NBYTES_MLOFFYES_MLOFF(-8);   //Minor loop byte count (I'm guessing two)
    TCD->SLAST = -8;    //Add to source address after major iteration
    TCD->DOFF = 4;  //Destination offset change per write
    TCD->CITER_ELINKNO = sizeof(spdif_rx_buffer) / noByteMinorLoop; //Major iteration count
    TCD->DLASTSGA = -sizeof(spdif_rx_buffer);   //major iteration destination offset
    TCD->BITER_ELINKNO = sizeof(spdif_rx_buffer) / noByteMinorLoop; //Starting major iteration count (this is used to reset CITER_ELINKNO when it hits zero)
    TCD->CSR = DMA_TCD_CSR_INTHALF | DMA_TCD_CSR_INTMAJOR;  //Enable interrupt when major counter is half complete. Enable an interrupt when major counter is complete 
    TCD->SADDR = (void *)((uint32_t)&SPDIF_SRL);    //Source address. In this case, left RX FIFO address.
    TCD->DADDR = spdif_rx_buffer;   //Dest addr
    
    // Trigger DMA at SPDIF RX
    volatile uint32_t *mux = &DMAMUX_CHCFG0 + channel;
    *mux = 0;
    *mux = (DMAMUX_SOURCE_SPDIF_RX & 0x7F) | DMAMUX_CHCFG_ENBL;
        
    DMA_SERQ = channel; // enable DMA

    NVIC_ENABLE_IRQ(IRQ_DMA_CH3);

    /// SPDIF HARDWARE SETUP ///

    CCM_CCGR5 |=CCM_CCGR5_SPDIF(CCM_CCGR_ON); //turn spdif clock on - necessary for receiver!
    
    SPDIF_SCR |=SPDIF_SCR_RXFIFO_OFF_ON;    //turn receive fifo off 1->off, 0->on

    SPDIF_SCR&=~(SPDIF_SCR_RXFIFO_CTR);     //reset rx fifo control: normal opertation

    SPDIF_SCR&=~(SPDIF_SCR_RXFIFOFULL_SEL(3));  //reset rx full select
    SPDIF_SCR|=SPDIF_SCR_RXFIFOFULL_SEL(2); //full interrupt if at least 8 sample in Rx left and right FIFOs

    SPDIF_SCR|=SPDIF_SCR_RXAUTOSYNC; //Rx FIFO auto sync on

    SPDIF_SCR&=(~SPDIF_SCR_USRC_SEL(3));    //No embedded U channel

    CORE_PIN15_CONFIG  = 3;  //pin 15 set to alt3 -> spdif input
    /// from eval board sample code
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
    CORE_PIN15_PADCONFIG=0x10B0;
    SPDIF_SCR &=(~SPDIF_SCR_RXFIFO_OFF_ON); //receive fifo is turned on again


    SPDIF_SRPC &= ~SPDIF_SRPC_CLKSRC_SEL(15);   //reset clock selection page 2136
    //SPDIF_SRPC |=SPDIF_SRPC_CLKSRC_SEL(6);        //if (DPLL Locked) SPDIF_RxClk else tx_clk (SPDIF0_CLK_ROOT)
    //page 2129: FrequMeas[23:0]=FreqMeas_CLK / BUS_CLK * 2^10 * GAIN
    SPDIF_SRPC &=~SPDIF_SRPC_GAINSEL(7);    //reset gain select 0 -> gain = 24*2^10
    //SPDIF_SRPC |= SPDIF_SRPC_GAINSEL(3);  //gain select: 8*2^10
    //==============================================

    //interrupts
    SPDIF_SIE |= SPDIF_SIE_LOCK;    //enable spdif receiver lock interrupt
    SPDIF_SIE |=SPDIF_SIE_LOCKLOSS;

    NVIC_SET_PRIORITY(IRQ_SPDIF, 208); // 255 = lowest priority, 208 = priority of update
    NVIC_ENABLE_IRQ(IRQ_SPDIF);

    spdif_isr();

    SPDIF_SRCD = 0;
    SPDIF_SCR |= SPDIF_SCR_DMA_RX_EN;        //DMA Receive Request Enable    
}

