# Sub-board

The sub-board is the board for the interactivity of the printer. It has **6 push buttons, a 16x4 LCD display, and a full-size SD card slot**. It also has a **16 pin connector** labeled "SD CARD SLOT CONNECTOR", and a **22 pin connector** labeled "LCM SLOT CONNECTOR". It has no visible MCU's, only some power related chips and components. It also has no visible unpopulated headers, connectors and chip sockets.

| Front view of the board                                               | Back view of the board                        |
| --------------------------------------------------------------------- | --------------------------------------------- |
| ![Front view](../images/hd/sub-board-front.jpg)                       | ![Back view](../images/hd/sub-board-back.jpg) |
| ![Front desoldered view](../images/hd/sub-board-front-desoldered.jpg) |                                               |

The board is **62mm to 185mm** in size.

**The components that i was able to identify so far are:**

- **1x** Winstar WH1604A 16x04 LCD module
- **1x** Generic SD card reader module
- **6x** Generic 6mm push buttons

There are some more chips that i didnt see as important, as they are passive or hardware driven. More information about these can be found in the [schematic](../schematic.md).

More information about the connectors on-board can be found in the [connectors](connectors.md) section.

## Winstar WH1604A 16x04 LCD module

The Winstar WH1604A 16x04 LCD module is a character LCD display that is used to display text on the sub-board. It has 16 columns and 4 rows of characters, and is connected to the main board via a 22 pin connector labeled "LCM SLOT CONNECTOR".

Refer to the [Winstar WH1604A 16x04 LCD Module Datasheet](../SOURCES.md#datasheet-winstar-wh1604a) for more information.

### Pinout

The connector mentioned here is the 22 pin connector.

| Pin | Name | Description                  | Connector Pin | Verified? | MCU Pin | Verified? |
| --- | ---- | ---------------------------- | ------------- | --------- | ------- | --------- |
| 01  | VSS  | Ground                       | GND           | ✅        | GND     | ✅        |
| 02  | VDD  | Supply Voltage for logic     | 5V            | ✅        | 5V      | ✅        |
| 03  | VO   | Contrast                     | -             | ✅        | -       | ✅        |
| 04  | RS   | H: DATA, L: Instruction code | 05            | ✅        | PC18    | ✅        |
| 05  | R/W  | H: Read L: Write             | 04            | ✅        | PC8     | ✅        |
| 06  | E    | Chip enable signal           | 03            | ✅        | PC13    | ✅        |
| 07  | DB0  | Data bus line                | 06            | ✅        | PC0     | ✅        |
| 08  | DB1  | Data bus line                | 07            | ✅        | PC1     | ✅        |
| 09  | DB2  | Data bus line                | 08            | ✅        | PC2     | ✅        |
| 10  | DB3  | Data bus line                | 09            | ✅        | PC3     | ✅        |
| 11  | DB4  | Data bus line                | 10            | ✅        | PC4     | ✅        |
| 12  | DB5  | Data bus line                | 11            | ✅        | PC5     | ✅        |
| 13  | DB6  | Data bus line                | 12            | ✅        | PC6     | ✅        |
| 14  | DB7  | Data bus line                | 13            | ✅        | PC7     | ✅        |
| 15  | A    | Backlight +                  | 5V            | ✅        | 5V      | ✅        |
| 16  | K    | Backlight -                  | 14            | ✅        | PC10    | ✅        |

### Photos

| Front view                                               | Back view                                              |
| -------------------------------------------------------- | ------------------------------------------------------ |
| ![Front view](../images/components/lcd-module-front.jpg) | ![Back view](../images/components/lcd-module-back.jpg) |

## SD Card Reader Module

The SD card reader module is used to read full size SD cards and is connected to the main board via a 16 pin connector labeled "SD CARD SLOT CONNECTOR".

The WP and CD pins are grounded when they are triggered, like:

- No card inserted: CD is grounded
- Card inserted but write-protected: WP is grounded
- Card inserted and not write-protected: Both are floating/non-grounded

It is a generic SD card reader module, so there is no specific datasheet available.

### Pinout

Pins start from P1 (on the right of the reader) and go to the left, up to P11.

The connector mentioned here is the 16 pin connector.

| Pin | Name    | Description        | Connector Pin | Verified? | MCU Pin | Verified? |
| --- | ------- | ------------------ | ------------- | --------- | ------- | --------- |
| 01  | DAT2    | -                  | 16            | ✅        | PA26    | ✅        |
| 02  | CS/DAT3 | Chip Select        | 14            | ✅        | PA27    | ✅        |
| 03  | CMD     | MOSI/Data in       | 12            | ✅        | PA28    | ✅        |
| 04  | VSS1    | Ground             | GND           | ✅        | GND     | ✅        |
| 05  | VDD     | Power              | 09            | ✅        | 3.3V    | ✅        |
| 06  | CLK     | SCK                | 07            | ✅        | PA29    | ✅        |
| 07  | VSS2    | Ground             | GND           | ✅        | GND     | ✅        |
| 08  | DAT0    | MISO               | 05            | ✅        | PA30    | ✅        |
| 09  | DAT1    | -                  | 03            | ✅        | PA31    | ✅        |
| 10  | WP      | Write Protect lock | -             | ✅        | -       | ✅        |
| 11  | CD      | Card Detect        | 10            | ✅        | PA25    | ✅        |

### Photos

| Top view of the SD card reader                       |
| ---------------------------------------------------- |
| ![Top view](../images/components/sd-card-reader.jpg) |

## Push Buttons

The push buttons are used to control the sub-board and are connected to the main board via one of the existing connectors.

They are pulled up to 5V and grounded when pressed.

I doubt a datasheet for a button would be useful even.

### Pinout

The connector mentioned here is the 22 pin connector.

| Button | Name   | Description  | Connector Pin | Verified? | MCU Pin | Verified? |
| ------ | ------ | ------------ | ------------- | --------- | ------- | --------- |
| SW1    | UP     | Up button    | 18            | ✅        | PE1     | ✅        |
| SW2    | DOWN   | Down button  | 16            | ✅        | PA21    | ✅        |
| SW3    | RIGHT  | Right button | 19            | ✅        | PE4     | ✅        |
| SW4    | LEFT   | Left button  | 17            | ✅        | PB3     | ✅        |
| SW5    | ENTER  | Enter button | 20            | ✅        | PA17    | ✅        |
| SW6    | ESCAPE | Home button  | 15            | ✅        | PD30    | ✅        |

### Photos

| Top view of the push buttons                       |
| -------------------------------------------------- |
| ![Top view](../images/components/push-buttons.jpg) |
