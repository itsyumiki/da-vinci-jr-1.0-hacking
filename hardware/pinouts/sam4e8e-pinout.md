# SAM4E8E Pinout

> **Traced** means the pin is visually traced on the PCB to both ends and at least verified multiple times (3+), or at least twice with a multimeter. **Verified** means the pin is verified to trigger the correct i/o on software.
>
> The pins marked as "?" are the ones that are connected to something, but havent been traced yet.
> The pins marked as "-" are the ones that do not have a VISIBLE connection. It does NOT mean the pin is not connected, as the chip has multiple connections going under it.

Due to the chip size, I have separated the pinout into four sections for easier readability and traceability.

## Pinout Table

### Side 1

| Pin | Pin Desc   | Connected To | Description   | Traced? | Verified? |
| --- | ---------- | ------------ | ------------- | ------- | --------- |
| 01  | PD0        | ?            | Unknown       | ❌      | ❌        |
| 02  | PD31       | ?            | Unknown       | ❌      | ❌        |
| 03  | VDDOUT     | ?            | Unknown       | ❌      | ❌        |
| 04  | PE0        | ?            | Unknown       | ❌      | ❌        |
| 05  | VDDIN      | ?            | Unknown       | ❌      | ❌        |
| 06  | PE1        | ?            | Unknown       | ❌      | ❌        |
| 07  | PE2        | -            | Not connected | ❌      | ❌        |
| 08  | GND        | ?            | Unknown       | ❌      | ❌        |
| 09  | ADVREFP    | ?            | Unknown       | ❌      | ❌        |
| 10  | PE3        | -            | Not connected | ❌      | ❌        |
| 11  | PC0        | -            | Not connected | ❌      | ❌        |
| 12  | PC27       | ?            | Unknown       | ❌      | ❌        |
| 13  | PC26       | -            | Not connected | ❌      | ❌        |
| 14  | PC31       | ?            | Unknown       | ❌      | ❌        |
| 15  | PC30       | ?            | Unknown       | ❌      | ❌        |
| 16  | PC29       | ?            | Unknown       | ❌      | ❌        |
| 17  | PC12       | ?            | Unknown       | ❌      | ❌        |
| 18  | PC15       | ?            | Unknown       | ❌      | ❌        |
| 19  | PC13       | ?            | Unknown       | ❌      | ❌        |
| 20  | PB1        | -            | Not connected | ❌      | ❌        |
| 21  | PB0        | -            | Not connected | ❌      | ❌        |
| 22  | PA20/PGMD8 | ?            | Unknown       | ❌      | ❌        |
| 23  | PA19/PGMD7 | ?            | Unknown       | ❌      | ❌        |
| 24  | PA18/PGMD6 | ?            | Unknown       | ❌      | ❌        |
| 25  | PA17/PGMD5 | -            | Not connected | ❌      | ❌        |
| 26  | PB2        | -            | Not connected | ❌      | ❌        |
| 27  | PE4        | -            | Not connected | ❌      | ❌        |
| 28  | PE5        | ?            | Unknown       | ❌      | ❌        |
| 29  | VDDCORE    | ?            | Unknown       | ❌      | ❌        |
| 30  | VDDIO      | 3.3V         | 3.3V Power    | ✅      | ❌        |
| 31  | PB3        | -            | Not connected | ❌      | ❌        |
| 32  | PA21/PGMD9 | -            | Not connected | ❌      | ❌        |
| 33  | VDDCORE    | ?            | Unknown       | ❌      | ❌        |
| 34  | PD30       | -            | Not connected | ❌      | ❌        |
| 35  | PA7/XIN32  | Y2 1 XTAL1   | 32kHz Crystal | ✅      | ❌        |
| 36  | PA8/XOUT32 | Y2 2 XTAL2   | 32kHz Crystal | ✅      | ❌        |

### Side 2

| Pin | Pin Desc    | Connected To | Description   | Traced? | Verified? |
| --- | ----------- | ------------ | ------------- | ------- | --------- |
| 37  | PA22/PGMD10 | -            | Not connected | ❌      | ❌        |
| 38  | PC1         | -            | Not connected | ❌      | ❌        |
| 39  | PC2         | -            | Not connected | ❌      | ❌        |
| 40  | PC3         | -            | Not connected | ❌      | ❌        |
| 41  | PC4         | -            | Not connected | ❌      | ❌        |
| 42  | PA13/PGMD1  | -            | Not connected | ❌      | ❌        |
| 43  | VDDIO       | 3.3V         | 3.3V Power    | ✅      | ❌        |
| 44  | GND         | ?            | Unknown       | ❌      | ❌        |
| 45  | PA16/PGMD4  | -            | Not connected | ❌      | ❌        |
| 46  | PA23/PGMD11 | ?            | Unknown       | ❌      | ❌        |
| 47  | PD27        | -            | Not connected | ❌      | ❌        |
| 48  | PC7         | -            | Not connected | ❌      | ❌        |
| 49  | PA15/PGMD3  | ?            | Unknown       | ❌      | ❌        |
| 50  | VDDCORE     | ?            | Unknown       | ❌      | ❌        |
| 51  | PA14/PGMD2  | -            | Not connected | ❌      | ❌        |
| 52  | PD25        | ?            | Unknown       | ❌      | ❌        |
| 53  | PD26        | ?            | Unknown       | ❌      | ❌        |
| 54  | PC6         | -            | Not connected | ❌      | ❌        |
| 55  | PD24        | -            | Not connected | ❌      | ❌        |
| 56  | PA24/PGMD12 | -            | Not connected | ❌      | ❌        |
| 57  | PD23        | J5 1 ?       | Top lamp      | ❌      | ✅        |
| 58  | PC5         | -            | Not connected | ❌      | ❌        |
| 59  | PA25/PGMD13 | -            | Not connected | ❌      | ❌        |
| 60  | PD22        | ?            | Unknown       | ❌      | ❌        |
| 61  | GND         | ?            | Unknown       | ❌      | ❌        |
| 62  | PA26/PGMD14 | -            | Not connected | ❌      | ❌        |
| 63  | PD21        | ?            | Unknown       | ❌      | ❌        |
| 64  | PA11/PGMM3  | -            | Not connected | ❌      | ❌        |
| 65  | PD20        | ?            | Unknown       | ❌      | ❌        |
| 66  | PA10/PGMM2  | ?            | Unknown       | ❌      | ❌        |
| 67  | PD19        | J37 ? ?      | Z endstop     | ❌      | ❌        |
| 68  | PA12/PGMD0  | -            | Not connected | ❌      | ❌        |
| 69  | PD18        | ?            | Unknown       | ❌      | ❌        |
| 70  | PA27/PGMD15 | -            | Not connected | ❌      | ❌        |
| 71  | PD28        | ?            | Unknown       | ❌      | ❌        |
| 72  | VDDIO       | 3.3V         | 3.3V Power    | ✅      | ❌        |

### Side 3

| Pin | Pin Desc    | Connected To  | Description     | Traced? | Verified? |
| --- | ----------- | ------------- | --------------- | ------- | --------- |
| 73  | PA5/PGMRDY  | -             | Not connected   | ❌      | ❌        |
| 74  | PD17        | U14 44 CW/CCW | E1 motor dir    | ❌      | ✅        |
| 75  | PA9/PGMM1   | ?             | Unknown         | ❌      | ❌        |
| 76  | PC28        | U14 2 CLK_IN  | E1 motor step   | ❌      | ✅        |
| 77  | PA4/PGMNCMD | -             | Not connected   | ❌      | ❌        |
| 78  | PD16        | U14 3 ENABLE  | E1 motor enable | ❌      | ✅        |
| 79  | PB6         | ?             | Unknown         | ❌      | ❌        |
| 80  | VDDIO       | ?             | Unknown         | ❌      | ❌        |
| 81  | VDDCORE     | ?             | Unknown         | ❌      | ❌        |
| 82  | PC8         | -             | Not connected   | ❌      | ❌        |
| 83  | NRST        | ?             | Unknown         | ❌      | ❌        |
| 84  | PD14        | -             | Not connected   | ❌      | ❌        |
| 85  | TEST        | ?             | Unknown         | ❌      | ❌        |
| 86  | PC9         | -             | Not connected   | ❌      | ❌        |
| 87  | PB12        | ?             | Unknown         | ❌      | ❌        |
| 88  | PD13        | -             | Not connected   | ❌      | ❌        |
| 89  | PB7         | ?             | Unknown         | ❌      | ❌        |
| 90  | PC10        | -             | Not connected   | ❌      | ❌        |
| 91  | PA3         | -             | Not connected   | ❌      | ❌        |
| 92  | PD12        | ?             | Unknown         | ❌      | ❌        |
| 93  | PA2         | -             | Not connected   | ❌      | ❌        |
| 94  | PC11        | ?             | Unknown         | ❌      | ❌        |
| 95  | GND         | ?             | Unknown         | ❌      | ❌        |
| 96  | VDDIO       | ?             | Unknown         | ❌      | ❌        |
| 97  | PC14        | ?             | Unknown         | ❌      | ❌        |
| 98  | PD11        | -             | Not connected   | ❌      | ❌        |
| 99  | PA1/PGMEN1  | ?             | Unknown         | ❌      | ❌        |
| 100 | PC16        | ?             | Unknown         | ❌      | ❌        |
| 101 | PD10        | ?             | Unknown         | ❌      | ❌        |
| 102 | PA0/PGMEN0  | ?             | Unknown         | ❌      | ❌        |
| 103 | PC17        | ?             | Unknown         | ❌      | ❌        |
| 104 | JTAGSEL     | ?             | Unknown         | ❌      | ❌        |
| 105 | PB4         | ?             | Unknown         | ❌      | ❌        |
| 106 | PD15        | ?             | Unknown         | ❌      | ❌        |
| 107 | VDDCORE     | ?             | Unknown         | ❌      | ❌        |
| 108 | PD29        | ?             | Unknown         | ❌      | ❌        |

### Side 4

| Pin | Pin Desc   | Connected To  | Description    | Traced? | Verified? |
| --- | ---------- | ------------- | -------------- | ------- | --------- |
| 109 | PB5        | ?             | Unknown        | ❌      | ❌        |
| 110 | PD9        | J37 ? ?       | Y endstop      | ❌      | ❌        |
| 111 | PC18       | -             | Not connected  | ❌      | ❌        |
| 112 | PA28       | -             | Not connected  | ❌      | ❌        |
| 113 | PD8        | J37 ? ?       | X endstop      | ❌      | ❌        |
| 114 | PA6/PGMNOE | -             | Not connected  | ❌      | ❌        |
| 115 | GND        | -             | Not connected  | ❌      | ❌        |
| 116 | PA30       | -             | Not connected  | ❌      | ❌        |
| 117 | PC19       | ?             | Unknown        | ❌      | ❌        |
| 118 | PA31       | -             | Not connected  | ❌      | ❌        |
| 119 | PD7        | U12 44 CW/CCW | Z motor dir    | ❌      | ✅        |
| 120 | PC20       | U12 2 CLK_IN  | Z motor step   | ❌      | ✅        |
| 121 | PD6        | U12 3 ENABLE  | Z motor enable | ❌      | ✅        |
| 122 | PC21       | ?             | Unknown        | ❌      | ❌        |
| 123 | VDDCORE    | ?             | Unknown        | ❌      | ❌        |
| 124 | PC22       | U11 2 CLK_IN  | Y motor step   | ❌      | ✅        |
| 125 | PD5        | U11 3 ENABLE  | Y motor enable | ❌      | ✅        |
| 126 | PD4        | U10 44 CW/CCW | X motor dir    | ❌      | ✅        |
| 127 | PC23       | U10 2 CLK_IN  | X motor step   | ❌      | ✅        |
| 128 | PD3        | U10 3 ENABLE  | X motor enable | ❌      | ✅        |
| 129 | PA29       | -             | Not connected  | ❌      | ❌        |
| 130 | PC24       | -             | Not connected  | ❌      | ❌        |
| 131 | PD2        | -             | Not connected  | ❌      | ❌        |
| 132 | PD1        | -             | Not connected  | ❌      | ❌        |
| 133 | PC25       | -             | Not connected  | ❌      | ❌        |
| 134 | VDDIO      | ?             | Unknown        | ❌      | ❌        |
| 135 | GND        | ?             | Unknown        | ❌      | ❌        |
| 136 | PB10       | ?             | Unknown        | ❌      | ❌        |
| 137 | PB11       | ?             | Unknown        | ❌      | ❌        |
| 138 | GND        | ?             | Unknown        | ❌      | ❌        |
| 139 | VDDPLL     | ?             | Unknown        | ❌      | ❌        |
| 140 | PB14       | -             | Not connected  | ❌      | ❌        |
| 141 | PB8/XOUT   | Y1 1 XTAL1    | 12MHz Crystal  | ✅      | ❌        |
| 142 | PB9/XIN    | Y1 2 XTAL2    | 12MHz Crystal  | ✅      | ❌        |
| 143 | VDDIO      | 3.3V          | 3.3V Power     | ✅      | ❌        |
| 144 | PB13       | ?             | Unknown        | ❌      | ❌        |

- Pin Source: [Atmel SAM4E Datasheet](../../SOURCES.md#datasheet-atmel-sam4e8e)
