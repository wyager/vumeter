#!/bin/bash
arm-none-eabi-objcopy -O ihex -R .eeprom ./target/thumbv7em-none-eabihf/release/main ./prog.hex
./teensy_loader_cli.exe --mcu=TEENSY40 -w -v ./prog.hex
#C:/_GitProjects/teensy_loader_cli/teensy_loader_cli.exe --mcu=imxrt1062 -w -v ./prog.hex
#.\teensy.exe --mcu=imxrt1062 -w -v ./audio_board/prog.hex