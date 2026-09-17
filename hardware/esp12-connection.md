# ESP12 Connection

> This file is unstable, anything can change at any time.

The pins that we need to use on ESP12 are:

| Pin    | Function       | Usage                             |
| ------ | -------------- | --------------------------------- |
| RST    | Reset          | Used on boot and firmware updates |
| CH_PD  | Enable         | Used on boot                      |
| GPIO14 | SPI SCK        | Used in normal function           |
| GPIO12 | SPI MISO       | Used in normal function           |
| GPIO13 | SPI MOSI       | Used in normal function           |
| VCC    | 3.3V in        | 3.3V power input                  |
| GND    | Ground         | Ground                            |
| GPIO15 | SPI CS         | Used in normal function           |
| GPIO0  | Data ready     | Used in normal function           |
| GPIO4  | Transfer ready | Used in normal function           |
| GPIO3  | RXD            | Used in firmware updates          |
| GPIO1  | TXD            | Used in firmware updates          |

We need 10 pins for full functionality.

The U18 gives us:

| Pin  | Function |
| ---- | -------- |
| PD24 | Generic  |
| GND  | Ground   |
| PB02 | Generic  |
| PA13 | MOSI     |
| PA14 | SCLK     |
| PA12 | MISO     |
| PE03 | Generic  |
| PA26 | Generic  |

The closeby pins:

| Pin  | Connector | Name    |
| ---- | --------- | ------- |
| PD29 | J24 1     | TopHOME |
| 3.3V | J24 3     | TopHOME |
| PC24 | J8 2      | LASER R |
| PB14 | J7 2      | LASER L |
| 3.3V | R86       | BedNTC  |

Connecting all of these pins should be enough for full functionality, but i would like to prioritize basic functionality pins to use the U18 so we are not dependent on the closeby pins.

**FIXME:** Create 2 guides, one for using the U18 only, and another one for the full connection.

I will be matching the pins in the way that requires the shortest wire connections and easiest to solder.

Okay a bit of plan change, i plan to solder the ESP12 upside down (shield side down) so the pin-heavy right side of ESP aligns with the gpio pin heavy left side of U18 and the connectors. I feel like it might make it a bit easier to solder too, since the pins on the current U18 footprint have very wrong spacings and short the pins on ESP12.

Another note: every sam pin here is explicitly driven and default to high on boot, but we need to ground the reset pin since while sam is updating itself, the status is undefined.

**Warning:** The proposals below are just a snapshot of my mental proposal, not a final design yet at all. It might include shorts, mislabeled connections, and more. When its finalized, firmware built and flashed, i will update this guide to remove the warnings and the extra notes.

Current proposal (it uses all the available pins in order to be the shortest pinout):

### Left side

| ESP Pin | Function       | SAM Pin |
| ------- | -------------- | ------- |
| TXD0    | UART TXD       | PD29    |
| RXD0    | UART RXD       | PC24    |
| GPIO4   | Transfer ready | PB14    |
| GPIO0   | Data ready     | PD24    |
| GPIO15  | SPI CS         | PB02    |
| GND     | Ground         | GND     |

### Right side

| ESP Pin | Function | SAM Pin |
| ------- | -------- | ------- |
| RST     | Reset    | PA26    |
| CH_PD   | Enable   | PE03    |
| GPIO14  | SPI SCK  | PA14    |
| GPIO12  | SPI MISO | PA12    |
| GPIO13  | SPI MOSI | PA13    |
| VCC     | 3.3V in  | 3.3V    |

The current proposal is pretty solid on paper, but we need to verify it by building a firmware with these pinouts and hardware testing it.

Second base functionality proposal will be added later.
