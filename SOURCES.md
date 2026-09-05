# Sources

I have used a lot of different resources and previous researches/projects in this repo, and im trying to document all of them here. If i have missed any, an issue or a pull request is welcome.

I want to thank everyone who has contributed to this project knowingly or unknowingly in the past.

## Photos

- [Teardown (YouTube)](https://www.youtube.com/watch?v=cn2mYWmanlk) for most of the external photos
- [<UndefinedDeluxe> (Soliforum)](https://www.soliforum.com/post/135251/#p135251) for the hotend board photo
- [Pilotkid2015 (Reddit)](https://www.reddit.com/r/3Dprinting/comments/3qrdu0/3f1j0_xyz_davinci_jr_nfc_reader_with_arduino/) for some of the RFID reader photos
- The close-up component/board photos are from my own unit

## Pinouts

- <a name="luc-main-board"></a> **[Luc (Soliforum)](https://www.soliforum.com/post/131637/#p131637)** for most of the main board pinouts
- <a name="julialongtin-repo"></a> **[Julialongtin (GitHub)](https://github.com/julialongtin/Davinci_Jr_Hacking)** for organizing the original pinouts
- <a name="pyr0ball-sub-board"></a> **[Pyr0ball (Soliforum)](https://www.soliforum.com/post/138269/#p138269)** for the sub board pinouts
- <a name="megatron-hotend-flex"></a> **[Megatron (Soliforum)](https://www.soliforum.com/post/135347/#p135347)** for the hotend board flex connections
- <a name="pyr0ball-schematic"></a> **[Pyr0ball (GitHub)](https://github.com/Duet3D/RepRapFirmware/issues/190#issuecomment-403314752)** for the partial schematic that enabled me to dump LPC1115 firmware

## Datasheets

- <a name="datasheet-atmel-sam4e8e"></a> **[Atmel SAM4E Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/Atmel-11157-32-bit-Cortex-M4-Microcontroller-SAM4E16-SAM4E8_Datasheet.pdf)**
- <a name="datasheet-nxp-lpc111x"></a> **[NXP LPC111x Datasheet](https://www.nxp.com/docs/en/data-sheet/LPC111X.pdf)**
- <a name="datasheet-toshiba-tb62269ftg"></a> **[Toshiba TB62269FTG Datasheet](https://toshiba.semicon-storage.com/info/TB62269FTG_datasheet_en_20140318.pdf?did=14719&prodName=TB62269FTG)**
- <a name="datasheet-macronix-mx25l3206e"></a> **[Macronix MX25L3206E Datasheet](https://www.mxic.com.tw/Lists/Datasheet/Attachments/8616/MX25L3206E,%203V,%2032Mb,%20v1.5.pdf)**
- <a name="datasheet-winstar-wh1604a"></a> **[Winstar WH1604A 16x04 LCD Module Datasheet](https://www.winstar.com.tw/uploads/files/a1a569d2ea5185895d028815d76787c9.pdf)**
- <a name="datasheet-nxp-pn512"></a> **[NXP PN512 Datasheet](https://www.nxp.com/docs/en/data-sheet/PN512.pdf)**

## Extras

- <a name="datasheet-nxp-an10968"></a> **[NXP Application Note AN10968](https://www.nxp.com/docs/en/application-note/AN10968.pdf)**
- <a name="datasheet-nxp-um10398"></a> **[NXP User Manual UM10398](https://www.usr.cn/Uploads/Attach/201010/user.manual.lpc11xx.lpc11cxx.pdf)** (3. party link due to NXP having the link behind a login, the original can be found here: [NXP UM10398](https://www.nxp.com/webapp/Download?colCode=UM10398&location=null))

## Firmware development

- **[atsam4-rs](https://github.com/atsam4-rs/atsam4-hal)**: HAL and peripheral definitions used by the Rust SAM4E8E firmware for clock setup and USB UDP support.
- **[Klipper](https://github.com/klipper3d/klipper)**: reference for the original SAM4E8E clock, GPIO, and USB CDC behavior that the Rust implementation replaces.
- **[Atmel Software Framework](https://github.com/avrxml/asf)**: cross-reference for SAM4E8E device definitions, memory layout, and clock/USB behavior.

## Firmware Archives

- <a name="archive-archiveorg"></a> **[Da Vinci Jr. Firmware Archive (Archive.org)](https://archive.org/details/da-vinci-jr-firmware)**
- <a name="archive-hidrive"></a> **[XYZ Firmware Archive by modfreakz (HiDrive)](https://my.hidrive.com/share/j8ee51tz13#$/)**
