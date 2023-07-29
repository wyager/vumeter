
#include "stdint.h"
#include "core_pins.h"
#include "stdbool.h"
#include "imxrt.h"
#include "unistd.h"

IMXRT_PIT_CHANNEL_t *pit_channel = IMXRT_PIT_CHANNELS;

volatile uint64_t pit_count = 0;

void pit_isr() {
    if (pit_channel->TFLG){
        pit_count++;
        pit_channel->TFLG = 1;
    }
}


bool timer_init(uint32_t cycles) {
    CCM_CCGR1 |= CCM_CCGR1_PIT(CCM_CCGR_ON);
    PIT_MCR = 1;
    while (1) {
        if (pit_channel->TCTRL == 0) break;
        if (++pit_channel >= IMXRT_PIT_CHANNELS + 4) {
            pit_channel = NULL;
            return false;
        }
    }
    pit_channel->LDVAL = cycles;
    pit_channel->TCTRL = 3;
    NVIC_SET_PRIORITY(IRQ_PIT,255);
    NVIC_ENABLE_IRQ(IRQ_PIT);
    return true;
}

uint64_t timer_count() {
    return pit_count;
}