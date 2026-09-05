# GPIO Controller UI Design

This file is the implementation reference for the Rust GPIO controller UI. It is intentionally practical rather than a generic design-system document. When the current UI conflicts with this file, this file wins.

The visual baseline is the existing Python GPIO demo: compact desktop controls, dark neutral surfaces, native-looking selectors/buttons, and a resizable GPIO/log split. The Rust UI should feel like the Python tool rebuilt cleanly, not like a custom web dashboard.

## Non-negotiable invariants

1. **Nothing in a pin row moves when state changes.**
   - LOW → HIGH must not move anything.
   - Listen → Stop must not move anything outside that button's own intrinsic width.
   - INPUT → OUTPUT must not move later columns.
   - Pending → complete must not move anything.

2. **Do not manually calculate text centering inside buttons.**
   - Let the widget lay out and center its text.
   - Horizontal padding must be symmetric.
   - Use the same proportional cell shares for every row when the layout needs alignment.

3. **Buttons are native-looking and mostly neutral.**
   - Do not make every action a bright Iced-primary button.
   - Match the Python/ttk feel as closely as Iced allows.
   - A button's width should follow its label and native padding.

4. **GPIO identity is physical-first and route-native.**
   - GUI: use the discovered native pin token and package pin when present, for example `PB12 (87)` or `PIO2_3 (38)`.
   - Wire protocol: use the discovered symbolic target token. Never substitute a virtual numeric GPIO ID.
   - Never show virtual numeric GPIO IDs to the user.

5. **Route-local banks are the navigation model.**
   - The user selects a configured route first, then navigates the banks discovered for that route.
   - No arbitrary `Page 2 of 4` navigation.

6. **The serial log remains visible beside the GPIO view and the divider is resizable.**

---

## Visual direction

### General feel

Use the Python UI as the visual reference:

- dense desktop utility, not spacious dashboard UI;
- dark neutral window/background;
- subtle panel separation;
- native-looking dropdowns and buttons;
- compact control heights;
- small consistent gaps;
- no decorative cards, gradients, pill buttons, or unnecessary shadows;
- monospace only for the serial log/raw command entry.

Iced cannot provide literal `ttk` widgets. The goal is visual and behavioral parity, not pixel-identical OS widget rendering.

### Approximate dark palette

Use these as reference values rather than mandatory one-off constants scattered through the code:

| Role | Reference |
|---|---|
| Window background | `#242424` |
| Main panel background | `#2B2B2B` |
| Raised/selected neutral | `#333333` |
| Input background | `#343638` |
| Input border | `#565B5E` |
| Hairline/divider | `#474747` |
| Main text | `#DCE4EE` |
| Muted/disabled text | approximately `#B0B0B0` |
| HIGH state | green, approximately `#3DDC97` |
| LOW state | medium/dark gray |
| Unknown/unset state | slightly lighter gray than the panel |
| Destructive text | red |

Normal action buttons should stay neutral. The most visually obvious color in the pin table should be the **level state**, not the controls.

### Spacing and density

Use a compact 4/8 px rhythm.

- row height: about 36 px;
- related controls: 4–8 px gap;
- larger groups: 8–16 px gap;
- panel padding: about 8–12 px;
- button height: approximately 28–30 px;
- default window target: approximately 1250 × 800, still fully resizable.

Do not invent unrelated 13 px / 17 px / 23 px spacing values to repair individual alignments.

---

## Window structure

The app has two main panes separated by a draggable vertical splitter:

```text
┌──────────────────────── GPIO / controls ───────────────────────┬──────── Serial log ────────┐
│                                                               │                             │
│                                                               │                             │
│                                                               │                             │
└───────────────────────────────────────────────────────────────┴─────────────────────────────┘
```

The Python split ratio is a good starting point: roughly 11:4 in favor of GPIO controls.

The user must be able to drag the divider. Within a visible pin block, resizing can change cell widths proportionally, but every row and its header must keep the same column shares.

If a GPIO pane becomes too narrow for two pin blocks side by side, stack the blocks vertically in an internal scroll area. Do not squeeze controls until they overlap. The serial-log pane must retain a usable minimum width as well.

The serial log keeps:

- selectable text;
- timestamps toggle;
- auto-scroll toggle;
- clear action;
- raw command field;
- command history;
- send action.

---

## GPIO bank navigation

The connection area includes a route selector. Changing the route changes only the route-local discovered state that the UI renders. It must not reconnect the serial device or reset another route's modes, values, listeners, or pending requests.

Below it, use a tab-like selector built from the selected route's discovered banks. For the SAM route, retain the established presentation grouping:

```text
[ ‹ ]   PIOA   PIOB + PIOE   PIOC   PIOD   [ › ]
```

Routes without a presentation override render one tab per discovered bank using the native bank labels, such as `PIO0`, `PIO1`, `PIO2`, and `PIO3`.

Rules:

- each tab is directly clickable;
- left/right arrow buttons switch one tab at a time;
- arrows stop at the first/last tab and disable there;
- no wrap-around;
- tab rows can wrap when needed instead of overflowing the pane.
- changing the visible tab **must not** change the bulk-action scope dropdown.
- changing the selected route rebuilds tabs and scopes from that route's discovered map.
- the selected tab must remain visually obvious without changing its geometry.

### Bank layouts

A discovered bank with pins in both lower and upper halves can render as two columns. Smaller banks render as one column with internal scrolling. The renderer derives rows from discovered `BankKey` and pin metadata rather than MCU-name branches.

For SAM, `PIOB + PIOE` is a **listing-only merge**. The two discovered banks remain visually and semantically separate:

```text
PIOB                                      PIOE
Pin ...                                   Pin ...
PB0 (...)                                 PE0 (...)
PB1 (...)                                 PE1 (...)
...
```

Do not create a `PIOB + PIOE` protocol target or bulk scope. The merge is presentation configuration only. Routes without an override render their discovered banks generically.

---

## Pin row

Every normal pin row has exactly five cells:

```text
Pin | Mode | Level | Read/Write | Listen/Stop
```

Every row in a visible pin block uses the same proportional five-column layout. State changes can change cell contents, but they must not change the column shares or shift later cells relative to the other rows.

### Pin cell

Display:

```text
PB12 (87)
```

Where:

- `PB12` is the MCU PIO pin name;
- `87` is the physical package pin;
- there is no virtual/wire ID anywhere in the visible label.

GUI display names do **not** need zero padding: `PA1`, not `PA01`.

Reserved/unavailable pins may stay visible so the listing matches the MCU, but they are not configurable.

### Mode cell

Use a native-looking dropdown with `INPUT`, `IN_PULLUP`, and `OUTPUT`. `UNSET` is the initial/read-only state shown as the dropdown placeholder; it is not a selectable mode because the protocol has no per-pin "unset" operation.

During an in-flight mode change:

- keep the current text;
- disable the dropdown in place;
- do not replace it with `Setting...`;
- do not resize the cell.

### Level cell

The level is a stable status box, not plain colored text. Its width follows the level-column share and its height stays constant.

States:

- `HIGH` → green background;
- `LOW` → gray background;
- unknown/unset → darker neutral background with `—`;
- pending read/write → same geometry, may show `…`.

The entire box background changes so the user can scan states quickly.

The level box is not clickable.

### Read/Write cell

Input modes:

```text
[ Read ]
```

Output mode:

```text
[ Write HIGH ]
```

or

```text
[ Write LOW ]
```

The label states what pressing the button will do, not what the current value is.

Do not use the ambiguous `Toggle` label.

The cell keeps the same proportional share in every row, while the button itself uses natural/native width.

### Listen/Stop cell

Input modes:

```text
[ Listen ]
```

or

```text
[ Stop ]
```

Output mode:

```text
<empty reserved cell>
```

Important:

- `Listen` and `Stop` should keep their natural widths;
- the difference in button width is useful for quick scanning;
- the cell around them keeps the same proportional share so the rest of the row stays aligned.
- do not make listening green;
- default behavior should stay native/neutral;
- if Stop receives a semantic color treatment, use red/danger, never HIGH-green.

---

## Pending-state behavior

Pending state must provide feedback without changing layout.

Good patterns:

- disable the affected button in place;
- show `…` in the fixed level box for a pending read/write;
- disable the mode dropdown while a mode request is pending;
- preserve every cell width and every column position.

Do not use dynamic labels such as:

- `Reading...`
- `Sending...`
- `Setting...`

when they make controls wider or visually jump.

---

## Bulk toolbar

The bulk toolbar lives directly above the pin table.

It uses **one independent scope dropdown**:

```text
Scope [ ALL ▼ ]
```

Build the options from the selected route's discovered map:

- `ALL`
- one entry for each discovered bank. Use its native token, for example `PIOA` or `PIO2`.

The bulk scope is independent from the visible tab. Switching routes resets the scope selector to that route's `ALL`. It does not alter device state.

Full toolbar concept:

```text
Scope [ ALL ▼ ]   Mode [ INPUT ▼ ]   [ ] Overwrite   [ Apply mode ]
[ Read ]   [ Listen ]   [ Stop listening ]   [ Set HIGH ]   [ Set LOW ]
```

Exact wrapping may adapt to available width, but the grouping should stay obvious.

### Apply mode

Selecting a mode does nothing by itself. The user must press `Apply mode`.

#### Overwrite off

A pin is protected if its mode is anything other than `UNSET`.

When Overwrite is off:

- only `UNSET` pins in the selected scope are changed;
- explicitly configured pins are skipped;
- the GUI iterates eligible pins individually and sends per-pin operations;
- do not use a `PIOx` or `ALL` direction command because that would overwrite protected pins.

#### Overwrite on

When Overwrite is on:

- existing configuration may be replaced;
- use the selected scope directly (`PIOA` ... `PIOE` or `ALL`);
- the firmware expands the bank/global target and skips reserved/unavailable pins.

### Read

Read all eligible/initialized pins in the selected scope.

### Listen / Stop listening

These are two separate buttons, not a toggle.

Reason: the selected scope can contain a mixed state, for example 10 listening pins and 10 non-listening pins.

- `Listen` enables listening for eligible pins in the scope.
- `Stop listening` disables listening for eligible pins in the scope.

### Set HIGH / Set LOW

Keep both actions in the toolbar.

- single-pin writes execute immediately;
- bank (`PIOx`) and `ALL` writes require an explicit confirmation before the command is sent;
- cancelling the confirmation sends nothing.

The protocol itself allows SET on individual pins, banks, and ALL.

---

## Protocol target naming

Packet IDs remain. Only GPIO target naming changes.

Example:

```text
042 GET PA01 OK?
042 HYG PA01 HIGH <3
```

`042` is the request/response correlation ID. It is not a GPIO ID.

### Individual pin targets

Wire format is the PIO name with a two-digit zero-padded bit number:

```text
PA00
PA01
PB08
PC25
PD31
PE05
```

Do not accept or emit numeric virtual GPIO target IDs such as `001`, `044`, or `103`.

### Bank targets

Bank targets use each MCU's native discovered bank token. SAM examples are `PIOA` through `PIOE`. LPC examples are `PIO0` through `PIO3`.

### Global target

Add:

```text
ALL
```

### Target grammar

Conceptually:

```text
TARGET = PIN | PIO_BANK | ALL
```

Every command that accepts a GPIO target may use all three target kinds:

```text
001 DIR PA01 IN OK?
002 DIR PIOA OUT OK?
003 GET PIOC OK?
004 SET ALL LOW OK?
005 PLL PIOB ON OK?
006 LSN PIOE ON OK?
007 WYD ALL DIR
```

### Responses and errors

Responses that identify a pin also use symbolic PIO naming:

```text
003 HYG PC25 HIGH <3
007 HYG PA01 DIR IN <3
008 UMM PB08 UNAVAILABLE <3
```

Numeric virtual GPIO IDs should disappear from raw logs as pin identities too.

### Scope semantics

- a native bank token affects eligible pins in that discovered bank only.
- `ALL` affects eligible pins across all banks of the addressed route.
- reserved/unavailable pins are skipped during bank/global expansion;
- directly targeting a reserved/unavailable pin still produces the normal unavailable error;
- `PIOB + PIOE` is never a protocol target.

Bulk GET/query/listener streams retain the same packet ID for their per-pin responses and finish using the existing completion/ack behavior.

---

## Implementation guidance

### Do not spread pin mapping logic

Each firmware adapter owns its native bank/pin metadata and exposes it through route-local MAP discovery. The desktop session turns that discovery into compact `RouteKey` / `BankKey` / `PinKey` identities and remains the authoritative owner of the discovered metadata and mutable device state.

The UI must render those session-owned keys and metadata directly. It must not rebuild MCU pin tables, derive package pins from names, or repeatedly translate presentation strings back into protocol targets. Presentation-only grouping can resolve native bank tokens to `BankKey`s once when a route map becomes available.

### Layout model

Prefer:

- `Length::Fill` / `Length::FillPortion` for horizontal layout.
- one shared set of proportional column weights for headers and rows.
- intrinsic widget sizes inside those cells;
- symmetric widget padding;
- one spacing scale;
- stable row/control heights.
- Iced `Responsive`, wrapping rows, and internal scrolling when the available width changes.

Avoid:

- hand-centering text by x-coordinate math;
- measuring every current label to determine the row layout;
- fixed pixel widths for table columns or top-level horizontal layout.
- character-count truncation as a substitute for layout.
- hiding one control in a way that causes later cells to shift.

Absolute dimensions are acceptable for compact control heights, spacing, usable pane/window minimums, and a responsive width threshold. Do not use them to hand-place or hand-size the horizontal table geometry.

Serial-device selectors display a semantic short label, such as the device name, while retaining the complete path internally for opening the port.

### Native-looking controls

The Python implementation mixes CustomTkinter surfaces with `ttk` controls. Reproduce the resulting feel rather than the exact implementation technique.

In Rust/Iced:

- use neutral backgrounds for ordinary buttons;
- use bordered dropdown/input wells;
- let button width follow the label;
- keep focus/pressed states subtle;
- avoid using the Iced primary color as the default visual identity of every action.

---

## Acceptance checklist

The redesign is not done until all of these are true:

### Geometry

- [ ] LOW → HIGH does not move anything in the row.
- [ ] Listen → Stop does not move later columns.
- [ ] INPUT → OUTPUT does not move later columns.
- [ ] pending → complete does not move later columns.
- [ ] button text has symmetric left/right padding.
- [ ] empty output Listen/Stop cells still reserve their width.
- [ ] minimum-window and minimum-pane layouts contain no overlapping primary controls.
- [ ] a narrow GPIO pane stacks the two pin blocks instead of squeezing their columns.

### Visual parity

- [ ] The Rust UI reads as the same family as the Python UI.
- [ ] Controls look native/desktop-like rather than web-dashboard-like.
- [ ] Normal buttons are visually neutral.
- [ ] HIGH/LOW state is immediately scannable by background color.
- [ ] A route selector switches the rendered route without reconnecting or resetting session state.
- [ ] Bank tabs come from the selected route's discovered map instead of page-number navigation.
- [ ] PIOB and PIOE remain separate discovered banks inside the optional SAM combined listing tab.
- [ ] GPIO/log divider is user-resizable.

### Pin identity

- [ ] GUI shows route-native symbolic labels plus package pins when discovered, such as `PB12 (87)` or `PIO2_3 (38)`.
- [ ] No virtual numeric GPIO IDs are visible in the GUI.
- [ ] Wire commands and responses use the selected route's native symbolic pin/bank targets.
- [ ] packet correlation IDs remain unchanged.

### Bulk behavior

- [ ] Scope dropdown contains ALL and each discovered bank for the selected route.
- [ ] Scope dropdown does not follow the visible tab.
- [ ] Apply mode requires an explicit button press.
- [ ] Overwrite off skips already configured pins and iterates only UNSET pins.
- [ ] Overwrite on uses the bank/ALL target directly.
- [ ] Listen and Stop listening are separate bulk actions.
- [ ] Set HIGH / Set LOW are present in the toolbar.
- [ ] bank/ALL SET asks for confirmation before transmission.

### Protocol

- [ ] Individual pin, bank, and ALL targets work for every target-taking command.
- [ ] Old numeric GPIO target syntax is rejected/removed.
- [ ] Bank/global operations skip reserved pins.
- [ ] PIOB + PIOE exists only in the UI listing, never in the protocol.
