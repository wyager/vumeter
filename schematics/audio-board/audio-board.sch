EESchema Schematic File Version 4
EELAYER 30 0
EELAYER END
$Descr A4 11693 8268
encoding utf-8
Sheet 1 1
Title ""
Date ""
Rev ""
Comp ""
Comment1 ""
Comment2 ""
Comment3 ""
Comment4 ""
$EndDescr
Text GLabel 6450 6950 1    50   Input ~ 0
V_mid_out
Text GLabel 6250 6950 1    50   Input ~ 0
tx2
Text GLabel 6350 6950 1    50   Input ~ 0
rx2
$Comp
L Mechanical:MountingHole_Pad H1
U 1 1 5E18D6B5
P 9150 6150
F 0 "H1" H 9250 6199 50  0000 L CNN
F 1 "MountingHole_Pad" H 9250 6108 50  0000 L CNN
F 2 "MountingHole:MountingHole_2.7mm_M2.5_ISO14580_Pad" H 9150 6150 50  0001 C CNN
F 3 "~" H 9150 6150 50  0001 C CNN
	1    9150 6150
	1    0    0    -1  
$EndComp
$Comp
L Mechanical:MountingHole_Pad H2
U 1 1 5E19C90D
P 9450 6150
F 0 "H2" H 9550 6199 50  0000 L CNN
F 1 "MountingHole_Pad" H 9550 6108 50  0000 L CNN
F 2 "MountingHole:MountingHole_2.7mm_M2.5_ISO14580_Pad" H 9450 6150 50  0001 C CNN
F 3 "~" H 9450 6150 50  0001 C CNN
	1    9450 6150
	1    0    0    -1  
$EndComp
$Comp
L Mechanical:MountingHole_Pad H3
U 1 1 5E19CEDD
P 9750 6150
F 0 "H3" H 9850 6199 50  0000 L CNN
F 1 "MountingHole_Pad" H 9850 6108 50  0000 L CNN
F 2 "MountingHole:MountingHole_2.7mm_M2.5_ISO14580_Pad" H 9750 6150 50  0001 C CNN
F 3 "~" H 9750 6150 50  0001 C CNN
	1    9750 6150
	1    0    0    -1  
$EndComp
$Comp
L Mechanical:MountingHole_Pad H4
U 1 1 5E19D344
P 10050 6150
F 0 "H4" H 10150 6199 50  0000 L CNN
F 1 "MountingHole_Pad" H 10150 6108 50  0000 L CNN
F 2 "MountingHole:MountingHole_2.7mm_M2.5_ISO14580_Pad" H 10050 6150 50  0001 C CNN
F 3 "~" H 10050 6150 50  0001 C CNN
	1    10050 6150
	1    0    0    -1  
$EndComp
NoConn ~ 3450 6200
NoConn ~ 9150 6250
NoConn ~ 9450 6250
NoConn ~ 9750 6250
NoConn ~ 10050 6250
$Comp
L teensy:Teensy4.0 U1
U 1 1 5DFFBFE0
P 5200 4800
F 0 "U1" H 5200 6415 50  0000 C CNN
F 1 "Teensy4.0" H 5200 6324 50  0000 C CNN
F 2 "vumeter:Teensy40" H 4800 5000 50  0001 C CNN
F 3 "" H 4800 5000 50  0001 C CNN
	1    5200 4800
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR0103
U 1 1 5DFFF5F8
P 6300 5850
F 0 "#PWR0103" H 6300 5600 50  0001 C CNN
F 1 "GND" H 6305 5677 50  0000 C CNN
F 2 "" H 6300 5850 50  0001 C CNN
F 3 "" H 6300 5850 50  0001 C CNN
	1    6300 5850
	0    -1   1    0   
$EndComp
Text GLabel 6300 3950 2    50   Input ~ 0
3V3
Text GLabel 6300 5950 2    50   Input ~ 0
3V3
$Comp
L vumeter:spdif_recv U2
U 1 1 5E00CF21
P 3700 5850
F 0 "U2" H 3917 5335 50  0000 C CNN
F 1 "spdif_recv" H 3917 5426 50  0000 C CNN
F 2 "vumeter:Cliff_ORJ-8" H 3700 5850 50  0001 C CNN
F 3 "ORJ-8" H 3700 5850 50  0001 C CNN
	1    3700 5850
	-1   0    0    1   
$EndComp
Text GLabel 3700 5650 2    50   Input ~ 0
3V3
$Comp
L power:GND #PWR0105
U 1 1 5E00F196
P 3700 5750
F 0 "#PWR0105" H 3700 5500 50  0001 C CNN
F 1 "GND" H 3705 5577 50  0000 C CNN
F 2 "" H 3700 5750 50  0001 C CNN
F 3 "" H 3700 5750 50  0001 C CNN
	1    3700 5750
	0    -1   1    0   
$EndComp
Text GLabel 6300 6050 2    50   Input ~ 0
master_clk
Text GLabel 4100 6150 0    50   Input ~ 0
audio_serial_clock
Text GLabel 4100 6050 0    50   Input ~ 0
lrclk
Text GLabel 4100 4350 0    50   Input ~ 0
audio_serial
Text GLabel 4100 5750 0    50   Input ~ 0
tx2
Text GLabel 4100 5650 0    50   Input ~ 0
rx2
Text GLabel 6300 6150 2    50   Input ~ 0
adc_powerdown
Text GLabel 4100 3950 0    50   Input ~ 0
15db_gain_en
Text GLabel 4100 4050 0    50   Input ~ 0
filter_select
Text GLabel 4100 3850 0    50   Input ~ 0
audio_format_i2s_hi
Text GLabel 4100 3750 0    50   Input ~ 0
mode_select
Wire Wire Line
	3700 5550 4100 5550
$Comp
L vumeter:S-2151 U5
U 1 1 5E009D8F
P 3500 4950
F 0 "U5" H 3692 4135 50  0000 C CNN
F 1 "S-2151" H 3692 4226 50  0000 C CNN
F 2 "vumeter:S-2151" H 3500 4950 50  0001 C CNN
F 3 "S-2151" H 3500 4950 50  0001 C CNN
	1    3500 4950
	-1   0    0    1   
$EndComp
$Comp
L power:GND #PWR0104
U 1 1 5E017574
P 3050 4250
F 0 "#PWR0104" H 3050 4000 50  0001 C CNN
F 1 "GND" H 3055 4077 50  0000 C CNN
F 2 "" H 3050 4250 50  0001 C CNN
F 3 "" H 3050 4250 50  0001 C CNN
	1    3050 4250
	0    1    -1   0   
$EndComp
NoConn ~ 4100 4850
NoConn ~ 4100 4950
NoConn ~ 4100 5050
NoConn ~ 4100 5150
NoConn ~ 4100 5250
NoConn ~ 4100 5350
NoConn ~ 4100 5450
NoConn ~ 4100 5850
NoConn ~ 4100 5950
NoConn ~ 6300 5350
NoConn ~ 6300 5250
NoConn ~ 6300 5150
NoConn ~ 6300 5050
NoConn ~ 6300 4950
NoConn ~ 6300 4850
NoConn ~ 6300 4750
NoConn ~ 6300 4650
NoConn ~ 6300 4550
NoConn ~ 6300 4450
NoConn ~ 6300 4350
NoConn ~ 6300 4250
NoConn ~ 6300 4050
NoConn ~ 6300 3850
NoConn ~ 6300 3750
NoConn ~ 6300 3650
NoConn ~ 6300 3550
NoConn ~ 6300 3450
NoConn ~ 6300 5650
NoConn ~ 4100 4250
$Comp
L vumeter:VN7140 U6
U 1 1 5E0EDEAC
P 2850 6800
F 0 "U6" H 2850 6825 50  0000 C CNN
F 1 "VN7140" H 2850 6734 50  0000 C CNN
F 2 "Package_SO:SO-8_3.9x4.9mm_P1.27mm" H 2850 6800 50  0001 C CNN
F 3 "" H 2850 6800 50  0001 C CNN
	1    2850 6800
	1    0    0    -1  
$EndComp
Wire Wire Line
	3300 7100 3300 7200
Text GLabel 3400 7200 2    50   Input ~ 0
V_mid_out
Wire Wire Line
	3400 7200 3300 7200
Connection ~ 3300 7200
Wire Wire Line
	3300 7000 4100 7000
Wire Wire Line
	4100 7000 4100 7300
Wire Wire Line
	4100 7300 3300 7300
Text GLabel 4300 7000 2    50   Input ~ 0
V_mid
Wire Wire Line
	4300 7000 4100 7000
Connection ~ 4100 7000
Wire Wire Line
	1650 7400 2400 7400
Wire Wire Line
	2400 7400 2400 7300
Wire Wire Line
	2400 7200 2000 7200
Wire Wire Line
	1800 7200 1800 7100
Wire Wire Line
	1800 7100 1650 7100
$Comp
L Device:R R1
U 1 1 5E0F10E9
P 1650 7250
F 0 "R1" H 1720 7296 50  0000 L CNN
F 1 "1k" H 1720 7205 50  0000 L CNN
F 2 "Resistor_SMD:R_1206_3216Metric_Pad1.42x1.75mm_HandSolder" V 1580 7250 50  0001 C CNN
F 3 "~" H 1650 7250 50  0001 C CNN
	1    1650 7250
	1    0    0    -1  
$EndComp
Text GLabel 2300 7000 0    50   Input ~ 0
output_power_en
Wire Wire Line
	2300 7000 2400 7000
Text GLabel 4100 4150 0    50   Input ~ 0
output_power_en
NoConn ~ 2400 7100
Text GLabel 9350 4700 2    50   Input ~ 0
mode_select
Text GLabel 9350 4400 2    50   Input ~ 0
filter_select
Text GLabel 9350 4500 2    50   Input ~ 0
audio_format_i2s_hi
Text GLabel 9350 4200 2    50   Input ~ 0
adc_powerdown
Text GLabel 9350 3800 2    50   Input ~ 0
audio_serial_clock
Text GLabel 9350 3700 2    50   Input ~ 0
master_clk
Text GLabel 9350 3900 2    50   Input ~ 0
lrclk
Text GLabel 9350 4000 2    50   Input ~ 0
audio_serial
Text GLabel 9350 4300 2    50   Input ~ 0
15db_gain_en
Text Notes 9200 5050 1    50   ~ 0
Todo: Figure out why AK5720VT didn't work right
NoConn ~ 9350 3700
NoConn ~ 9350 3800
NoConn ~ 9350 3900
NoConn ~ 9350 4000
NoConn ~ 9350 4200
NoConn ~ 9350 4300
NoConn ~ 9350 4400
NoConn ~ 9350 4500
NoConn ~ 9350 4700
$Comp
L power:GND #PWR016
U 1 1 5E115979
P 6150 6950
F 0 "#PWR016" H 6150 6700 50  0001 C CNN
F 1 "GND" H 6155 6777 50  0000 C CNN
F 2 "" H 6150 6950 50  0001 C CNN
F 3 "" H 6150 6950 50  0001 C CNN
	1    6150 6950
	-1   0    0    1   
$EndComp
$Comp
L Connector_Generic:Conn_01x04 J3
U 1 1 5E107773
P 6350 7150
F 0 "J3" V 6222 7330 50  0000 L CNN
F 1 "Conn_01x04" V 6313 7330 50  0000 L CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical" H 6350 7150 50  0001 C CNN
F 3 "PPPC041LGBN-RC‎" H 6350 7150 50  0001 C CNN
	1    6350 7150
	0    1    1    0   
$EndComp
NoConn ~ 4100 3550
NoConn ~ 4100 3650
Wire Wire Line
	3500 4450 4100 4450
Wire Wire Line
	3500 4550 4100 4550
Wire Wire Line
	3500 4650 4100 4650
Wire Wire Line
	3500 4750 4100 4750
Wire Wire Line
	3050 4250 3500 4250
Wire Wire Line
	3500 4250 3500 4350
$Comp
L power:GND #PWR0102
U 1 1 5DFFEC71
P 6300 4150
F 0 "#PWR0102" H 6300 3900 50  0001 C CNN
F 1 "GND" H 6305 3977 50  0000 C CNN
F 2 "" H 6300 4150 50  0001 C CNN
F 3 "" H 6300 4150 50  0001 C CNN
	1    6300 4150
	0    -1   1    0   
$EndComp
$Comp
L power:GND #PWR01
U 1 1 5E20C0B7
P 4100 3450
F 0 "#PWR01" H 4100 3200 50  0001 C CNN
F 1 "GND" H 4105 3277 50  0000 C CNN
F 2 "" H 4100 3450 50  0001 C CNN
F 3 "" H 4100 3450 50  0001 C CNN
	1    4100 3450
	0    1    -1   0   
$EndComp
Text GLabel 1150 2900 0    50   Input ~ 0
V_mid
$Comp
L Regulator_Linear:LM1117-3.3 U3
U 1 1 5E0AEB06
P 2550 2900
F 0 "U3" H 2550 3142 50  0000 C CNN
F 1 "LM1117-5" H 2550 3051 50  0000 C CNN
F 2 "Package_TO_SOT_SMD:SOT-223-3_TabPin2" H 2550 2900 50  0001 C CNN
F 3 "LM1117MPX-3.3/NOPB" H 2550 2900 50  0001 C CNN
	1    2550 2900
	1    0    0    -1  
$EndComp
Text GLabel 3650 2800 2    50   Input ~ 0
TEENSY_5V_SUPPLY
$Comp
L Device:C C_pwr2
U 1 1 5E156E90
P 2850 3200
F 0 "C_pwr2" V 2598 3200 50  0000 C CNN
F 1 "10u" V 2689 3200 50  0000 C CNN
F 2 "Capacitor_SMD:C_1206_3216Metric" H 2888 3050 50  0001 C CNN
F 3 "GRT31CR61H106ME01L" H 2850 3200 50  0001 C CNN
	1    2850 3200
	-1   0    0    -1  
$EndComp
Wire Wire Line
	2550 3200 2550 3350
Wire Wire Line
	2850 3050 2850 2900
Wire Wire Line
	2850 3350 2550 3350
Connection ~ 2550 3350
Wire Wire Line
	2550 3350 2550 3450
$Comp
L Device:C C_pwr1
U 1 1 5E16A6E7
P 2250 3200
F 0 "C_pwr1" V 1998 3200 50  0000 C CNN
F 1 "10u" V 2089 3200 50  0000 C CNN
F 2 "Capacitor_SMD:C_1206_3216Metric" H 2288 3050 50  0001 C CNN
F 3 "GRT31CR61H106ME01L" H 2250 3200 50  0001 C CNN
	1    2250 3200
	1    0    0    1   
$EndComp
Wire Wire Line
	2250 3050 2250 2900
Connection ~ 2250 2900
Wire Wire Line
	2250 3350 2550 3350
$Comp
L power:PWR_FLAG #FLG0101
U 1 1 5DFA0D02
P 1150 2900
F 0 "#FLG0101" H 1150 2975 50  0001 C CNN
F 1 "PWR_FLAG" H 1150 3073 50  0000 C CNN
F 2 "" H 1150 2900 50  0001 C CNN
F 3 "~" H 1150 2900 50  0001 C CNN
	1    1150 2900
	1    0    0    -1  
$EndComp
Wire Wire Line
	1150 2900 2250 2900
$Comp
L Connector:Conn_Coaxial_Power J4
U 1 1 5E0129FA
P 1150 3150
F 0 "J4" H 1238 3146 50  0000 L CNN
F 1 "Conn_Coaxial_Power" H 1238 3055 50  0000 L CNN
F 2 "Connector_BarrelJack:BarrelJack_CUI_PJ-102AH_Horizontal" H 1150 3100 50  0001 C CNN
F 3 "PJ-102AH" H 1150 3100 50  0001 C CNN
	1    1150 3150
	1    0    0    -1  
$EndComp
Wire Wire Line
	1150 3050 1150 2900
Connection ~ 1150 2900
$Comp
L power:GND #PWR0109
U 1 1 5E01459B
P 1150 3350
F 0 "#PWR0109" H 1150 3100 50  0001 C CNN
F 1 "GND" H 1155 3177 50  0000 C CNN
F 2 "" H 1150 3350 50  0001 C CNN
F 3 "" H 1150 3350 50  0001 C CNN
	1    1150 3350
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR09
U 1 1 5E0B5114
P 2550 3450
F 0 "#PWR09" H 2550 3200 50  0001 C CNN
F 1 "GND" H 2555 3277 50  0000 C CNN
F 2 "" H 2550 3450 50  0001 C CNN
F 3 "" H 2550 3450 50  0001 C CNN
	1    2550 3450
	-1   0    0    -1  
$EndComp
$Comp
L Switch:SW_SPDT SW1
U 1 1 5E23314A
P 3450 2800
F 0 "SW1" H 3450 2475 50  0000 C CNN
F 1 "5V_BYPASS" H 3450 2566 50  0000 C CNN
F 2 "Jumper:SolderJumper-3_P1.3mm_Open_RoundedPad1.0x1.5mm" H 3450 2800 50  0001 C CNN
F 3 "~" H 3450 2800 50  0001 C CNN
	1    3450 2800
	-1   0    0    1   
$EndComp
Wire Wire Line
	2850 2900 3250 2900
Connection ~ 2850 2900
Wire Wire Line
	2250 2900 2250 2550
Wire Wire Line
	2250 2550 3250 2550
Wire Wire Line
	3250 2550 3250 2700
Text GLabel 3050 2900 3    50   Input ~ 0
5V
Text GLabel 6300 5750 2    50   Input ~ 0
TEENSY_5V_SUPPLY
$Comp
L Diode:BZX384-xxx D1
U 1 1 5E294ED7
P 7650 6700
F 0 "D1" V 7604 6779 50  0000 L CNN
F 1 "BZX384-xxx" V 7695 6779 50  0000 L CNN
F 2 "Diode_SMD:D_SOD-323" H 7650 6525 50  0001 C CNN
F 3 "https://assets.nexperia.com/documents/data-sheet/BZX384_SERIES.pdf" H 7650 6700 50  0001 C CNN
	1    7650 6700
	0    1    1    0   
$EndComp
$Comp
L Diode:BZX384-xxx D2
U 1 1 5E296D4D
P 8300 6700
F 0 "D2" V 8254 6779 50  0000 L CNN
F 1 "BZX384-xxx" V 8345 6779 50  0000 L CNN
F 2 "Diode_SMD:D_SOD-323" H 8300 6525 50  0001 C CNN
F 3 "https://assets.nexperia.com/documents/data-sheet/BZX384_SERIES.pdf" H 8300 6700 50  0001 C CNN
	1    8300 6700
	0    1    1    0   
$EndComp
Text GLabel 7650 6550 1    50   Input ~ 0
tx2
Text GLabel 8300 6550 1    50   Input ~ 0
rx2
$Comp
L power:GND #PWR02
U 1 1 5E2978F8
P 7650 6850
F 0 "#PWR02" H 7650 6600 50  0001 C CNN
F 1 "GND" H 7655 6677 50  0000 C CNN
F 2 "" H 7650 6850 50  0001 C CNN
F 3 "" H 7650 6850 50  0001 C CNN
	1    7650 6850
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR03
U 1 1 5E2980CA
P 8300 6850
F 0 "#PWR03" H 8300 6600 50  0001 C CNN
F 1 "GND" H 8305 6677 50  0000 C CNN
F 2 "" H 8300 6850 50  0001 C CNN
F 3 "" H 8300 6850 50  0001 C CNN
	1    8300 6850
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR?
U 1 1 5E2C23CB
P 2000 7200
F 0 "#PWR?" H 2000 6950 50  0001 C CNN
F 1 "GND" H 2005 7027 50  0000 C CNN
F 2 "" H 2000 7200 50  0001 C CNN
F 3 "" H 2000 7200 50  0001 C CNN
	1    2000 7200
	-1   0    0    1   
$EndComp
Connection ~ 2000 7200
Wire Wire Line
	2000 7200 1800 7200
$EndSCHEMATC
