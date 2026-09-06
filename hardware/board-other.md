# Other Hardware

The remaining components that do not belong to a board, but are important for the project are documented here.

There are 4 steppers, 3 endstops, 1 filament sensor on the extruder motor, and another on the hotend.

**Current known components:**

- [Stepper Motors](steppers.md)
- [Sensors](sensors.md)
- [RFID reader](rfid-reader.md)
- Top light bar
- Reflow fan

Top light bar connects to **pin PD23**. It is active-high.

## Lights

### Pinout

| Pin | Name | Description | MCU  | Verified? |
| --- | ---- | ----------- | ---- | --------- |
| 01  | -    | TopLamp     | PD23 | ✅        |
| 01  | -    | BottomLamp  | PD27 | ✅        |
