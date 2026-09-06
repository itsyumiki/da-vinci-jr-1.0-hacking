# Da Vinci Jr. 1.0 Hacking

There was a lot of efforts on reverse engineering/hacking this printer years ago, but nobody seemed to have done it fully due to various reasons. I have decided to continue over their work, and adding my own work while documenting my progress.

This repository is a mix of:

- Organization of the old community's findings
- My own work
- Official datasheets/documentation from manufacturers

I try to add source links whenever possible.

## Status

| Task Name  | Description                             | Status | Main Board | Sub Board | Hotend Board |
| ---------- | --------------------------------------- | ------ | ---------- | --------- | ------------ |
| Pinouts    | Trace all of the pinouts of the printer | 99%    | 98%        | 100%      | 100%         |
| Schematics | Draw the schematics of the printer      | 85%    | 30%        | 100%      | 100%         |
| Configs    | Build firmware configs for the printer  | 0%     | -          | -         | -            |

All of the populated pinouts has been traced. Some unpopulated pins are traced as well. The status stays at 99% because I plan to populate the wifi chip (it seems to be ESP32-WROOM-32) and I haven't traced the pins for it. The current traced pins are more than enough for a fully functional firmware.

## More Information

More information can be found in the [hardware](hardware.md) and [firmware](firmware.md) sections.

In case of doubts, refer to the KiCAD schematics for pinout information. All the pinouts in the schematics are verified, while some of the docs might be outdated or have typos.

Info about my personal progress on reverse engineering can be found in the [progress](PROGRESS.md) section.

## Other Boards

This repository is mainly for Da Vinci Jr series printers, but feel free to help extend it to other boards.

## Contributing

Any kind of contribution is welcome, whether it's a bug report, a feature request, or a pull request. I am open to any kind of feedback or suggestion, and I want to make this project better for everyone.
