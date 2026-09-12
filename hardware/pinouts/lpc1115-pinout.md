# LPC1115 Pinout

> **Traced** means the pin is visually traced on the PCB to both ends and at least verified multiple times (3+), or at least twice with a multimeter. **Verified** means the pin is verified to trigger the correct i/o on software.
>
> The pins marked as "?" are the ones that are connected to something, but havent been traced yet.
> The pins marked as "-" are the ones that do not have a VISIBLE connection. It does NOT mean the pin is not connected, as the chip has multiple connections going under it.

## Pinout Table

### Side 1

| Pin | Pin Desc   | Connected To | Description            | Traced? | Verified? |
| --- | ---------- | ------------ | ---------------------- | ------- | --------- |
| 01  | PIO2_6     | -            | Not connected          | ❌      | ✅        |
| 02  | PIO2_0     | -            | Not connected          | ❌      | ✅        |
| 03  | PIO0_0/RST | ?            | Unknown                | ❌      | ❌        |
| 04  | PIO0_1/ISP | ?            | Unknown                | ❌      | ❌        |
| 05  | VSS        | GND          | Ground                 | ✅      | ❌        |
| 06  | XTALIN     | Y4 1 XTAL1   | 12MHz Crystal          | ✅      | ❌        |
| 07  | XTALOUT    | Y4 2 XTAL2   | 12MHz Crystal          | ✅      | ❌        |
| 08  | VDD        | 3.3V         | 3.3V                   | ✅      | ❌        |
| 09  | PIO1_8     | -            | Not connected          | ❌      | ❌        |
| 10  | PIO0_2     | -            | Not connected          | ❌      | ❌        |
| 11  | PIO2_7/RXD | J19/J14      | Filament runout sensor | ❌      | ✅        |
| 12  | PIO2_8/TXD | ?            | Unknown                | ❌      | ❌        |

### Side 2

| Pin | Pin Desc   | Connected To | Description            | Traced? | Verified? |
| --- | ---------- | ------------ | ---------------------- | ------- | --------- |
| 13  | PIO2_1     | J19/J14      | E1 rotation sensor     | ❌      | ✅        |
| 14  | PIO0_3     | ?            | Unknown                | ❌      | ❌        |
| 15  | PIO0_4/SCL | J12/J123 6   | NFC board              | ❌      | ✅        |
| 16  | PIO0_5/SDA | J12/J123 5   | NFC board              | ❌      | ✅        |
| 17  | PIO1_9     | ?            | Unknown                | ❌      | ❌        |
| 18  | PIO3_4/RXD | ?            | Unknown                | ❌      | ❌        |
| 19  | PIO2_4     | ?            | Unknown                | ❌      | ❌        |
| 20  | PIO2_5     | J11 J2       | Hotend fan             | ❌      | ✅        |
| 21  | PIO3_5/TXD | -            | Unknown                | ❌      | ❌        |
| 22  | PIO0_6     | J11 U1       | Hotend filament sensor | ❌      | ✅        |
| 23  | PIO0_7     | ?            | Unknown                | ❌      | ❌        |
| 24  | PIO2_9     | ?            | Unknown                | ❌      | ❌        |

### Side 3

| Pin | Pin Desc      | Connected To | Description   | Traced? | Verified? |
| --- | ------------- | ------------ | ------------- | ------- | --------- |
| 25  | PIO2_10       | D18          | Status LED    | ❌      | ✅        |
| 26  | PIO2_2        | ?            | Unknown       | ❌      | ❌        |
| 27  | PIO0_8        | ?            | Unknown       | ❌      | ❌        |
| 28  | PIO0_9        | J11 J4       | Heater        | ❌      | ✅        |
| 29  | PIO0_10/SWCLK | ?            | Unknown       | ❌      | ❌        |
| 30  | PIO1_10       | J121/J122    | Reflow FAN    | ❌      | ✅        |
| 31  | PIO2_11       | -            | Not connected | ❌      | ❌        |
| 32  | PIO0_11       | ?            | Unknown       | ❌      | ❌        |
| 33  | PIO1_0        | J11 J3       | Hotend NTC    | ❌      | ✅        |
| 34  | PIO1_1        | -            | Not connected | ❌      | ❌        |
| 35  | PIO1_2        | -            | Not connected | ❌      | ❌        |
| 36  | PIO3_0/TXD    | J12/J123 4   | NFC board     | ❌      | ✅        |

### Side 4

| Pin | Pin Desc     | Connected To | Description   | Traced? | Verified? |
| --- | ------------ | ------------ | ------------- | ------- | --------- |
| 37  | PIO3_1/RXD   | J12/J123 3   | NFC board     | ❌      | ✅        |
| 38  | PIO2_3       | -            | Not connected | ❌      | ❌        |
| 39  | PIO1_3/SWDIO | ?            | Unknown       | ❌      | ❌        |
| 40  | PIO1_4       | -            | Not connected | ❌      | ❌        |
| 41  | VSS          | GND          | Ground        | ❌      | ❌        |
| 42  | PIO1_11      | ?            | Unknown       | ❌      | ❌        |
| 43  | PIO3_2       | -            | Not connected | ❌      | ❌        |
| 44  | VDD          | 3.3V         | 3.3V          | ❌      | ❌        |
| 45  | PIO1_5       | -            | Not connected | ❌      | ❌        |
| 46  | PIO1_6/RXD   | ?            | Unknown       | ❌      | ❌        |
| 47  | PIO1_7/TXD   | ?            | Unknown       | ❌      | ❌        |
| 48  | PIO3_3       | ?            | Unknown       | ❌      | ❌        |
