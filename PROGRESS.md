## Current progress

I have dumped the firmware of the 2. mpu, but i now need to trace all the pins and cables to the mpu's and connectors to be able to continue at all. the problem is, my multimeter is dead and im stuck unable to measure anything, aka not being able to continue at all.

Will continue when i figure out a way to power the multimeter.

Until then i will continue by fixing the broken links and update the docs.

### Update 26.08.2026 13:30 GMT+3:

I got a battery and now im back in business!
Going to trace the sub board first as it is the easiest to trace so far

### Update 26.08.2026 15:00 GMT+3:

I need a better way to note my pins. Having them spread out makes it A LOT harder to keep track of them.

I have decided to desolder the LCM since i cant trace cables under it easily. Ill probably end up breaking something but idonno

### Update 26.08.2026 18:45 GMT+3:

I have finished tracing the sub board and drawing the KiCAD schematic. I am tired and going to continue tonight/tomorrow.

### Update 27.08.2026 04:40 GMT+3:

I drew the hotend board and almost finished it, and started working on the main board drawings. At the same time im also cleaning up the repo structure. Its too lonely working like this but somehow i dont feel it

Tomorrow i will continue on working on the secondary MCU and its dumped firmware.

### Update 27.08.2026 14:30 GMT+3:

I will proceed by decompiling the secondary mcu firmware.

### Update 28.08.2026 18:54 GMT+3:

I decompiled the secondary mcu firmware, but im realizing i first need to have some idea on which pins might be for what functions.

### Update 31.08.2026 19:00 GMT+3:

I stopped decompiling work, and continued with pinout tracing. So far we have all the essential pins traced (including buttons/lcd/sd), but i want to trace everything before creating a schematic. The LPC MCU is also easily flashable via UART and SWD, which will make things easier.

### Update 31.08.2026 20:00 GMT+3:

Due to the split MCU design, klipper support looks hard without hardware modding. I will continue by searching about RRF instead.

### Update 31.08.2026 23:20 GMT+3:

I need to fork RRF since the dual mcu stuff, and write custom firmware for the LPC chip and manage it over uart.

### Update 01.09.2026 00:57 GMT+3:

Figured out there are fuses on the main board. There were 2 separate 12v lanes and i was confused. first one is unfused, goes to the motor drivers current sense pin? weird. Other one is fused, goes to motors. there is also a hotend heater fuse, but its separate.

There are 4 fuses so far from what i can tell. One for heater, one for reflow fan (maybe mine is popped?) one for motors inputs.

This fixed me being stuck about the schematics, as i did not know what was some of the main power lanes were connected to. Now i got a bigger problem about marking power lanes or drawing them all fully. ig ill end up drawing them all and only labeling the raw one as 12v.

Update: okay they are all fused, i was just blind. on a side note, all my fuses seem to be intact.

### Update 01.09.2026 22:00 GMT+3:

Its getting tiring to trace everything manually on the main board since its too big to keep track of, especially since the high amount of vias. I keep losing track of which pin goes where, especially power pins, which makes it look like stuff are getting power from some divine place or something, i cant find the source of the power. I will continue tracing by creating a serial firmware to poke at the pins. For now ill do something basic that prints the triggered pins to serial, in the future i can make some firmata style interface or a firmata fork for it if i need to.

### Update 02.09.2026 13:00 GMT+3:

I checked the klipper source, firmata source and RRF source and decided to take the klipper gpio and uart code, take their linker and compiling scripts, gut klipper specific stuff from them and implement firmata on it. It will take a while probably since im not too familiar with raw C toolchain and their flags.

### Update 05.09.2026 14:50 GMT+3:

For the last 2 days, I offloaded the firmware writing part to LLMs, and so far its going well. Firstly I wrote a python GUI for the host side, then used Claude to improve it a bit, then ChatGPT decided to optimize it (it was taking 3+ minutes to start and render all the pins, now its <2 seconds.).

Then GPT took my protocol description and wrote a C firmware for the printer side. I want to thank @henmalib for most of this here. Then again, thanks to @henmalib, GPT rewrote the firmware and the GUI in rust, and its a lot cleaner and faster. We also improved the GUI and the protocol a lot, fixed dropped packets and more. Currently GPT is working on a uart passthrough for lpc to sam chip so i can control both of them over 1 uart line with no soldering extra wires.

Currently I mapped almost all the pins for the sam4e8e (missing the flash chip and the buzzer, maybe some leds), waiting for the lpc to sam chip uart passthrough so I can map the rest. I mapped enough pins for an interactive control system (endstops, buttons, motors etc all mapped), mainly missing the hotend and LCD for the rest.

Later on, I will also add a ESP12 chip to the board so I can use the RRF webui.
