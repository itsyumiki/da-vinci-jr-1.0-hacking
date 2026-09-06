# Main Board

The main board is the brain of the printer. It has **2 MCU's, a 4MB Flash memory, 4 stepper drivers**, various voltage regulators and some crystal oscillators. It has a lot of unpopulated headers, connectors and chip sockets, mainly for other versions of the device that use the same board. There are also some debug headers and connectors.

| Front view of the board                          | Back view of the board                         |
| ------------------------------------------------ | ---------------------------------------------- |
| ![Front view](../images/hd/main-board-front.jpg) | ![Back view](../images/hd/main-board-back.jpg) |

The board is **100mm to 240mm** in size without the headers and connectors. It seems to be only dual layer, which makes it easier to work with. It is 1.5mm thick.

**The components that i was able to identify so far are:**

- **1x** Atmel SAM4E8E MCU
- **1x** NXP LPC1115 MCU
- **4x** Toshiba TB62269FTG Stepper driver
- **1x** Macronix MX25L3206E 4MB Flash memory
- **1x** AC-1203D RP1 Buzzer

**The ones awaiting identification are:**

- **2x** 12.000 Hz crystal oscillators
- **1x** unmarked crystal oscillator (for RTC?)

There are some more chips that i didnt see as important, as they are passive or hardware driven.

## Atmel SAM4E8E MCU

A 32-bit ARM Cortex-M4 MCU with FPU, DSP instructions and Thumb-2 instruction set. It can run at up to 120 MHz.

It has:

- **512KB Flash**
- **128KB SRAM**
- **16KB ROM** with embedded boot loader (UART) and IAP routines
- **117 GPIO pins** available
- **12-bit ADC and DAC** natively, that can sample up to **1 Msps**
- Possible to go up to 16-bit via oversampling

Refer to the [Atmel SAM4E Datasheet](../SOURCES.md#datasheet-atmel-sam4e8e) for more information.

### Pinout

It can be found here: [sam4e8e-pinout.md](pinouts/sam4e8e-pinout.md)

### Photos

| 144 Pin LQFP package SAM4E8E chip from top-view. First pin starts on the left of the bottom side, and goes counterclockwise. | Datasheet diagram (no pin numbers).                           |
| ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| ![Top view](../images/chips/ATSAM4E8E.jpeg)                                                                                  | ![Datasheet diagram](../images/chips/ATSAM4E8E-datasheet.png) |

## NXP LPC1115 MCU

A 32-bit ARM Cortex-M0 MCU with Thumb instruction set. It can run at up to 50 MHz.

It has:

- **64KB of Flash**
- **8KB of SRAM**
- **16KB of ROM** with embedded boot loader (UART) and IAP routines
- **Up to 42 GPIO pins** available
- **10-bit ADC**, no confirmed DAC
-

Refer to the [NXP LPC111x Datasheet](../SOURCES.md#datasheet-nxp-lpc111x) for more information.

### Pinout

It can be found here: [lpc1115-pinout.md](pinouts/lpc1115-pinout.md)

### Photos

| 48 Pin LQFP package LPC1115 chip from top-view. First pin starts on the left of the bottom side, and goes counterclockwise. | Datasheet diagram with the first pin on top of the left side. |
| --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| ![Top view](../images/chips/LPC1115.jpg)                                                                                    | ![Datasheet diagram](../images/chips/LPC1115-datasheet.png)   |

## Toshiba TB62269FTG Driver

A two-phase bipolar stepping motor driver using a PWM chopper. It can run on **full, 1/2, 1/4, 1/8, 1/16, and 1/32 steps**.

Refer to the [Toshiba TB62269FTG Datasheet](../SOURCES.md#datasheet-toshiba-tb62269ftg) for more information.

### Pinout

Per-chip connections: [steppers.md](pinouts/steppers.md)

General pinout: [tb62269ftg-pinout.md](pinouts/tb62269ftg-pinout.md)

### Photos

| 48 Pin WQFN package TB62269FTG chip from top-view. First pin starts on the left of the bottom side, and goes counterclockwise. | Datasheet diagram with the first pin on the left of the bottom side. |
| ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| ![Top view](../images/chips/TB62269FTG.jpeg)                                                                                   | ![Datasheet diagram](../images/chips/TB62269FTG-datasheet.png)       |

## Macronix MX25L3206E Flash Memory

A 32 Mbit (4MB) CMOS serial NOR flash chip with a Serial Peripheral Interface (3-wire bus: clock, serial data in, serial data out), usable in single or dual output mode.

It has:
  
  - **SPI connection** with a max clock speed of 86MHz (80MHz in Dual Output mode)
  
  Refer to the [Macronix MX25L3206E Datasheet](../SOURCES.md#datasheet-macronix-mx25l3206e) for more information.

### Pinout

| Pin | Name    | Description               | MCU  | Verified? |
| --- | ------- | ------------------------- | ---- | --------- |
| 01  | CS#     | Chip Select               | pa11     | ❌        |
| 02  | SO/SIO1 | Serial Data Output        |  pa12     | ❌        |
| 03  | WP#     | Write protection          |      | ❌        |
| 04  | GND     | Ground                    | GND  | ✅        |
| 05  | SI/SIO0 | Serial Data Input         | PA13 | ❌        |
| 06  | SCLK    | Clock Input               |  PA14    | ❌        |
| 07  | HOLD#   | Hold, to pause the device |      | ❌        |
| 08  | VCC     | + 3.3V Power Supply       | 3.3V | ✅        |

### Photos

| 8 Pin SOP package MX25L3206E chip from top-view. First pin starts on top of the left side, and goes counterclockwise. | Datasheet diagram with the first pin on top of the left side.  |
| --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| ![Top view](../images/chips/MX25L3206E.jpeg)                                                                          | ![Datasheet diagram](../images/chips/MX25L3206E-datasheet.png) |

## AC-1203D RP1 Buzzer

Despite not being a critical component, it was fitting to put it here.

Refer to the [AATC AC-1203D RP1 Datasheet](https://aatc.tw/mouser_2025/AC-1203D-RP1.pdf) for more information.

### Pinout

| Pin | Name | Description | MCU | Verified? |
| --- | ---- | ----------- | --- | --------- |
| 01  |      |             |     | ❌        |

### Photos

| Top view of the buzzer                             |
| -------------------------------------------------- |
| ![Top view](../images/components/AC-1203D-RP1.jpg) |
