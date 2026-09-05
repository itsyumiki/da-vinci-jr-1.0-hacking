# Da Vinci Jr. 1.0 Firmware

The built-in firmware is fragmented across multiple chips (not exactly fragmented, but all the parts need to exist to be fully functional), and so dumping the firmware is **strongly recommended** if you plan to work on the printer.

**WARNING**: Erasing the firmware of the main MCU will make the hotend heat uncontrolled! It is strongly advised to unplug everything from the board before erasing its firmware.

## Dumping the Official Firmware

There are 4 chips that we can dump the firmware of:

- [SAM4E8E (main mcu)](firmware/dumping-sam4e8e.md)
- [LPC1115 (secondary mcu)](firmware/dumping-lpc1115.md)
- [MX25L3206E (flash memory on main board)](firmware/dumping-mx25l3206e.md)
- [AT24C02D (EEPROM on hotend)](firmware/dumping-at24c02d.md)

Dumping before doing anything is very strongly suggested, since this is a very experimental project and any mistakes can be very difficult to recover from. Having a working dump for each chip is essential for debugging and recovery.

**An archive of official firmware dumps can be found [here](sources.md#archive-archiveorg).**

## Decompiling the Firmware

You can use [Ghidra](https://github.com/nationalsecurityagency/ghidra) to decompile the official firmware.

Current progress on official firmware:

| Chip       | Status | Description                                                                               |
| ---------- | ------ | ----------------------------------------------------------------------------------------- |
| SAM4E8E    | 0% (?) | I have no idea if anyone decompiled it yet. I only dumped it and did some basic analysis. |
| LPC1115    | 20%    | Started decompiling, currently mapping the entry points and functions.                    |
| MX25L3206E | 0%     | Not started yet and no dump available.                                                    |
| AT24C02D   | 0%     | Not started yet and no dump available.                                                    |

## Flashing New Firmware

I will create a flashing section for per-chip guide on flashing them, after finishing the dumping section. Until then, you can use the flashing tool mentioned in the [dumping section](firmware/dumping-sam4e8e.md).

## Possible Firmware Options

I separated this section as:

- Official Firmware
- Demo Firmware
- Unofficial Firmware

Official firmware is the one the printer ships with. Demo are ones that I write for testing, like for testing a single motor or a sensor. Unofficial ones are Klipper, RepRapFirmware, Marlin, etc.

### Official Firmware

I am working on reading the decompiled official firmware for the SAM4E8E, and decompiling the LPC1115 dumps. I don't have enough to document yet, but I will update this section as I progress.

### Demo Firmware

I have successfully built and flashed multiple demo firmwares for the SAM4E8E. It is fairly easy to flash it, but building/writing code for it is torturing.

If you are interested in writing or building a demo firmware, let me warn you: **Do NOT use MPLAB X/IDE**. Despite Atmel advertising it as supporting SAM4E series, SAM4E support is deprecated.

Instead, you can use:

- [Atmel/Microchip Studio](https://www.microchip.com/en-us/tools-resources/develop/microchip-studio) if you want a full and easier IDE experience. Windows only, and I couldn't get it to run under Wine due to Visual Studio Shell requirements.
- [Atmel Software Framework](https://github.com/avrxml/asf) if you want to use a more barebones approach. You need to write your own Makefile etc for this. I just asked Claude to scrap some things together, but its too ugly to commit, and I might commit it only when I have a better understanding of the toolchain and when I rewrite it myself.
- The repository now includes a [Rust SAM4E8E GPIO controller](gpiodemo/README.md) using the board's USB CDC connection for host communication.

### Unofficial Firmware

There are several firmware options available for the Da Vinci Jr. 1.0 printer, and these are the ones that I am actively targeting:

- [Klipper](https://www.klipper3d.org/)
- [RepRapFirmware](https://www.reprapfirmware.org/)

I have built Klipper for the main MCU and flashed it successfully, and got the host communication working. I haven't tried building a config yet, since I want to get the full pinout first.

I have no work on RepRapFirmware yet, but it should be straightforward to build it for the main MCU since it natively supports the SAM4E8E. We still need the pinout though.
