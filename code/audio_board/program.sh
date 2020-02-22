#!/bin/bash
arm-none-eabi-objcopy -O ihex -R .eeprom ./target/thumbv7em-none-eabihf/release/main /tmp/prog.hex
teensy_loader_cli --mcu=TEENSY40 -w -v /tmp/prog.hex
