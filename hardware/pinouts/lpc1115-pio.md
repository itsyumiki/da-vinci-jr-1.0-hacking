# LPC1115 PIO Pinout

I have decided to list pins by PIO and not by MCU pin, since most of them are useless for our use case.

We have 5 PIOs available, PIO0 through PIO3.

- PIO 0: 12 pins (PIO0_0 through PIO0_11)
- PIO 1: 12 pins (PIO1_0 through PIO1_11)
- PIO 2: 12 pins (PIO2_0 through PIO2_11)
- PIO 3: 6 pins (PIO3_0 through PIO3_5)

## PIO 0

| Pin     | Connected to | Description        | Traced? | Verified? |
| ------- | ------------ | ------------------ | ------- | --------- |
| PIO0_0  | PC15 J115 4  | Reset pin          | ❌      | ✅        |
| PIO0_1  | PC13 J115 3  | ISP pin            | ❌      | ✅        |
| PIO0_2  | -            | Not connected      | ❌      | ❌        |
| PIO0_3  | J19/J14 5    | Unknown input      | ❌      | ✅        |
| PIO0_4  | J12/J123 6   | NFC board          | ❌      | ✅        |
| PIO0_5  | J12/J123 5   | NFC board          | ❌      | ✅        |
| PIO0_6  | J11 U1       | Hotend fil. sensor | ❌      | ✅        |
| PIO0_7  | ?            | Unknown            | ❌      | ❌        |
| PIO0_8  | ?            | Unknown            | ❌      | ❌        |
| PIO0_9  | J11 J4       | Heater             | ❌      | ✅        |
| PIO0_10 | J114 5       | SWCLK              | ❌      | ✅        |
| PIO0_11 | ?            | Unknown            | ❌      | ❌        |

## PIO 1

| Pin     | Connected to | Description   | Traced? | Verified? |
| ------- | ------------ | ------------- | ------- | --------- |
| PIO1_0  | J11 J3       | Hotend NTC    | ❌      | ✅        |
| PIO1_1  | -            | Not connected | ❌      | ❌        |
| PIO1_2  | -            | Not connected | ❌      | ❌        |
| PIO1_3  | J114 2       | SWDIO         | ❌      | ✅        |
| PIO1_4  | -            | Not connected | ❌      | ❌        |
| PIO1_5  | -            | Not connected | ❌      | ❌        |
| PIO1_6  | U2 PA6       | UART RXD      | ❌      | ✅        |
| PIO1_7  | U2 PA5       | UART TXD      | ❌      | ✅        |
| PIO1_8  | -            | Not connected | ❌      | ❌        |
| PIO1_9  | ?            | Unknown       | ❌      | ❌        |
| PIO1_10 | J121/J122    | Reflow FAN 1  | ❌      | ✅        |
| PIO1_11 | ?            | Unknown       | ❌      | ❌        |

## PIO 2

| Pin     | Connected to | Description        | Traced? | Verified? |
| ------- | ------------ | ------------------ | ------- | --------- |
| PIO2_0  | -            | Not connected      | ❌      | ✅        |
| PIO2_1  | J19/J14 1    | E1 rotation sensor | ❌      | ✅        |
| PIO2_2  | ?            | Unknown            | ❌      | ❌        |
| PIO2_3  | -            | Not connected      | ❌      | ❌        |
| PIO2_4  | ?            | Unknown            | ❌      | ❌        |
| PIO2_5  | J11 J2       | Hotend fan         | ❌      | ✅        |
| PIO2_6  | -            | Not connected      | ❌      | ✅        |
| PIO2_7  | J19/J14 2    | Fil. runout sensor | ❌      | ✅        |
| PIO2_8  | J19/J14 6    | Unknown input      | ❌      | ✅        |
| PIO2_9  | ?            | Unknown            | ❌      | ❌        |
| PIO2_10 | D18          | Status LED         | ❌      | ✅        |
| PIO2_11 | J121/J122    | Reflow FAN 2       | ❌      | ✅        |

## PIO 3

| Pin    | Connected to | Description   | Traced? | Verified? |
| ------ | ------------ | ------------- | ------- | --------- |
| PIO3_0 | J12/J123 4   | NFC board     | ❌      | ✅        |
| PIO3_1 | J12/J123 3   | NFC board     | ❌      | ✅        |
| PIO3_2 | -            | Not connected | ❌      | ❌        |
| PIO3_3 | ?            | Unknown       | ❌      | ❌        |
| PIO3_4 | J12/J123 8   | NFC board     | ❌      | ✅        |
| PIO3_5 | J12/J123 9   | NFC board     | ❌      | ✅        |
