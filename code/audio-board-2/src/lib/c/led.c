
#include "stdint.h"
#include "core_pins.h"
#include "stdbool.h"
#include "imxrt.h"
#include "unistd.h"


void set_led_en(bool on) {
    digitalWrite(13,on);
}

void led_init() {
    pinMode(13, 1);
}

