
#include "stdint.h"
#include "core_pins.h"
#include "stdbool.h"
#include "imxrt.h"
#include "unistd.h"

#ifndef SERIAL4_TX_BUFFER_SIZE
#define SERIAL4_TX_BUFFER_SIZE     40 // number of outgoing bytes to buffer
#endif
#ifndef SERIAL4_RX_BUFFER_SIZE
#define SERIAL4_RX_BUFFER_SIZE     64 // number of incoming bytes to buffer
#endif
#define IRQ_PRIORITY  64  // 0 = highest priority, 255 = lowest

// From teensy HWserial cpp file
#define PIN_TO_BASEREG(pin)             (portOutputRegister(pin))
#define PIN_TO_BITMASK(pin)             (digitalPinToBitMask(pin))
#define IO_REG_TYPE uint32_t
#define IO_REG_BASE_ATTR
#define IO_REG_MASK_ATTR
#define DIRECT_READ(base, mask)         ((*((base)+2) & (mask)) ? 1 : 0)
#define DIRECT_MODE_INPUT(base, mask)   (*((base)+1) &= ~(mask))
#define DIRECT_MODE_OUTPUT(base, mask)  (*((base)+1) |= (mask))
#define DIRECT_WRITE_LOW(base, mask)    (*((base)+34) = (mask))
#define DIRECT_WRITE_HIGH(base, mask)   (*((base)+33) = (mask))
#define UART_CLOCK 24000000
#define CTRL_ENABLE         (LPUART_CTRL_TE | LPUART_CTRL_RE | LPUART_CTRL_RIE | LPUART_CTRL_ILIE)
#define CTRL_TX_ACTIVE      (CTRL_ENABLE | LPUART_CTRL_TIE)
#define CTRL_TX_COMPLETING  (CTRL_ENABLE | LPUART_CTRL_TCIE)
#define CTRL_TX_INACTIVE    CTRL_ENABLE 

#define SERIAL_8N1 0x00
#define SERIAL_8N1_RXINV_TXINV 0x30

int nvic_execution_priority(void)
{
    uint32_t priority=256;
    uint32_t primask, faultmask, basepri, ipsr;

    // full algorithm in ARM DDI0403D, page B1-639
    // this isn't quite complete, but hopefully good enough
    __asm__ volatile("mrs %0, faultmask\n" : "=r" (faultmask)::);
    if (faultmask) return -1;
    __asm__ volatile("mrs %0, primask\n" : "=r" (primask)::);
    if (primask) return 0;
    __asm__ volatile("mrs %0, ipsr\n" : "=r" (ipsr)::);
    if (ipsr) {
        if (ipsr < 16) priority = 0; // could be non-zero
        else priority = NVIC_GET_PRIORITY(ipsr - 16);
    }
    __asm__ volatile("mrs %0, basepri\n" : "=r" (basepri)::);
    if (basepri > 0 && basepri < priority) priority = basepri;
    return priority;
}

static uint8_t tx_buffer4[SERIAL4_TX_BUFFER_SIZE];
static uint8_t rx_buffer4[SERIAL4_RX_BUFFER_SIZE];

static ssize_t rx_buffer_head_ = 0;
static ssize_t rx_buffer_tail_ = 0;
static ssize_t tx_buffer_head_ = 0;
static ssize_t tx_buffer_tail_ = 0;
volatile uint8_t    transmitting_ = 0;
#define LO_WATERMARK 38
#define HI_WATERMARK 24
const ssize_t rts_low_watermark_  = SERIAL4_RX_BUFFER_SIZE - LO_WATERMARK;
const ssize_t rts_high_watermark_ = SERIAL4_RX_BUFFER_SIZE - HI_WATERMARK;

// static HardwareSerial::hardware_t UART3_Hardware = {
//     serial_index: 3
//     irq number: IRQ_LPUART3
//     irq handler: &IRQHandler_Serial4
//     serial event handler check: &serial_event_check_serial4

//     ccm register: CCM_CCGR0
//     ccm value: CCM_CCGR0_LPUART3(CCM_CCGR_ON)

    // rx pins: {{16, // Pin number
    //             2, // mux val
    //             &IOMUXC_LPUART3_RX_SELECT_INPUT,  // "Which register controls selection"
    //             0}, // "Value for that selection"
    //           {0xff, 0xff, nullptr, 0}},
//     tx pins: {{17,2, nullptr, 0}, {0xff, 0xff, nullptr, 0}},
//     cts pin: 0xff, // No CTS pin
//     cts mux: 0, // No CTS
//     IRQ_PRIORITY, 
//     rts low watermark: 38, 
//     rts high watermark: 24,
// };
// HardwareSerial Serial4(
//     port: &IMXRT_LPUART3,
//     hw: &UART3_Hardware, 
//     tx_buffer4, SERIAL4_TX_BUFFER_SIZE,
//     rx_buffer4,  SERIAL4_RX_BUFFER_SIZE);

#define RX_PIN_NO 16
#define RX_MUX_VAL 2
#define TX_PIN_NO 17
#define TX_MUX_VAL 2


bool uart4_is_transmitting() {
    return transmitting_;
}

void uart4_init(uint32_t baud) {
    float base = (float)UART_CLOCK / (float)baud;
    float besterr = 1e20;
    int bestdiv = 1;
    int bestosr = 4;
    for (int osr=4; osr <= 32; osr++) {
        float div = base / (float)osr;
        int divint = (int)(div + 0.5f);
        if (divint < 1) divint = 1;
        else if (divint > 8191) divint = 8191;
        float err = ((float)divint - div) / div;
        if (err < 0.0f) err = -err;
        if (err <= besterr) {
            besterr = err;
            bestdiv = divint;
            bestosr = osr;
        }
    }
    //printf(" baud %d: osr=%d, div=%d\n", baud, bestosr, bestdiv);
    rx_buffer_head_ = 0;
    rx_buffer_tail_ = 0;
    tx_buffer_head_ = 0;
    tx_buffer_tail_ = 0;

    transmitting_ = 0;

    CCM_CCGR0 |= CCM_CCGR0_LPUART3(CCM_CCGR_ON);

 //   uint32_t fastio = IOMUXC_PAD_SRE | IOMUXC_PAD_DSE(3) | IOMUXC_PAD_SPEED(3);

    *(portControlRegister(RX_PIN_NO)) = IOMUXC_PAD_DSE(7) | IOMUXC_PAD_PKE | IOMUXC_PAD_PUE | IOMUXC_PAD_PUS(3) | IOMUXC_PAD_HYS;
    *(portConfigRegister(RX_PIN_NO)) = RX_MUX_VAL;

    IOMUXC_LPUART3_RX_SELECT_INPUT = 0;

    *(portControlRegister(TX_PIN_NO)) =  IOMUXC_PAD_SRE | IOMUXC_PAD_DSE(3) | IOMUXC_PAD_SPEED(3);
    *(portConfigRegister(TX_PIN_NO)) = TX_MUX_VAL;

    IMXRT_LPUART3.BAUD = LPUART_BAUD_OSR(bestosr - 1) | LPUART_BAUD_SBR(bestdiv)  | (bestosr <= 8 ? LPUART_BAUD_BOTHEDGE : 0);
    IMXRT_LPUART3.PINCFG = 0;


    NVIC_SET_PRIORITY(IRQ_LPUART3, IRQ_PRIORITY);  
    NVIC_ENABLE_IRQ(IRQ_LPUART3);
    uint16_t tx_fifo_size = (((IMXRT_LPUART3.FIFO >> 4) & 0x7) << 2);
    uint8_t tx_water = (tx_fifo_size < 16) ? tx_fifo_size >> 1 : 7;
    uint16_t rx_fifo_size = (((IMXRT_LPUART3.FIFO >> 0) & 0x7) << 2);
    uint8_t rx_water = (rx_fifo_size < 16) ? rx_fifo_size >> 1 : 7;
    IMXRT_LPUART3.WATER = LPUART_WATER_RXWATER(rx_water) | LPUART_WATER_TXWATER(tx_water);
    IMXRT_LPUART3.FIFO |= LPUART_FIFO_TXFE | LPUART_FIFO_RXFE;


    // lets configure up our CTRL register value
    uint32_t ctrl = CTRL_TX_INACTIVE;

    uint16_t format = SERIAL_8N1;
    // Now process the bits in the Format value passed in
    // Bits 0-2 - Parity plus 9  bit. 
    ctrl |= (format & (LPUART_CTRL_PT | LPUART_CTRL_PE) );  // configure parity - turn off PT, PE, M and configure PT, PE
    if (format & 0x04) ctrl |= LPUART_CTRL_M;       // 9 bits (might include parity)
    if ((format & 0x0F) == 0x04) ctrl |=  LPUART_CTRL_R9T8; // 8N2 is 9 bit with 9th bit always 1

    // Bit 5 TXINVERT
    if (format & 0x20) ctrl |= LPUART_CTRL_TXINV;       // tx invert

    // write out computed CTRL
    IMXRT_LPUART3.CTRL = ctrl;

//     // Bit 3 10 bit - Will assume that begin already cleared it.
//     // process some other bits which change other registers.
    if (format & 0x08)  IMXRT_LPUART3.BAUD |= LPUART_BAUD_M10;

    // Bit 4 RXINVERT 
    uint32_t c = IMXRT_LPUART3.STAT & ~LPUART_STAT_RXINV;
    if (format & 0x10) c |= LPUART_STAT_RXINV;      // rx invert
    IMXRT_LPUART3.STAT = c;

    // bit 8 can turn on 2 stop bit mote
    if ( format & 0x100) IMXRT_LPUART3.BAUD |= LPUART_BAUD_SBNS;    

//     enableSerialEvents();       // Enable the processing of serialEvent for this object
}

int uart4_availableForWrite(void)
{
    uint32_t head, tail;

    head = tx_buffer_head_;
    tail = tx_buffer_tail_;
    if (head >= tail) return SERIAL4_TX_BUFFER_SIZE - 1 - head + tail;
    return tail - head - 1;
}

int uart4_available(void)
{
    uint32_t head, tail;

    head = rx_buffer_head_;
    tail = rx_buffer_tail_;
    if (head >= tail) return head - tail;
    return SERIAL4_RX_BUFFER_SIZE + head - tail;
}

int16_t uart4_peek(void)
{
    uint32_t head, tail;

    head = rx_buffer_head_;
    tail = rx_buffer_tail_;
    if (head == tail) return -1;
    if (++tail >= SERIAL4_RX_BUFFER_SIZE) tail = 0;
    return rx_buffer4[tail];
}

int16_t uart4_read(void)
{
    uint32_t head, tail;
    int16_t c;

    head = rx_buffer_head_;
    tail = rx_buffer_tail_;
    if (head == tail) return -1;
    if (++tail >= SERIAL4_RX_BUFFER_SIZE) tail = 0;
    c = rx_buffer4[tail];
    rx_buffer_tail_ = tail;
    return c;
}  

size_t uart4_write(uint8_t c)
{
    uint32_t head, n;
    head = tx_buffer_head_;
    if (++head >= SERIAL4_TX_BUFFER_SIZE) head = 0;
    while ((uint32_t)tx_buffer_tail_ == head) {
        int priority = nvic_execution_priority();
        if (priority <= IRQ_PRIORITY) {
            if ((IMXRT_LPUART3.STAT & LPUART_STAT_TDRE)) {
                uint32_t tail = tx_buffer_tail_;
                if (++tail >= SERIAL4_TX_BUFFER_SIZE) tail = 0;
                n = tx_buffer4[tail];
                IMXRT_LPUART3.DATA  = n;
                tx_buffer_tail_ = tail;
            }
        } else if (priority >= 256) 
        {
            return 0;
        } 
    }

    tx_buffer4[head] = c;
    __disable_irq();
    transmitting_ = 1;
    tx_buffer_head_ = head;
    IMXRT_LPUART3.CTRL |= LPUART_CTRL_TIE; // (may need to handle this issue)BITBAND_SET_BIT(LPUART0_CTRL, TIE_BIT);
    __enable_irq();
    return 1;
}

void uart_isr() {
    uint32_t head, tail, n;
    uint32_t ctrl;

    // See if we have stuff to read in.
    // Todo - Check idle. 
    if (IMXRT_LPUART3.STAT & (LPUART_STAT_RDRF | LPUART_STAT_IDLE)) {
        // See how many bytes or pending. 
        //digitalWrite(5, HIGH);
        uint8_t avail = (IMXRT_LPUART3.WATER >> 24) & 0x7;
        if (avail) {
            uint32_t newhead;
            head = rx_buffer_head_;
            tail = rx_buffer_tail_;
            do {
                n = IMXRT_LPUART3.DATA & 0x3ff;     // Use only up to 10 bits of data
                newhead = head + 1;

                if (newhead >= SERIAL4_RX_BUFFER_SIZE) newhead = 0;
                if (newhead != (uint32_t)rx_buffer_tail_) {
                    head = newhead;
                    rx_buffer4[head] = n;
                }
            } while (--avail > 0) ;
            rx_buffer_head_ = head;
        }

        // If it was an idle status clear the idle
        if (IMXRT_LPUART3.STAT & LPUART_STAT_IDLE) {
            IMXRT_LPUART3.STAT |= LPUART_STAT_IDLE; // writing a 1 to idle should clear it. 
        }

    }

    // See if we are transmitting and room in buffer. 
    ctrl = IMXRT_LPUART3.CTRL;
    if ((ctrl & LPUART_CTRL_TIE) && (IMXRT_LPUART3.STAT & LPUART_STAT_TDRE))
    {
        //digitalWrite(3, HIGH);

        head = tx_buffer_head_;
        tail = tx_buffer_tail_;
        do {
            if (head == tail) break;
            if (++tail >= SERIAL4_TX_BUFFER_SIZE) tail = 0;    
            n = tx_buffer4[tail];
            IMXRT_LPUART3.DATA = n;
        } while (((IMXRT_LPUART3.WATER >> 8) & 0x7) < 4);   // need to computer properly
        tx_buffer_tail_ = tail;
        if (head == tail) {
            IMXRT_LPUART3.CTRL &= ~LPUART_CTRL_TIE; 
            IMXRT_LPUART3.CTRL |= LPUART_CTRL_TCIE; // Actually wondering if we can just leave this one on...
        }
        //digitalWrite(3, LOW);
    }

    if ((ctrl & LPUART_CTRL_TCIE) && (IMXRT_LPUART3.STAT & LPUART_STAT_TC))
    {
        transmitting_ = 0;
        IMXRT_LPUART3.CTRL &= ~LPUART_CTRL_TCIE;
    }
}