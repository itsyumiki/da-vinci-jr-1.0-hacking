device := env("DEVICE", "/dev/tty.usbmodem101")
manifest := "gpiodemo/Cargo.toml"
sam_target := "thumbv7em-none-eabi"
lpc_target := "thumbv6m-none-eabi"
firmware_elf := "gpiodemo/target/" + sam_target + "/firmware/da-vinci-firmware"
lpc_firmware_elf := "gpiodemo/target/" + lpc_target + "/firmware/da-vinci-lpc1115"
objcopy := env("OBJCOPY", "arm-none-eabi-objcopy")
size := env("SIZE", "arm-none-eabi-size")

default:
    @just --list

build:
    cargo build --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-firmware --no-default-features --features sam4e8e --profile firmware --target {{ sam_target }}
    mkdir -p build
    cp {{ firmware_elf }} build/firmware.elf
    {{ objcopy }} -O binary build/firmware.elf build/firmware.bin
    {{ size }} build/firmware.elf

build-lpc:
    cargo build --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-lpc1115 --no-default-features --features lpc1115 --profile firmware --target {{ lpc_target }}
    mkdir -p build
    cp {{ lpc_firmware_elf }} build/lpc1115.elf
    {{ objcopy }} -O binary build/lpc1115.elf build/lpc1115.bin
    {{ size }} build/lpc1115.elf

gui:
    cargo run --manifest-path {{ manifest }} -p da-vinci-gui

gui-release:
    cargo run --manifest-path {{ manifest }} -p da-vinci-gui --release

check:
    cargo fmt --manifest-path {{ manifest }} --all -- --check
    cargo test --manifest-path {{ manifest }} --workspace --all-features
    cargo clippy --manifest-path {{ manifest }} --workspace --all-targets --all-features -- -D warnings

verify: check
    cargo build --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-firmware --no-default-features --features sam4e8e --profile firmware --target {{ sam_target }}
    cargo build --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-lpc1115 --no-default-features --features lpc1115 --profile firmware --target {{ lpc_target }}
    cargo clippy --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-firmware --no-default-features --features sam4e8e --profile firmware --target {{ sam_target }} -- -D warnings
    cargo clippy --manifest-path {{ manifest }} -p da-vinci-firmware --bin da-vinci-lpc1115 --no-default-features --features lpc1115 --profile firmware --target {{ lpc_target }} -- -D warnings

flash file="build/firmware.bin":
    bossac --port={{ device }} -e -w -v -b {{ file }}

clean:
    cargo clean --manifest-path {{ manifest }}
    rm -rf build
