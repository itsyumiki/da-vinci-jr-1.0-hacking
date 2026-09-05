# GPIO wire protocol

The GPIO controller uses newline-delimited ASCII packets. The desktop host sends requests and firmware nodes send responses. Every packet begins with a request ID and a route token.

## Framing and packet IDs

Each packet is one ASCII line terminated by `\n`. Receivers can ignore `\r` while assembling a line. A packet must fit within the protocol's 64-byte maximum frame size, including its newline terminator.

The host allocates request IDs from `001` through `999`. IDs are decimal, and the host renders them as three digits on the wire. After `999`, allocation wraps to `001`. The host must not reuse an ID while a request with that ID is still outstanding. Firmware preserves the host's ID in every response and listener event. Intermediate devices do not allocate or translate IDs.

Most requests finish after one response. Grouped requests and `MAP` finish at their terminal `OKA`. A successful `LSN ... ON` keeps its request ID alive for later `HYG` listener events until listening is stopped, replaced, or reset.

## Packet envelopes

Requests have this form:

```text
<id> <destination> <command...>
```

Responses have this form:

```text
<id> <source> <response...>
```

For example:

```text
001 SAM HAI
001 SAM HII <3
```

`destination` and `source` are protocol-name tokens such as `SAM`, `LPC`, or another configured name. They are not a closed list defined by the protocol. A token is non-empty and contains no whitespace or control characters.

The current PC-connected node is named `SAM`.

## Targets

GPIO commands that take a target accept one of three forms:

| Form | Example | Meaning |
| --- | --- | --- |
| Pin | `PA00` | One GPIO pin |
| Bank | `PIOA` | One GPIO bank |
| All | `ALL` | Every exposed GPIO pin |

The current SAM node exposes these native names:

- `PA00` through `PA31` in `PIOA`.
- `PB00` through `PB14` in `PIOB`.
- `PC00` through `PC31` in `PIOC`.
- `PD00` through `PD31` in `PIOD`.
- `PE00` through `PE05` in `PIOE`.

Numeric GPIO IDs are not valid targets. UART1 reserves SAM pins PA05 and PA06 for the LPC link. The board/USB path reserves PB08 through PB11. These pins report `UNAVAILABLE` for individual GPIO operations. Grouped operations skip unavailable pins.

Pins begin in the `UNSET` state. `DIR` initializes a pin. Commands whose meaning depends on an initialized direction do not implicitly initialize it.

## Requests

| Command | Form | Meaning |
| --- | --- | --- |
| Hello | `HAI` | Check that the addressed node is responding. |
| Status | `HRU` | Ask the node to identify itself. |
| Map | `MAP` | Stream the addressed node's GPIO bank and pin metadata. |
| Direction | `DIR <target> IN OK?` / `DIR <target> OUT OK?` | Configure input or output direction. |
| Read | `GET <target> OK?` | Read initialized pins in the selected scope. |
| Write | `SET <target> LOW OK?` / `SET <target> HIGH OK?` | Drive initialized output pins. |
| Pull-up | `PLL <target> OFF OK?` / `PLL <target> ON OK?` | Turn input pull-up off or on. |
| Listen | `LSN <target> OFF OK?` / `LSN <target> ON OK?` | Stop or start change notifications. |
| Query | `WYD <target> DIR` / `WYD <target> PLL` / `WYD <target> LSN` | Query direction, pull-up, or listener state. |
| Reset | `BYE` | Reset GPIO/listener state for the node. |

### Command ordering

`DIR` establishes the pin's direction and initializes its GPIO state. A direction change resets pull-up state to off. Configure direction before relying on `GET`, `SET`, `PLL`, or `LSN`.

`PLL` is meaningful for initialized inputs. `SET` acts on initialized outputs. `LSN` reports changes on initialized inputs. Grouped forms apply only to pins for which the operation is meaningful and skip pins that do not qualify.

## Responses

All successful and error responses keep the original request ID and name the responding source node.

| Response | Meaning |
| --- | --- |
| `HII <3` | Reply to `HAI`. |
| `IAM SAM4E8E GPIO <3` | Current SAM status reply to `HRU`. |
| `MAP BANK <bank> <3` | One bank record in a `MAP` stream. |
| `MAP PIN <target> <package-pin|-> <bank> <bit> <capabilities> <3` | One pin record in a `MAP` stream. |
| `OKA <3` | Successful acknowledgement or grouped-response terminator. |
| `HYG <pin> LOW <3` / `HYG <pin> HIGH <3` | Pin value or listener event. |
| `HYG <pin> DIR <value> <3` | Direction state. Value is `IN`, `OUT`, or `UNSET`. |
| `HYG <pin> PLL <value> <3` | Pull-up state. Value is `ON`, `OFF`, or `UNSET`. |
| `HYG <pin> LSN <value> <3` | Listener state. Value is `ON`, `OFF`, or `UNSET`. |
| `UMM BAD_PACKET <3` | A known request form was malformed. |
| `UMM <pin> UNSET <3` | An individual operation required an initialized pin. |
| `UMM <pin> UNAVAILABLE <3` | An individual operation addressed an unavailable/reserved pin. |
| `UMM NO_ROUTE <destination> <3` | This node has no configured path to the requested destination. |
| `UMM ROUTE_BUSY <next-hop> <3` | The bounded queue for the selected next hop cannot accept another frame. |
| `UMM ROUTE_DOWN <next-hop> <3` | The selected next-hop link reported a hard failure. |
| `IDK <3` | The command name is unknown. |
| `CYA <3` | Reply to `BYE`. |

If packet framing or the packet ID is malformed before a usable request ID exists, no correlated response is possible.

## Pin-map discovery

`MAP` describes the GPIO topology of the addressed node. It discovers that node's banks and pins. It does not discover routes or the network graph.

One `MAP` request streams every bank record followed by every pin record under the same request ID. `OKA <3` terminates the stream. A node with no exposed banks or pins can reply with the terminal `OKA` immediately.

Bank records have this form:

```text
070 SAM MAP BANK PIOA <3
```

Pin records have this form:

```text
070 SAM MAP PIN PA00 102 PIOA 0 7 <3
070 SAM MAP PIN PA05 73 PIOA 5 0 <3
070 SAM OKA <3
```

The pin fields are the native target token, physical package-pin number, bank token, bit/order within the bank, and capability bits. A `-` package-pin field means that the physical package pin is unknown.

Capability bits are additive: `1` means input, `2` means output, and `4` means pull-up control. For example, `7` supports all three operations, `1` is input-only, and `0` marks a pin unavailable for GPIO commands. MAP includes unavailable pins so a host can display the complete node topology while respecting those capability flags.

The metadata in a `MAP` response is the same topology that the node uses when it resolves later GPIO target tokens. A host must discard a partial map if the stream fails before the terminal `OKA`.

## Individual and grouped operations

An individual target produces its normal direct response or error. Bank and `ALL` targets are grouped operations where applicable.

`GET <bank|ALL> OK?` sends one `HYG <pin> <level> <3` response for each initialized matching pin, then finishes with `OKA <3` under the same request ID.

`WYD <bank|ALL> <query>` sends one `HYG` state response for each available matching pin, then finishes with `OKA <3` under the same request ID.

Grouped mutations such as `DIR`, `SET`, `PLL`, and `LSN` apply to the matching eligible pins and finish with `OKA <3`. Unavailable pins are skipped rather than turning the whole grouped operation into an error.

Example grouped read:

```text
020 SAM GET PIOC OK?
020 SAM HYG PC00 LOW <3
020 SAM HYG PC01 HIGH <3
020 SAM OKA <3
```

The exact number of `HYG` lines depends on current pin state and the chosen operation.

## Listener lifecycle

A successful listener-enable request is persistent. The acknowledgement and every later change notification use the original request ID:

```text
030 SAM LSN PA00 ON OK?
030 SAM OKA <3
030 SAM HYG PA00 HIGH <3
030 SAM HYG PA00 LOW <3
```

`LSN PA00 OFF OK?` stops that listener. Re-enabling/replacing a listener transfers future events to the new request ID only after the new request succeeds. `BYE` clears all listener state.

Grouped listener setup follows the same rule: one request ID identifies the listeners created by that grouped request until a later command stops, replaces, or resets them.

## Reset behavior

`BYE` resets initialized pins to input with pull-up off, clears listener state, and forgets the node's GPIO configuration state. The response is:

```text
040 SAM BYE
040 SAM CYA <3
```

After reset, pins are `UNSET` again from the protocol's point of view.

## Error examples

Malformed known command:

```text
050 SAM DIR PA00 SIDEWAYS OK?
050 SAM UMM BAD_PACKET <3
```

Unknown command:

```text
051 SAM WAT PA00
051 SAM IDK <3
```

Unavailable pin:

```text
052 SAM DIR PB08 OUT OK?
052 SAM UMM PB08 UNAVAILABLE <3
```

Uninitialized individual read:

```text
053 SAM GET PA00 OK?
053 SAM UMM PA00 UNSET <3
```

Unknown route:

```text
054 XYZ HAI
054 SAM UMM NO_ROUTE XYZ <3
```

The response source identifies the node that failed the route lookup. In this example, that node is `SAM`. The `NO_ROUTE` argument preserves the unresolved destination.

## Routed forwarding

A node can configure one or more downstream next hops. One next hop can serve several destination names. The router selects a next hop from the request destination and forwards the complete request frame without changing its ID, destination, or command body.

Downstream responses and listener events travel upstream as complete frames. Intermediate nodes do not rewrite their IDs or source names.

The current SAM firmware configures one downstream route named `LPC` over SAM UART1. The router forwards a request such as `054 LPC HAI` unchanged. An LPC reply such as `054 LPC HII <3` returns to the host unchanged with the same host-allocated ID.

If a link temporarily cannot make progress, the router keeps accepted frames in a fixed-capacity queue. Temporary flow control does not mean that the route is down. When that queue is full, a new request fails locally:

```text
060 LPC HAI
060 SAM UMM ROUTE_BUSY LPC <3
```

If the next-hop adapter reports a hard failure, the request fails with `ROUTE_DOWN` instead:

```text
061 LPC HAI
061 SAM UMM ROUTE_DOWN LPC <3
```

The error argument names the failed next hop, while the response source names the node that detected the routing failure.
