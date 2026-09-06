# RFID Reader

I am working on the pinout as the secondary MCU firmware uses it and I need to map the pins.
More details are available [here](https://youtu.be/cn2mYWmanlk?t=556).

The important chip on this board is **PN512 NFC Chip**.

| Front view without the shield                          | Back view of the board                                |
| ------------------------------------------------------ | ----------------------------------------------------- |
| ![Front view](../images/components/rfid-reader-2.jpeg) | ![Back view](../images/components/rfid-reader-3.jpeg) |

| Front view of the board while its mounted (YouTube)        | Back view of the board while its mounted (Reddit)             | Front view of the board while its mounted (Reddit)             | Back view of the board while its mounted (Reddit)             |
| ---------------------------------------------------------- | ------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------- |
| ![Front view (YT)](../images/components/rfid-reader-1.png) | ![Back view (Reddit)](../images/components/rfid-reader-4.jpg) | ![Front view (Reddit)](../images/components/rfid-reader-6.jpg) | ![Back view (Reddit)](../images/components/rfid-reader-5.jpg) |

The board is **60mm to 36.5mm** in size. It seems to share the other characteristics of the other PCB's.

### Pinout

The board has a 6 pin connector going to the main board, and a 2 pin antenna connector. We only need the 6 pin connector.

It connects to the secondary MCU.

| Pin | Name   | Description | MCU    | Verified? |
| --- | ------ | ----------- | ------ | --------- |
| 01  | VIN    | Power input | 3.3V   | ✅        |
| 02  | GND    | Ground      | GND    | ✅        |
| 03  | NRSTPD |             | PIO3_1 | ✅        |
| 04  | ALE    |             | PIO3_0 | ✅        |
| 05  | IRQ    |             | PIO0_5 | ✅        |
| 06  | D7     |             | PIO0_4 | ✅        |

## PN512 NFC Chip

Refer to the [NXP PN512 Datasheet](../SOURCES.md#datasheet-nxp-pn512) for more information.

### Pinout

It can be found in the [datasheet](../SOURCES.md#datasheet-nxp-pn512).

### Photos

| 32 Pin HVQFN package PN512 chip from top-view. First pin starts on the top of the left side, and goes counterclockwise | Datasheet diagram                                         |
| ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| ![Top view](../images/chips/PN512.jpg)                                                                                 | ![Datasheet diagram](../images/chips/PN512-datasheet.png) |

## Extras

This board also has a LDO regulator (RT9179). It took me some time to figure out the chip since its very small and a discontinued part.

Information about the chip can be found [here](https://basicaodaeletronica.com.br/smd/pdfs/rt9179_richtek.pdf). I wont add it to sources as its not a mission-critical part, but it is still important when tracing the pins for this specific board.

It has 68 kΩ to Vout, 45.3 kΩ to GND; which calculates to a little below 3.0V.
