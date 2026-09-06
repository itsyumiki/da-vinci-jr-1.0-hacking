# Hotend

The hotend components are mostly standard, with an extra flash chip for hotend identification.

- 1x Heater
- 1x NTC
- 1x Fan
- 1x Filament sensor
- 1x Atmel AT24C02D Flash Memory

## Heater

The heater is a standard heater element connected to the hotend. I will add photos of it later.

### Pinout

| Pin | Name | Description  | MCU    | Verified? |
| --- | ---- | ------------ | ------ | --------- |
| 01  | -    | Heater input | PIO0_9 | ✅        |

## NTC

Standard NTC thermistor

### Pinout

| Pin | Name | Description | MCU    | Verified? |
| --- | ---- | ----------- | ------ | --------- |
| 01  | -    | NTC pin     | PIO1_0 | ✅        |

## Fan

### Pinout

| Pin | Name | Description | MCU    | Verified? |
| --- | ---- | ----------- | ------ | --------- |
| 01  | -    | Hotend fan  | PIO2_5 | ✅        |

## Filament Sensor

### Pinout

| Pin | Name | Description | MCU    | Verified? |
| --- | ---- | ----------- | ------ | --------- |
| 01  | -    | Sensor out  | PIO0_6 | ✅        |

## Atmel AT24C02D Flash Memory

### Pinout

| Pin | Name | Description   | MCU  | Verified? |
| --- | ---- | ------------- | ---- | --------- |
| 01  | A0   | Device addr   | GND  | ✅        |
| 02  | A1   | Device addr   | GND  | ✅        |
| 03  | A2   | Device addr   | GND  | ✅        |
| 04  | GND  | Ground        | GND  | ✅        |
| 05  | SDA  | Serial Data   | PA3  | ✅        |
| 06  | SCL  | Serial Clock  | PA4  | ✅        |
| 07  | WP   | Write Protect | GND  | ✅        |
| 08  | VCC  | 3.3V          | 3.3V | ✅        |

Refer to the [Atmel AT24C01D/AT24C02D Flash Memory Datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/AT24C01D-AT24C02D-I2C-Compatible-Two-Wire-Serial-EEPROM-1Kbit-2Kbit-20006100A.pdf) datasheet for more information.

### Photos

| 8 Pin SOIC package AT24C02D chip from top-view. First pin starts on the left of the bottom side, and goes counterclockwise. | Datasheet diagram with the first pin on top of the left side. |
| --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| ![Top View](../images/chips/AT24C02D.jpeg)                                                                                  | ![Datasheet diagram](../images/chips/AT24C02D-datasheet.png)  |
