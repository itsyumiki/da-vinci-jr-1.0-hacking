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

Connecting all of these pins should be enough for full functionality, but i would like to prioritize basic functionality pins to use the U18 so we are not dependent on the closeby pins.

**FIXME:** Create 2 guides, one for using the U18 only, and another one for the full connection.

I will be matching the pins in the way that requires the shortest wire connections and easiest to solder.

Current proposal (it uses all the available pins in order to be the shortest pinout):

| ESP Pin | Function       | SAM Pin |
| ------- | -------------- | ------- |
| RST     | Reset          | PD24    |
| CH_PD   | Enable         | PD29    |
| GPIO14  | SPI SCK        | PA14    |
| GPIO12  | SPI MISO       | PA12    |
| GPIO13  | SPI MOSI       | PA13    |
| VCC     | 3.3V in        | J24 3   |
| GND     | Ground         | GND     |
| GPIO15  | SPI CS         |         |
| GPIO0   | Data ready     | PE03    |
| GPIO4   | Transfer ready | PA26    |
| GPIO3   | RXD            | PC24    |
| GPIO1   | TXD            | PB14    |

Second base functionality proposal will be added below soon.
