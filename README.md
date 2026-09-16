# Da Vinci Jr. 1.0 Hacking

There was a lot of efforts on reverse engineering/hacking this printer years ago, but nobody seemed to have done it fully due to various reasons. I have decided to continue over their work, and adding my own work while documenting my progress.

This repository is a mix of:

- Organization of the old community's findings
- My own work
- Official datasheets/documentation from manufacturers

I try to add source links whenever possible.

## Project Status

| Task Name  | Description                             | Status | Main Board | Sub Board | Hotend Board |
| ---------- | --------------------------------------- | ------ | ---------- | --------- | ------------ |
| Pinouts    | Trace all of the pinouts of the printer | 99%    | 99%        | 100%      | 100%         |
| Schematics | Draw the schematics of the printer      | 85%    | 30%        | 100%      | 100%         |
| Configs    | Build firmware configs for the printer  | 90%    | -          | -         | -            |

Every visible connector has been traced. There are extra SAM pins that seem to have a connection but tracing where they are connected is extremely difficult and not worth the effort as the current pinout provides a very rich set of pins for our use.

There is RepRapFirmware builds available in the [RepRapFirmware repo actions](https://github.com/itsyumiki/RepRapFirmware-for-da-vinci-jr-1.0/actions/workflows/build.yml). Keep in mind that they are still in development, and some functionality might break on some builds. You can use the `sys` folder in the same repository for the default configurations.

## More Information

More information can be found in the [hardware](hardware.md) and [firmware](firmware.md) sections.

In case of doubts, refer to the KiCAD schematics for pinout information. All the pinouts in the schematics are verified, while some of the docs might be outdated or have typos.

Info about my personal progress on reverse engineering can be found in the [progress](PROGRESS.md) section.

## Other Boards

This repository is mainly for Da Vinci Jr series printers, but feel free to help extend it to other boards.

## Contributing

Any kind of contribution is welcome, whether it's a bug report, a feature request, or a pull request. I am open to any kind of feedback or suggestion, and I want to make this project better for everyone.
