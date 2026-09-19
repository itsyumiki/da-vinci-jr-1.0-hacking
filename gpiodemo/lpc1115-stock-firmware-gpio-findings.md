# LPC1115 stock firmware GPIO findings

This note records GPIO and pin-mux use recovered from the Da Vinci Jr 1.0 LPC1115 `+2.2.0` stock firmware. The source image and byte-identical reverse-engineering project are in `henmabot/da-vinci-jr-lpc1115-firmware-re`. This audit uses commit `a0bbe13` and the archived image with SHA-1 `ab13e81742ccd2aa5085f48e6780063a43b98be4`.

The purpose of this document is to separate three things that are easy to conflate during pinout work:

1. pins the stock application actually touches;
2. pins only present in generic LPC11xx peripheral-driver branches that the application never selects; and
3. board functions already verified independently in this repository.

## Recovered GPIO helpers

The firmware's generic GPIO layer is identifiable from direct LPC1115 GPIO MMIO access:

| Address | Recovered name | Behavior |
| ---: | --- | --- |
| `0x3864` | `GPIOInit` | Enables the GPIO AHB clock. |
| `0x3874` | `GPIOSetValue` | Writes through the LPC masked GPIO access region. |
| `0x3888` | `GPIOGetValue` | Reads through the same masked GPIO access region. |
| `0x389c` | `GPIOSetDir` | Changes the GPIO `DIR` bit at offset `0x8000`. |

There are 37 application call sites to these helpers. Separate IOCON accesses in the recovered I2C, ADC, timer/PWM, and UART initialization code expose additional pins that do not pass through the GPIO helpers.

## Pins used by the stock execution path

| LPC pin | Package pin | Firmware evidence | Existing board knowledge | Interpretation |
| --- | ---: | --- | --- | --- |
| `PIO0_3` | 14 | Configured as GPIO input in `BoardGPIO_Init`. No stock `GPIOGetValue` use found. | Currently unknown. | Reserved or variant input; exact purpose unknown. |
| `PIO0_4/SCL` | 15 | `I2CInit` configures IOCON for I2C SCL. | Verified to NFC board J12/J123 pin 6. | NFC I2C SCL. |
| `PIO0_5/SDA` | 16 | `I2CInit` configures IOCON for I2C SDA. | Verified to NFC board J12/J123 pin 5. | NFC I2C SDA. |
| `PIO0_6` | 22 | Configured input and sampled by the stock GPIO polling path. | Verified hotend filament sensor, J11 U1. | Hotend filament sensor. |
| `PIO0_7` | 23 | Configured as GPIO input. No stock `GPIOGetValue` use found. | Currently unknown. | Reserved or variant input; exact purpose unknown. |
| `PIO0_8` | 27 | `init_timer16PWM` muxes it as a CT16B0 PWM output selected by one application PWM channel. | Currently unknown. | Intentional second PWM-controlled output; load still unknown. |
| `PIO0_9` | 28 | `init_timer16PWM` muxes it as the sibling CT16B0 PWM output; application match updates select it as the heater channel. | Verified heater, J11 J4. | Heater PWM. |
| `PIO1_0` | 33 | `ADCInit` configures it for ADC; the application repeatedly samples the corresponding ADC channel. | Verified hotend NTC, J11 J3. | Hotend thermistor input. |
| `PIO1_1` | 34 | `ADCInit` configures it for ADC; the application alternates ADC sampling between the channels corresponding to `PIO1_0` and `PIO1_1`. | No visible connection in the current pinout. | Real second analog input expected by firmware; attached circuit still unknown. |
| `PIO1_6/RXD` | 46 | `UARTInit` muxes IOCON offset `0xa4` to UART function 1 immediately before UART setup. | Existing GPIO demo already reserves the PIO1_6/PIO1_7 UART link. | LPC UART RX, used for the SAM-LPC link. |
| `PIO1_7/TXD` | 47 | `UARTInit` muxes IOCON offset `0xa8` to UART function 1 immediately before UART setup. | Existing GPIO demo already reserves the PIO1_6/PIO1_7 UART link. | LPC UART TX, used for the SAM-LPC link. |
| `PIO1_10` | 30 | Configured as GPIO output and changed from the application's digital-output/fan command path. | Verified reflow fan, J121/J122. | Reflow fan control. |
| `PIO1_11` | 42 | Configured as GPIO output, initialized low, and deliberately switched high/low by application logic. | Currently unknown. | Real handshake/enable-style output, but the attached device is not established yet. |
| `PIO2_1` | 13 | Configured input and sampled by the stock polling path. | Verified E1 rotation sensor, J19/J14. | Extruder rotation sensor. |
| `PIO2_4` | 19 | Configured as GPIO output and switched in the same application command family as known fan outputs. | Currently unknown. | Unknown board output; likely related to the fan/output family, but exact load needs electrical confirmation. |
| `PIO2_5` | 20 | Configured output; written and also read back by application code. | Verified hotend fan, J11 J2. | Hotend fan control. |
| `PIO2_7` | 11 | Configured input and sampled by the stock polling path. | Verified filament runout sensor, J19/J14. | Filament runout sensor. |
| `PIO2_8` | 12 | Configured as GPIO input. No stock `GPIOGetValue` use found. | Currently unknown. | Reserved or variant input; exact purpose unknown. |
| `PIO2_11` | 31 | Configured as GPIO output and switched by the same application output family as `PIO2_4`/`PIO2_5`. | No visible connection in the current pinout. | Unknown or variant output; do not assign a specific load without hardware evidence. |
| `PIO3_0/TXD` | 36 | Configured as GPIO output and driven low then high during startup. | Verified connection to NFC board J12/J123 pin 4. | Very likely NFC reset/enable; physical connection is verified, exact signal name is not. |

### Strong new candidates for hardware tracing

`PIO0_8`, `PIO1_1`, `PIO1_11`, `PIO2_4`, and `PIO2_11` are the most useful unresolved pins to trace next. The stock image demonstrates deliberate use of each one, while the present board pinout does not identify the attached load.

Two are particularly strong findings:

- `PIO0_8` is not merely configured as a generic output. It is muxed into the same CT16B0 PWM subsystem as the verified `PIO0_9` heater output and receives application-controlled match updates.
- `PIO1_1` is not merely placed in analog mode. The stock application repeatedly cycles ADC conversion between the `PIO1_0` and `PIO1_1` channels and maintains measurement state for both.

## Pins only referenced by dormant generic driver branches

These pins occur in compiled LPC11xx timer-driver code, but the stock application does not select the branch that would configure them. They should not be treated as evidence that the Da Vinci Jr board actively uses these pins.

| LPC pin | Package pin | Dormant reference |
| --- | ---: | --- |
| `PIO0_10/SWCLK` | 29 | Alternate CT16B0 PWM/match pin option in the generic timer helper. |
| `PIO1_4` | 40 | Alternate CT32 timer/capture pin option. |
| `PIO1_8` | 9 | CT16B1 capture/input configuration path. |
| `PIO1_9` | 17 | CT16B1 PWM/match output path. |

## Known board pins not observed in the stock image

The current hardware pinout contains two verified board connections that this `+2.2.0` image does not appear to reference:

- `PIO2_10` / package pin 25—verified D18 status LED.
- `PIO3_1` / package pin 37—verified NFC-board connection.

Their absence from this image does not invalidate the hardware measurements. It only means this firmware audit does not provide independent evidence for their use.

## Confidence rules

Descriptions marked verified above come from the existing hardware pinout. Firmware-only interpretations are intentionally narrower:

- peripheral function is considered strong when established directly by IOCON/MMIO setup and the recovered LPC11xx driver;
- application role is considered strong when command/data flow reaches a known output or sampled input;
- exact board-net names remain unknown when there is no independent PCB/electrical evidence.

This distinction is especially important for `PIO3_0`. The firmware strongly supports a reset/enable-like startup pulse, and the board connection to NFC is independently verified. The exact NFC signal name has not yet been electrically established.
