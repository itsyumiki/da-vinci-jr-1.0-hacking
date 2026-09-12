# Connectors

There are 3 main data connectors on the Da Vinci Jr 1.0 board.

- 22-pin LCD connector
- 16-pin SD Card connector
- 51-pin Hotend connector

There are of course a lot more connectors on board, but these are the main ones needing identification.

## 51-pin Hotend connector

The 51-pin Hotend connector connects the main board with the hotend. Its mostly power wires, and a few data wires.

### Photos

| Main board connector                                                              | Sub-board connector                                            |
| --------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| ![Main board with text visible](../images/connectors/connector-51-pin-main-1.png) | ![Sub-board](../images/connectors/connector-51-pin-hotend.png) |
| ![Main board without text](../images/connectors/connector-51-pin-main-2.png)      |                                                                |

### Pinout

> Pins 1-6 and 46-51 are connected to each other, and they connect to the power via a fuse labeled R292 on board.
>
> IR sensor is the filament sensor. Its other end is connected to GND on the hotend, and its active low.
>
> J5 is a connector that I have no idea what it is for yet. I'm guessing its for calibrating the bed.

| Pin | Main board | Description   | Verified? | Hotend | Description   | Verified? |
| --- | ---------- | ------------- | --------- | ------ | ------------- | --------- |
| 01  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 02  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 03  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 04  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 05  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 06  | 12V        | Power 12V     | ✅        | -      | Not connected | ✅        |
| 07  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 08  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 09  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 10  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 11  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 12  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 13  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 14  | Q8 5-8     | Mosfet drain  | ✅        | -      | Not connected | ✅        |
| 15  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 16  | Q2 3       | Transistor C  | ✅        | J2 1   | Fan GND       | ✅        |
| 17  | Q2 3       | Transistor C  | ✅        | J2 1   | Fan GND       | ✅        |
| 18  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 19  | -          | -             | ❌        | GND    | Ground        | ✅        |
| 20  | -          | -             | ❌        | U1 3   | IR Signal     | ✅        |
| 21  | -          | -             | ❌        | U1 3   | IR Signal     | ✅        |
| 22  | -          | -             | ❌        | GND    | Ground        | ✅        |
| 23  | -          | -             | ❌        | U2 5   | EEPROM SDA    | ✅        |
| 24  | -          | -             | ❌        | U2 5   | EEPROM SDA    | ✅        |
| 25  |            | -             | ❌        | U2 8   | EEPROM VCC    | ✅        |
| 26  | -          | -             | ❌        | U2 8   | EEPROM VCC    | ✅        |
| 27  | -          | -             | ❌        | U2 6   | EEPROM SCL    | ✅        |
| 28  | -          | -             | ❌        | U2 6   | EEPROM SCL    | ✅        |
| 29  | -          | -             | ❌        | GND    | Ground        | ✅        |
| 30  | -          | -             | ❌        | J3 1   | NTC signal    | ✅        |
| 31  | -          | -             | ❌        | J3 1   | NTC signal    | ✅        |
| 32  | GND        | Ground        | ✅        | GND    | Ground        | ✅        |
| 33  | -          | -             | ❌        | J5 1   | Unknown conn  | ✅        |
| 34  | -          | -             | ❌        | J5 1   | Unknown conn  | ✅        |
| 35  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 36  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 37  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 38  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 39  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 40  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 41  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 42  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 43  | Q8 5-8     | Mosfet drain  | ✅        | J4 1   | Heater GND    | ✅        |
| 44  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 45  | -          | Not connected | ✅        | -      | Not connected | ✅        |
| 46  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |
| 47  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |
| 48  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |
| 49  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |
| 50  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |
| 51  | 12V        | Power 12V     | ✅        | 12V    | Power 12V     | ✅        |

## 22-pin LCD connector

The 22-pin LCD connector connects the main board with the sub-board, and carries the LCD display data. It also carries the buttons signals.

### Photos

| Main board connector                                          | Sub-board connector                                         |
| ------------------------------------------------------------- | ----------------------------------------------------------- |
| ![Main board](../images/connectors/connector-22-pin-main.png) | ![Sub-board](../images/connectors/connector-22-pin-sub.png) |

### Pinout

| Pin | Main Board | Description | Verified? | Sub-board | Description   | Verified? |
| --- | ---------- | ----------- | --------- | --------- | ------------- | --------- |
| 01  | +5V        | 5V Power    | ✅        | 5V        | 5V power      | ✅        |
| 02  | GND        | Ground      | ✅        | GND       | Ground        | ✅        |
| 03  | 111        | PC13        | ✅        | E         | LCD enable    | ✅        |
| 04  | 82         | PC8         | ✅        | R/W       | LCD R/W       | ✅        |
| 05  | U1_Pin1    | PC18        | ✅        | RS        | LCD RS        | ✅        |
| 06  | 11         | PC0         | ✅        | D0        | LCD data bus  | ✅        |
| 07  | 38         | PC1         | ✅        | D1        | LCD data bus  | ✅        |
| 08  | 39         | PC2         | ✅        | D2        | LCD data bus  | ✅        |
| 09  | 40         | PC3         | ✅        | D3        | LCD data bus  | ✅        |
| 10  | 41         | PC4         | ✅        | D4        | LCD data bus  | ✅        |
| 11  | 58         | PC5         | ✅        | D5        | LCD data bus  | ✅        |
| 12  | 54         | PC6         | ✅        | D6        | LCD data bus  | ✅        |
| 13  | 48         | PC7         | ✅        | D7        | LCD data bus  | ✅        |
| 14  | 90         | PC10        | ✅        | LCD       | LCD backlight | ✅        |
| 15  | 34         | PD29        | ✅        | ESCAPE    | Home button   | ✅        |
| 16  | 32         | PA21        | ✅        | DOWN      | Down button   | ✅        |
| 17  | 31         | PB3         | ✅        | LEFT      | Left button   | ✅        |
| 18  | 28         | PE5         | ✅        | UP        | Up button     | ✅        |
| 19  | 27         | PE4         | ✅        | RIGHT     | Right button  | ✅        |
| 20  | 25         | PA17        | ✅        | ENTER     | Enter button  | ✅        |
| 21  | GND        | Ground      | ✅        | GND       | Ground        | ✅        |
| 22  | +5V        | Power       | ✅        | 5V        | 5V Power      | ✅        |

## 16-pin SD Card connector

The 16-pin LCD connector connects the main board with the sub-board, and carries the SD card data.

### Photos

| Main board connector                                          | Sub-board connector                                         |
| ------------------------------------------------------------- | ----------------------------------------------------------- |
| ![Main board](../images/connectors/connector-16-pin-main.png) | ![Sub-board](../images/connectors/connector-16-pin-sub.png) |

### Pinout

| Pin | Main Board | Description       | Verified? | Sub-board | Description   | Verified? |
| --- | ---------- | ----------------- | --------- | --------- | ------------- | --------- |
| 01  | -          | Not connected     | ✅        | -         | Not connected | ✅        |
| 02  | GND        | Ground            | ✅        | GND       | Ground        | ✅        |
| 03  | 118        | PA31/MCDA1        | ❌        | DAT1      | SD Card DAT1  | ✅        |
| 04  | GND        | Ground            | ✅        | GND       | Ground        | ✅        |
| 05  | 116        | PA30/MCDA0        | ❌        | DAT0      | SD Card DAT0  | ✅        |
| 06  | GND        | Ground            | ✅        | GND       | Ground        | ✅        |
| 07  | 129        | PA29/MCCK         | ❌        | CLK       | SD Card CLK   | ✅        |
| 08  | +3.3V      | Power             | ❌        | +3.3V     | 3.3V Power    | ✅        |
| 09  | +3.3V      | Power             | ❌        | +3.3V     | 3.3V Power    | ✅        |
| 10  | 59         | PA25/PGMD13/CTS1  | ❌        | CD        | Card detect   | ✅        |
| 11  | GND        | Ground            | ❌        | GND       | Ground        | ✅        |
| 12  | 112        | PA28/MCCDA        | ❌        | CMD       | SD Card CMD   | ✅        |
| 13  | GND        | Ground            | ❌        | GND       | Ground        | ✅        |
| 14  | 70         | PA27/PGMD15/MCDA3 | ❌        | DAT3      | SD Card DAT3  | ✅        |
| 15  | GND        | Ground            | ❌        | GND       | Ground        | ✅        |
| 16  | 62         | PA28/PGMD14/MCDA2 | ❌        | DAT2      | SD Card DAT2  | ✅        |
