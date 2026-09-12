# SAM4E8E PIO Pinout

I have decided to list pins by PIO and not by MCU pin, since most of them are useless for our use case.

We have 5 PIOs available, PA through PE.

- PIO A: 32 pins (PA0 through PA31)
- PIO B: 15 pins (PB0 through PB14)
- PIO C: 32 pins (PC0 through PC31)
- PIO D: 32 pins (PD0 through PD31)
- PIO E: 6 pins (PE0 through PE5)

## PIO A

| Pin  | Connected to | Description | Traced? | Verified? |
| ---- | ------------ | ----------- | ------- | --------- |
| PA0  |              |             | ❌      | ❌        |
| PA1  |              |             | ❌      | ❌        |
| PA2  | Buzzer       |             | ❌      | ✅        |
| PA3  | U2 SDA       | Hotend chip | ❌      | ✅        |
| PA4  | U2 SCL       | Hotend chip | ❌      | ✅        |
| PA5  |              |             | ❌      | ❌        |
| PA6  |              |             | ❌      | ❌        |
| PA7  |              |             | ❌      | ❌        |
| PA8  |              |             | ❌      | ❌        |
| PA9  |              |             | ❌      | ❌        |
| PA10 |              |             | ❌      | ❌        |
| PA11 | U5 1         | Flash CS    | ❌      | ❌        |
| PA12 | U18 GPIO8    | Unknown     | ❌      | ✅        |
| PA13 | U18 GPIO6    | Unknown     | ❌      | ✅        |
| PA14 | U18 GPIO7    | Unknown     | ❌      | ✅        |
| PA15 |              |             | ❌      | ❌        |
| PA16 | J120 1       | WIFI Panel  | ❌      | ✅        |
| PA17 | SW5          | Enter btn   | ❌      | ✅        |
| PA18 |              |             | ❌      | ❌        |
| PA19 |              |             | ❌      | ❌        |
| PA20 |              |             | ❌      | ❌        |
| PA21 | SW2          | Down btn    | ❌      | ✅        |
| PA22 |              |             | ❌      | ❌        |
| PA23 |              |             | ❌      | ❌        |
| PA24 |              |             | ❌      | ❌        |
| PA25 | SD CD        | Card detect | ❌      | ✅        |
| PA26 | SD DAT2      | -           | ❌      | ✅        |
| PA27 | SD CS/DAT3   | CS          | ❌      | ✅        |
| PA28 | SD CMD       | MOSI        | ❌      | ✅        |
| PA29 | SD CLK       | SCK         | ❌      | ✅        |
| PA30 | SD DAT0      | MISO        | ❌      | ✅        |
| PA31 | SD DAT1      | -           | ❌      | ✅        |

## PIO B

| Pin  | Connected to | Description | Traced? | Verified? |
| ---- | ------------ | ----------- | ------- | --------- |
| PB0  |              |             |         |           |
| PB1  |              |             |         |           |
| PB2  | U18 GPIO11   | Unknown     | ❌      | ✅        |
| PB3  | SW4          | Left btn    | ❌      | ✅        |
| PB4  |              |             |         |           |
| PB5  |              |             |         |           |
| PB6  |              |             |         |           |
| PB7  |              |             |         |           |
| PB8  |              |             |         |           |
| PB9  |              |             |         |           |
| PB10 |              |             |         |           |
| PB11 |              |             |         |           |
| PB12 |              |             |         |           |
| PB13 |              |             |         |           |
| PB14 | J7 LASER L   | Left laser  | ❌      | ✅        |

## PIO C

| Pin  | Connected to | Description | Traced? | Verified? |
| ---- | ------------ | ----------- | ------- | --------- |
| PC0  | LCD DB0      | Data line   | ❌      | ✅        |
| PC1  | LCD DB1      | Data line   | ❌      | ✅        |
| PC2  | LCD DB2      | Data line   | ❌      | ✅        |
| PC3  | LCD DB3      | Data line   | ❌      | ✅        |
| PC4  | LCD DB4      | Data line   | ❌      | ✅        |
| PC5  | LCD DB5      | Data line   | ❌      | ✅        |
| PC6  | LCD DB6      | Data line   | ❌      | ✅        |
| PC7  | LCD DB7      | Data line   | ❌      | ✅        |
| PC8  | LCD R/W      | R/W line    | ❌      | ✅        |
| PC9  | E2 step      | E2 step     | ❌      | ✅        |
| PC10 | LCD K (GND)  | Backlight   | ❌      | ✅        |
| PC11 |              |             |         |           |
| PC12 |              |             |         |           |
| PC13 | LCD E        | Enable      | ❌      | ✅        |
| PC14 |              |             |         |           |
| PC15 |              |             |         |           |
| PC16 |              |             |         |           |
| PC17 | 3dLED 4      | 3D led      | ❌      | ✅        |
| PC18 | LCD RS       | RS line     | ❌      | ✅        |
| PC19 | Y endstop    | Y endstop   | ❌      | ✅        |
| PC20 | Z step       | Z step      | ❌      | ✅        |
| PC21 |              |             |         |           |
| PC22 | Y step       | Y step      | ❌      | ✅        |
| PC23 | X step       | X step      | ❌      | ✅        |
| PC24 | J8 LASER R   | Right laser | ❌      | ✅        |
| PC25 | 3D enable    | 3D enable   | ❌      | ✅        |
| PC26 |              |             |         |           |
| PC27 |              |             |         |           |
| PC28 | E1 step      | E1 step     | ❌      | ✅        |
| PC29 |              |             |         |           |
| PC30 |              |             |         |           |
| PC31 |              |             |         |           |

## PIO D

| Pin  | Connected to | Description | Traced? | Verified? |
| ---- | ------------ | ----------- | ------- | --------- |
| PD0  |              |             |         |           |
| PD1  | 3D step      | 3D step     | ❌      | ✅        |
| PD2  | 3D dir       | 3D dir      | ❌      | ✅        |
| PD3  | X enable     | X enable    | ❌      | ✅        |
| PD4  | X dir        | X dir       | ❌      | ✅        |
| PD5  | Y enable     | Y enable    | ❌      | ✅        |
| PD6  | Z enable     | Z enable    | ❌      | ✅        |
| PD7  | Z dir        | Z dir       | ❌      | ✅        |
| PD8  | X endstop    | X endstop   | ❌      | ✅        |
| PD9  | Z endstop    | Z endstop   | ❌      | ✅        |
| PD10 | J15          | Distance    |         | ✅        |
| PD11 |              |             |         |           |
| PD12 |              |             |         |           |
| PD13 | E2 enable    | E2 enable   | ❌      | ✅        |
| PD14 |              |             |         |           |
| PD15 | 3dLED 3      | 3D led      | ❌      | ✅        |
| PD16 | E1 enable    | E1 enable   | ❌      | ✅        |
| PD17 | E1 dir       | E1 dir      | ❌      | ✅        |
| PD18 | E2 dir       | E2 dir      | ❌      | ✅        |
| PD19 |              |             |         |           |
| PD20 |              |             |         |           |
| PD21 | J120 4       | WIFI Panel  | ❌      | ✅        |
| PD22 |              |             |         |           |
| PD23 | J5           | Top lamp    | ❌      | ✅        |
| PD24 | U18 GPIO26   | Unknown     | ❌      | ✅        |
| PD25 |              |             |         |           |
| PD26 |              |             |         |           |
| PD27 | J119         | Bottom lamp | ❌      | ✅        |
| PD28 |              |             |         |           |
| PD29 | J24 1        | Top home    |         | ✅        |
| PD30 | SW6          | Home btn    | ❌      | ✅        |
| PD31 |              |             |         |           |

## PIO E

| Pin | Connected to | Description | Traced? | Verified? |
| --- | ------------ | ----------- | ------- | --------- |
| PE0 |              |             |         |           |
| PE1 | SW1          | Up btn      | ❌      | ✅        |
| PE2 | Y dir        | Y dir       | ❌      | ✅        |
| PE3 | U18 GPIO15   | Unknown     | ❌      | ✅        |
| PE4 | SW3          | Right btn   | ❌      | ✅        |
| PE5 |              |             |         |           |
