# ESP32 Pinout

There is a place on the main board for an ESP32-WROOM-32E module. This is its pin mapping.

Note: I started to doubt it being an ESP32 due to missing power connections.

## Side 1

| Pin | Pin Desc | Connected To | Description     | Traced? | Verified? |
| --- | -------- | ------------ | --------------- | ------- | --------- |
| 01  | GND      | GND          | Ground          | ✅      | ❌        |
| 02  | 3.3V     | -            | Not connected   | ✅      | ❌        |
| 03  | EN       | -            | Not connected   | ✅      | ❌        |
| 04  | GPIO36   | -            | Not connected   | ✅      | ❌        |
| 05  | GPIO39   | -            | Not connected   | ✅      | ❌        |
| 06  | GPIO34   | -            | Not connected   | ✅      | ❌        |
| 07  | GPIO35   | -            | Not connected   | ✅      | ❌        |
| 08  | GPIO32   | -            | Not connected   | ✅      | ❌        |
| 09  | GPIO33   | -            | Not connected   | ✅      | ❌        |
| 10  | GPIO25   | -            | Not connected   | ✅      | ❌        |
| 11  | GPIO26   | PD24         | Unknown         | ❌      | ✅        |
| 12  | GPIO27   | GND          | Ground via R289 | ✅      | ❌        |
| 13  | GPIO14   | -            | Not connected   | ✅      | ❌        |
| 14  | GPIO12   | GND          | Ground          | ✅      | ❌        |

## Side 2

There are 2 extra pins on this side, and they are not listed here as they are N/C and GND, and arent present on the ESP32.

| Pin | Pin Desc | Connected To | Description   | Traced? | Verified? |
| --- | -------- | ------------ | ------------- | ------- | --------- |
| 15  | GND      | -            | Not connected | ✅      | ❌        |
| 16  | GPIO13   | -            | Not connected | ✅      | ❌        |
| 17  | GPIO9    | -            | Not connected | ✅      | ❌        |
| 18  | GPIO10   | -            | Not connected | ✅      | ❌        |
| 19  | GPIO11   | PB02         | Unknown       | ❌      | ✅        |
| 20  | GPIO6    | PA13         | Unknown       | ❌      | ✅        |
| 21  | GPIO7    | PA14         | Unknown       | ❌      | ✅        |
| 22  | GPIO8    | PA12         | Unknown       | ❌      | ✅        |
| 23  | GPIO15   | PE03         | Unknown       | ❌      | ✅        |
| 24  | GPIO2    | -            | Not connected | ✅      | ❌        |

## Side 3

| Pin | Pin Desc | Connected To | Description   | Traced? | Verified? |
| --- | -------- | ------------ | ------------- | ------- | --------- |
| 25  | GPIO0    | J117 1       |               | ✅      | ❌        |
| 26  | GPIO4    | PA26 J118 3  | Unknown       | ✅      | ✅        |
| 27  | GPIO16   | J117 4       |               | ✅      | ❌        |
| 28  | GPIO17   | -            | Not connected | ✅      | ❌        |
| 29  | GPIO5    | J118 1       | Unknown       | ✅      | ❌        |
| 30  | GPIO18   | -            | Not connected | ✅      | ❌        |
| 31  | GPIO19   | -            | Not connected | ✅      | ❌        |
| 32  | NC       | -            | Not connected | ✅      | ❌        |
| 33  | GPIO21   | -            | Not connected | ✅      | ❌        |
| 34  | GPIO3    | -            | Not connected | ✅      | ❌        |
| 35  | GPIO1    | -            | Not connected | ✅      | ❌        |
| 36  | GPIO22   | -            | Not connected | ✅      | ❌        |
| 37  | GPIO23   | -            | Not connected | ✅      | ❌        |
| 38  | GND      | GND          | Ground        | ✅      | ❌        |
