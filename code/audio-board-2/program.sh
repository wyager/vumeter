#!/bin/bash
cargo objcopy --release -- -O ihex audio_board_h.hex
#or do this
#arm-none-eabi-objcopy -O ihex -R .eeprom ./target/thumbv7em-none-eabihf/release/audio_board_h ./audio_board_h.hex
#you need to delete the first section from hex file
teensy_loader_cli.exe --mcu=TEENSY40 -w -v audio_board_h.hex
