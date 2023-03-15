# Nixie Tube Audio Meter

![Picture of VU meter](http://yager.io/vumeter/Main.jpeg)

A full writeup is available [here](http://yager.io/vumeter/vu.html).

## Directory Structure
* pdfs - some relevant datasheets
* schematics
    * tube-board - the board that powers and controls the nixie tubes
    * tube-offset-1,2 - PCBs that hold the tubes in place
    * audio-board - a teensy-based board that takes in S/PDIF audio over TOSLINK, does DSP, and outputs control data to the tube boards
    * custom - custom kicad parts for this project
    * case - freecad designs and macros for the case, plus some resultant SVGs I used to laser-cut the acrylic

