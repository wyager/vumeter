
#include "stdint.h"
#include "core_pins.h"
#include "stdbool.h"
#include "imxrt.h"
#include "unistd.h"

uint8_t switch_pins() {
    uint8_t result = 0;
    for(int i = 0; i < 4; i++) {
        result |= !digitalRead(9+i) << i;
    }
    return result;
}

void switch_init() {
    for(int i = 0; i < 4; i++) {
        pinMode(9+i, INPUT_PULLUP);
    }
}


void set_pwr_en(bool on) {
    digitalWrite(6,on);
}

void pwr_init() {
    // For some reason, OUTPUT mode doesn't seem to be working.
    // This is close enough for now, LOL
    pinMode(6, INPUT_PULLUP);
}

void set_sleep_pin(bool on) {
    digitalWriteFast(7,on);
}

void sleep_pin_init() {
    pinMode(7, OUTPUT);
}