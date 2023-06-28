#!/bin/bash
cargo objcopy --release -- -O ihex audio-board.hex
teensy_loader_cli --mcu=TEENSY40 -w -v audio-board.hex
