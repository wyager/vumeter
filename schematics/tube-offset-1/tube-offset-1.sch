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
$Comp
L Connector:Conn_01x04_Male J1
U 1 1 5DFC456D
P 1650 2500
F 0 "J1" H 1758 2781 50  0000 C CNN
F 1 "Conn_01x04_Male" H 1758 2690 50  0000 C CNN
F 2 "Connector_PinSocket_2.54mm:PinSocket_1x04_P2.54mm_Vertical" H 1650 2500 50  0001 C CNN
F 3 "~" H 1650 2500 50  0001 C CNN
	1    1650 2500
	1    0    0    -1  
$EndComp
$Comp
L Connector:Conn_01x04_Female J2
U 1 1 5DFC4EAC
P 2800 2450
F 0 "J2" H 2828 2426 50  0000 L CNN
F 1 "Conn_01x04_Female" H 2828 2335 50  0000 L CNN
F 2 "Connector_PinSocket_2.54mm:PinSocket_1x04_P2.54mm_Vertical" H 2800 2450 50  0001 C CNN
F 3 "ESQ-104-12-T-S" H 2800 2450 50  0001 C CNN
	1    2800 2450
	1    0    0    -1  
$EndComp
$Comp
L Connector:Conn_01x04_Male J4
U 1 1 5DFC7020
P 6950 2600
F 0 "J4" H 7058 2881 50  0000 C CNN
F 1 "Conn_01x04_Male" H 7058 2790 50  0000 C CNN
F 2 "Connector_PinSocket_2.54mm:PinSocket_1x04_P2.54mm_Vertical" H 6950 2600 50  0001 C CNN
F 3 "~" H 6950 2600 50  0001 C CNN
	1    6950 2600
	1    0    0    -1  
$EndComp
$Comp
L Connector:Conn_01x04_Female J3
U 1 1 5DFC7D19
P 6300 2500
F 0 "J3" H 6328 2476 50  0000 L CNN
F 1 "Conn_01x04_Female" H 6328 2385 50  0000 L CNN
F 2 "Connector_PinSocket_2.54mm:PinSocket_1x04_P2.54mm_Vertical" H 6300 2500 50  0001 C CNN
F 3 "ESQ-104-12-T-S" H 6300 2500 50  0001 C CNN
	1    6300 2500
	1    0    0    -1  
$EndComp
$Comp
L nixies-us:IN-13 N1
U 1 1 5DFC8DDD
P 3750 2500
F 0 "N1" H 3750 2770 45  0000 C CNN
F 1 "IN-13" H 3750 2500 45  0001 L BNN
F 2 "vumeter:fixed-IN-13-footprint" H 3780 2650 20  0001 C CNN
F 3 "" H 3750 2500 50  0001 C CNN
	1    3750 2500
	1    0    0    -1  
$EndComp
$Comp
L nixies-us:IN-13 N2
U 1 1 5DFCB068
P 4400 2500
F 0 "N2" H 4400 2770 45  0000 C CNN
F 1 "IN-13" H 4400 2500 45  0001 L BNN
F 2 "vumeter:fixed-IN-13-footprint" H 4430 2650 20  0001 C CNN
F 3 "" H 4400 2500 50  0001 C CNN
	1    4400 2500
	1    0    0    -1  
$EndComp
$Comp
L nixies-us:IN-13 N3
U 1 1 5DFCC1E7
P 5000 2500
F 0 "N3" H 5000 2770 45  0000 C CNN
F 1 "IN-13" H 5000 2500 45  0001 L BNN
F 2 "vumeter:fixed-IN-13-footprint" H 5030 2650 20  0001 C CNN
F 3 "" H 5000 2500 50  0001 C CNN
	1    5000 2500
	1    0    0    -1  
$EndComp
$Comp
L nixies-us:IN-13 N4
U 1 1 5DFCCA4B
P 5650 2500
F 0 "N4" H 5650 2770 45  0000 C CNN
F 1 "IN-13" H 5650 2500 45  0001 L BNN
F 2 "vumeter:fixed-IN-13-footprint" H 5680 2650 20  0001 C CNN
F 3 "" H 5650 2500 50  0001 C CNN
	1    5650 2500
	1    0    0    -1  
$EndComp
Wire Wire Line
	1850 2400 1850 2500
Wire Wire Line
	1850 2500 1850 2600
Connection ~ 1850 2500
Wire Wire Line
	1850 2600 1850 2700
Connection ~ 1850 2600
Wire Wire Line
	2600 2350 2600 2450
Wire Wire Line
	2600 2450 2600 2550
Connection ~ 2600 2450
Wire Wire Line
	2600 2550 2600 2650
Connection ~ 2600 2550
Wire Wire Line
	6100 2400 6100 2500
Wire Wire Line
	6100 2500 6100 2600
Connection ~ 6100 2500
Wire Wire Line
	6100 2600 6100 2700
Connection ~ 6100 2600
Wire Wire Line
	7150 2500 7150 2600
Wire Wire Line
	7150 2600 7150 2700
Connection ~ 7150 2600
Wire Wire Line
	7150 2700 7150 2800
Connection ~ 7150 2700
NoConn ~ 3450 2500
NoConn ~ 4050 2400
NoConn ~ 4050 2600
NoConn ~ 4100 2500
NoConn ~ 4700 2400
NoConn ~ 4700 2600
NoConn ~ 4700 2500
NoConn ~ 5300 2400
NoConn ~ 5300 2600
NoConn ~ 5350 2500
NoConn ~ 5950 2400
NoConn ~ 5950 2600
$EndSCHEMATC
