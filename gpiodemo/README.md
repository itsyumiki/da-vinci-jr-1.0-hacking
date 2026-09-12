# GPIO controller

The GPIO controller is a Rust workspace with one shared wire-protocol crate and separate firmware and desktop programs:

```text
gpiodemo/
├── protocol/   no_std packet types plus ASCII encode/decode
├── firmware/   shared no_std node/GPIO logic plus SAM4E8E and LPC1115 targets
└── gui/        iced desktop controller
```

The desktop app and firmware both use `da-vinci-protocol`. Neither side has separate packet formatting or parsing code.

## Build and run

From the repository root:

```sh
just build       # build SAM4E8E firmware and emit build/firmware.bin
just build-lpc   # build LPC1115 firmware and emit build/lpc1115.bin
just gui         # run the desktop controller
just gui-release # run an optimized desktop build for performance testing
just check       # formatting, tests, clippy, and both firmware-target checks
```

Install the Rust `thumbv7em-none-eabi` target for the SAM4E8E build and `thumbv6m-none-eabi` for the LPC1115 build. Firmware binary generation also needs `arm-none-eabi-objcopy`. `just flash` uses BOSSA for the SAM target.

`just flash` flashes `build/firmware.bin` with BOSSA. Set `DEVICE` to override the default serial device.

The desktop controller also runs natively on macOS. From the repository root, run:

```sh
cargo run --manifest-path gpiodemo/Cargo.toml -p da-vinci-gui
```

Running the GUI does not require firmware cross-compilation tools. If you have `just`, `just gui` runs the same command. Use `just gui-release` when measuring desktop performance. The firmware uses its own size-optimized release profile.

## LPC1115 assumptions and safety

The LPC1115 target passes compile and protocol tests, but no one has electrically tested it on the printer board. The checked-in schematic supplies the 48-pin package mapping. The firmware keeps RESET/PIO0_0, SWCLK/PIO0_10, SWDIO/PIO1_3, and the PIO1_6/PIO1_7 UART link unavailable to ordinary GPIO control.

The LPC pin map advertises the GPIO operations supported by the MCU so the tool can characterize the printer board. Capability bits do not claim that driving an unknown printer net is electrically safe. Ordinary non-reserved LPC pads support input, output, and internal pull-up control. PIO0_4 and PIO0_5 are special open-drain/I2C pads. They support input and output but not the normal internal pull-up mode. An output HIGH releases either line instead of sourcing it, so the board needs an external pull-up for a measured HIGH. The adapter selects Standard-I/O mode for these two pads. RESET/PIO0_0, SWCLK/PIO0_10, SWDIO/PIO1_3, and the PIO1_6/PIO1_7 UART link remain unavailable while reserved by the firmware.

The LPC firmware assumes the reset-default 12 MHz internal RC oscillator and configures its upstream UART for approximately 115200 baud. The SAM target configures UART1 on PA5/PA6 at 115200 baud and routes `LPC` frames through that link. Compile-time and fake-link tests cover this path, but the code does not claim physical validation of the SAM-to-LPC connection. Hardware measurements can require different clock or baud settings.

## Protocol

Packets are newline-delimited ASCII with a host-allocated three-digit request ID and an explicit route/source token. For example, the host sends `001 SAM HAI` and the SAM node replies `001 SAM HII <3`.

See [`protocol.md`](protocol.md) for the complete wire contract, including commands, responses, GPIO target syntax, grouped operations, listener lifetime, errors, and reset behavior.
