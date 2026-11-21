# Part limits

One TOML file per part number. Each says which voltages, currents and power a part's pins may see, and where in
the datasheet that comes from. Checkers compare these with the voltage range they work out for each net.

**Never invent a value.** If the datasheet doesn't give it, write `{ none = "reason" }`: there is nothing to
check against, so that end isn't checked (an output with no voltage rating, a part with no recommended operating
conditions, the forward-biased end of a diode, which current limits rather than voltage). If it isn't known yet
(nobody has read it, or the datasheet can't be had), write `{ unknown = "reason" }`: a checker reports `UNKNOWN`
with that reason until someone fills it in. `none` skips the check only in a limit's `abs_max` and `recommended`;
a `breakdown` or a `[[drive]]` range needs the number, so a `none` end there leaves what rests on it UNKNOWN.

## Format

```toml
mpn = "TPS560430YFDBVR"            # matched against the schematic's MPN field (or Value, if there's no MPN)
manufacturer = "Texas Instruments"
description = "36 V 600 mA buck, SOT-23-6"
class = "ic"                        # capacitor, resistor, inductor, diode, tvs, led, connector, oscillator, ic

[datasheet]                         # leave out only when no datasheet can be found; then every value is unknown
title = "TPS560430 ... Step-Down Converter"
document = "SLVSE22B, revised June 2018"   # document number and revision, so pages can be checked
url = "https://www.ti.com/lit/ds/symlink/tps560430.pdf"

[pins]                              # pin number (as on the footprint) = datasheet pin name; list every pin
1 = "CB"
2 = "GND"
5 = "VIN"

[[variant]]                         # optional: another part number with the same limits (another package)
mpn = "OTHER-PACKAGE-MPN"           # placeholder; gives its own pin map, or reuses [pins] without one
pins = { 1 = "CB", 2 = "GND", 3 = "FB", 4 = "EN", 5 = "VIN", 6 = "SW" }

[[limit]]
pins = ["VIN"]                      # datasheet pin names, never numbers, so every package shares the limits
to = "GND"                          # measured from this pin
abs_max = { min = -0.3, max = 38, page = 4 }
recommended = { min = 4, max = 36, page = 4, note = "optional free text" }
```

### Pin map

`[pins]` is the datasheet's pin table for this package, and `gerberus pinout` compares it with the symbol and
the footprint. List every pin, including `NC`, `DNC` and an exposed pad (by the number the footprint gives it),
and write names as the datasheet prints them. Pads that are not pins at all, like a connector's mounting tabs,
stay out. A pin named `DNC` (do not connect) must not be wired to anything; `NC` may be.

### Ranges

A range is `{ min = ..., max = ..., page = N, note = "..." }`, `{ none = "reason" }` (the datasheet gives none)
or `{ unknown = "reason" }` (not known yet).

- `page` is the 1-based page of the PDF at `url` (not the printed page number). It is required whenever a bound
  is known.
- `derived = "<min> .. <max>; explanation"` works out a value the page doesn't print from what it does print.
  Each end is arithmetic on numbers (`+ - * /`, brackets, `sqrt(...)`), such as
  `"3.3 * 0.98 .. 3.3 * 1.02; VOUT(T) x 0.98 to x 1.02"` for an LDO's output range, or a name plus a number as
  the page prints it, such as `"-0.3 .. VDDIOx + 4.0; VDDIOx is VDD for these pins"`. Write `-0.3`-style ends
  even where the bound is printed. `gerberus datasheet verify` evaluates each end and calls a bound that doesn't
  match a MISMATCH; it looks for the arithmetic's numbers (or the printed name and offset) on the page and
  lists any it can't find. A derivation never hides a value printed in the wrong column.
- Each bound is one of:
  - a number: `38`
  - another pin of the same part: `{ pin = "VIN", plus = 0.3 }`, optionally capped:
    `{ pin = "VDD", plus = 3.6, at_most = 5.5 }` means min(VDD + 3.6 V, 5.5 V). The other pin's voltage is
    measured from the limit's `to` pin. `at_most` only goes on `max`.
  - `{ none = "reason" }`, for a datasheet that gives only one end
  - `{ unknown = "reason" }`, for an end not known yet

### Fields of a limit

| Field | Meaning |
| --- | --- |
| `quantity` | `"voltage"` (default, volts), `"current"` (amps into the part at `pins`) or `"power"` (watts between `pins` and `to`) |
| `pins` | Datasheet pin names the limit applies to |
| `to` | Reference pin. Without it, a voltage limit holds between any two of `pins` (a connector's contact rating) and a current limit holds for each pin |
| `abs_max` | Beyond this the part may be damaged |
| `recommended` | The range the datasheet says to operate in |
| `breakdown` | TVS, Zener and ESD parts: where the part starts to conduct (VBR). A rail that can sit above `breakdown.min` drives current through the part continuously |
| `clamp` | TVS and ESD parts, with `breakdown`: `{ volts, current, page }`, the maximum clamping voltage VC at the peak pulse current IPP. `gerberus stress` takes the part's resistance past breakdown from it |
| `bidirectional` | `breakdown` is a magnitude and applies in both polarities. `abs_max` and `recommended` stay signed |

### What a part drives

Limits say what a pin may see. `[[drive]]` entries say what the part itself sets when it works, so a checker
can work out the rails:

```toml
[[drive]]
kind = "regulates"     # a regulator holds `pin` at `range` from `to` (a fixed output, or a feedback pin)
pin = "FB"
to = "GND"
supply = "VIN"         # the pin that powers it; the drive holds only while this pin is in its recommended range
feedback = true        # the output is the far end of the two-resistor divider on this pin
headroom = { duty = 0.98, page = 1 }   # supply × duty − volts must reach the output; an LDO gives `volts`
range = { min = 0.985, max = 1.015, page = 5 }

[[drive]]
kind = "switches"      # a switch node; its range may follow the supply pin
pin = "SW"
to = "GND"
supply = "VIN"
range = { min = { none = "undershoot below GND isn't given" }, max = { none = "overshoot above VIN isn't given" }, page = 3 }

[[drive]]
kind = "biases"        # an input the part biases itself, such as an amplifier's AC-coupled input
pin = "INP"
to = "GND"
supply = "VDD"
range = { min = 1.2, max = 1.4, page = 7, note = "input bias voltage, VDD = 2.7 to 5.5 V" }

[[drive]]
kind = "forward"       # a diode conducting from `pin` (anode) to `to` (cathode) drops `range`
pin = "A"
to = "K"
range = { min = { none = "only a maximum is given" }, max = 0.6, page = 1, note = "at IF = 1 A" }

[[drive]]
kind = "channel"       # a MOSFET channel from `pin` (drain) to `to` (source), switched by `gate`
pin = "D"
to = "S"
gate = "G"
range = { min = 1.1, max = 2.1, page = 6, note = "VGSth, ID = 250 µA, Tj = 25 °C" }   # V(gate) − V(source)
on = { vgs = 5, page = 6, note = "RDSon at VGS = 5 V" }   # the VGS RDS(on) is given at nearest the threshold

[[drive]]
kind = "forward"       # an LED: `current` is the IF its VF range is measured at
pin = "A"
to = "K"
current = 0.020
range = { min = 1.8, max = 2.4, page = 3, note = "VF at IF = 20 mA" }
```

A `biases` drive sets the input's net only when nothing but capacitors share it (an AC-coupled input); with a
resistor or another part's pin on the net, the net is worked out as usual and the bias is ignored. It counts as
a pull, so `floating-input` passes. Its range works like a switch node's: a number, or another pin plus an
offset. Give `{ none = "..." }` when the datasheet says the input is biased internally but not to what.
`stress` doesn't model the bias: in its runs the input sits at whatever its capacitor couples in.

On a part with no part file, an input reached only through capacitors is `UNKNOWN` rather than a FAIL when it is
AC-coupled: a capacitor brings in a signal (its far side is driven and isn't a 0 V rail), or it is AC-grounded
while another input of the same part is AC-coupled (the unused leg of a differential input). Only the datasheet
can say whether the part biases it. An input with nothing on it but a capacitor to ground, on a part with no
other AC-coupled input, is still a FAIL.

A `channel` drive's `range` is the gate threshold VGS(th), negative for a P-channel part, and `on` the VGS
nearest the threshold that RDS(on) is given at. Their signs say whether the part is N- or P-channel. gerberus
measures the gate from both ends of the channel, since either can act as the source:

- **Off** when the gate never passes either end in the direction that turns the part on (above, for N-channel).
  The threshold is taken as 0 V rather than the datasheet's minimum, which holds only at 25 °C.
- **Fully on** when the gate is always at least `on` past one end.
- **Maybe on** otherwise.

A channel that isn't off pulls each end toward the other, but no further than the gate: an N-channel part can't
raise a net above its gate (a level shifter's low side stays at or below the gate), and a P-channel part can't
lower one below it. Its drop depends on a current gerberus doesn't know, so it isn't counted. A gate nothing is
sure to drive leaves the channel maybe on. A MOSFET with no part file, or a part file with no `channel` drive,
stays a part gerberus can't model. Keep a P-channel reverse-polarity switch's body diode as a `forward` drive from
D to S as well: it gives the source its range when the gate can't turn the channel on.

An LED's VF minimum holds only at and above `current`. Below it the LED drops less, so a series resistor's
current is then bounded with VF taken as 0 V.

A resistor, capacitor or inductor can give its tolerance, temperature coefficient and rated temperature range as
top-level keys, before the first table:

```toml
tolerance = { value = 0.01, page = 2, note = "7th code F = ±1 %" }
tcr = { ppm = 100, page = 6 }
temperature = { min = -55, max = 155, page = 5 }
```

### Op-amps

An `[[opamp]]` names one amplifier's pins, for `gerberus signal`, which models it as an ideal amplifier:

```toml
[[opamp]]
plus = "+IN"
minus = "–IN"
out = "OUT"
vpos = "V+"
vneg = "V–"
swing = { value = 0.02, page = 10, note = "output swing from rail, VS = 5.5 V, RL = 10 kΩ, MAX" }
```

`swing` is the MAX of the datasheet's output swing from the rails (or the supply less the MIN output voltage
high), at the load in the note. Without it, an output inside its rails is UNKNOWN rather than PASS. A dual
op-amp has two entries.

### Power and temperature

Any part can give its rated ambient range, its thermal resistance and junction limit, the most current it draws,
and (a switching regulator) a guaranteed efficiency, as top-level keys and `[[supply]]` tables:

```toml
temperature = { min = -40, max = 85, page = 47, note = "TA, suffix 6" }   # operating ambient
theta_ja = { value = 118, page = 111, note = "UFQFPN28" }                # θJA in °C/W, for this package
tj_max = { value = 105, page = 47, note = "TJ, suffix 6" }               # recommended TJ max, else absolute
efficiency = { value = 0.85, page = 7, note = "..." }                   # a guaranteed minimum, 0 to 1

[[supply]]
pins = ["VDD", "VDDIO2"]                # the pins together draw at most `current.max` amps
current = { min = 0, max = 0.0246, page = 50, note = "IDD, Run mode, every peripheral on, max" }
```

`current` takes the datasheet's maximum at the worst operating point the board can reach, never a typical.
Where a datasheet gives only typical figures (as most switching regulators do for efficiency, in curves),
leave the key out: the part's dissipation and anything fed through it come out UNKNOWN. A `derived` range can
add a typical share to a maximum when that is all there is; its note says so.

A part that loads a signal line, such as an ESD diode on a USB line, can give the capacitance its pins add, in pF.
`gerberus si` adds it to the lines those pins sit on. Give `typ`, `max` or both, as printed; checks use `max`
when there is one.

```toml
[[capacitance]]
pins = ["I/O"]
to = "GND"
typ = 0.42
page = 7
note = "line capacitance CL, VIO = 2.5 V, f = 1 MHz"
```

A pin no voltage limit mentions (other than `NC`/`DNC`) is UNKNOWN to every checker.

### ESD ratings

```toml
[[esd]]
model = "hbm"          # "hbm", "cdm", "iec-contact" or "iec-air"
volts = 4000           # the magnitude: ±4 kV is 4000
pins = ["CANH", "CANL"]  # left out: every pin
page = 4
note = "..."
```

`iec-contact` and `iec-air` are system-level ratings: IEC 61000-4-2, or ISO 10605 as automotive parts print it
(say which in `note`). Only these meet the `[esd]` level in gerberus's built-in `policy.toml`. A datasheet with no ESD rating gets no
`[[esd]]` entry, and its pins on an external connector come out UNKNOWN.

### Which datasheet number goes where

- `abs_max` takes what the datasheet calls absolute maximum or maximum ratings.
- `recommended` takes recommended operating conditions. When a datasheet has no such table, other operating
  ranges count: a supply range in the electrical characteristics, a receiver's common-mode range, a TVS's
  stand-off voltage (VRWM), and a passive's rated voltage, power or current. Notes say which.
- When a bound is relative to another pin, `recommended` can come out above `abs_max` (an STM32's 5 V-tolerant
  pin at low VDD). Checkers use the tighter of the two at the voltages they work out.
- Rated values of passives and connectors go in `recommended`, not `abs_max`: a capacitor's rated voltage is the
  most it may see continuously, and the withstanding-voltage test above it is a short test, not a rating.

### Starting a part file

`gerberus part new board.kicad_sch U2` writes `parts/<MPN>.toml` next to the schematic from U2's symbol: the part
number, the pin map (symbol pin names, without KiCad's overbar markup), a class guessed from the reference and
symbol (`ic` when it can't tell, and the comment says so), the datasheet link, and the `[[limit]]` the class takes
(across a passive, cathode to anode, between connector contacts, every pin from an IC's ground pin) with
`{ unknown = "not read from the datasheet yet" }` for every range. A part it can't give one to, such as a
transistor or an IC with no ground pin, gets a comment instead.

The file loads as it is. Until its values are read, `check` reports its pins UNKNOWN, and the rules that give a
part with no part file the benefit of the doubt (an input it may bias itself, a TVS that may clamp, an ESD path
that may reach an IC) treat it as missing. `gerberus doctor` lists it as a file to fill in, and `gerberus
datasheet verify` fails on it until no value says "not read from the datasheet yet". The pin names come from the
symbol, so check them against the datasheet's pin table yourself.

Keep the PDFs in a `datasheets/` folder beside `parts/`, each named after its part number with `.pdf`, characters
other than letters, digits and `-_.+` turned into `_` (`SM04B-GHS-TB(LF)(SN)` is `SM04B-GHS-TB_LF__SN_.pdf`);
the part file's own name with `.pdf` works too. `gerberus datasheet verify` reads a PDF saved there before it
downloads the `url`, and says so, so a part whose maker's site blocks downloads, or whose datasheet came by
email, still verifies. `doctor` names the file to save for each part it lists.

Name the part by reference or part number. `--stdout` prints the file instead, and `--parts DIR` adds folders to
look in. It won't replace a file, or write one for a part a built-in file, a part family or the symbol's own
rating (a capacitor's `Voltage` field) already covers, unless `--force`. It never writes a second file for a part
number one of the board's part files already defines.

### Checking a part file

Every value is read twice from the datasheet, by readers who haven't seen each other's work, and checked against
its page by `gerberus datasheet verify` (see the main README and `.claude/skills/datasheet-to-part`). Nobody signs
off on a file by hand, so a value that both readings and the page don't back stays `{ unknown = "..." }`.

### Where limits come from

Each placed part's limits come from the first of these that covers it:

1. **A part file.** The shared `parts/` folder loads first, then a `parts/` folder next to the schematic, then
   any `--parts` folders. A later file for the same part number replaces the earlier one, so a board can
   override the shared library.
2. **A part family**, for a part number with no part file of its own (see below).
3. **A class default.** A capacitor with no part file takes its rated voltage from its schematic `Voltage`
   field, or from its value (`100nF 50V`, `6V3`). It is marked as not checked against a datasheet. Other
   classes have no default.
4. **Nothing.** The part is reported as `UNCHECKED`, never as passing.

### Part families

Most resistors, capacitors and ferrite beads come in series whose part number spells out the size, tolerance, value and rating,
all from one datasheet. A family file covers the whole series: a file with a top-level `family` key, in any part
folder. For a part number with no part file, gerberus decodes it with each family (a later folder's family first)
and writes the part file a person would have written for that one part, every value cited to the series
datasheet. `gerberus part show <MPN>` prints it, so it can be checked with `gerberus datasheet verify` like any
other file.

```toml
family = "Yageo RC_L"
manufacturer = "Yageo"
description = "thick-film chip resistor"
class = "resistor"                                  # resistor, capacitor or inductor
mpn = "RC{size}{tolerance}{packing}-{reel}{value}L" # each {field} is one coded part of the number

[datasheet]                                         # the series datasheet, as in a part file
title = "..."
document = "..."
url = "..."

[field.size]                                        # every field names the page that explains its codes
page = 2
codes = ["0402", "0603", "0805"]

[field.tolerance]                                   # codes with the fraction each stands for
page = 2
codes = { B = 0.001, D = 0.005, F = 0.01, J = 0.05 }

[field.value]                                       # the value: "resistance" (4K7, 97R6, 1M),
page = 2                                            # "resistance-digits" (1001 = 1 kohm, 0103 = 10 kohm,
                                                    # 470K = 4.7 ohm: three figures, then a power of ten
                                                    # 0-6 or J-P for 10^-1 to 10^-6) or
                                                    # "capacitance-pf" (104 = 100 nF, 1R5 = 1.5 pF)
kind = "resistance"

[[rating]]                                          # a row of the ratings table, for the codes in `when`
when = { size = ["0805"], reel = ["07", "10", "13"] }
power = 0.125                                       # resistors: watts at the rated ambient
max_working_voltage = 150                           # resistors: the size's cap on sqrt(P x R)
temperature = { min = -55, max = 155 }
page = 5

[[tcr]]                                             # resistors: temperature coefficient by resistance
when = { size = ["0805"] }
ohms = { above = 10, max = 10e6 }                   # min/max include the end, above/below don't
ppm = 100
page = 5

[[range]]                                           # values the series is made in; with any of these, a
when = { size = ["0805"], tolerance = ["F"] }       # part number outside all of them isn't rated
ohms = { min = 1, max = 10e6 }
page = 5
```

A resistor family also gives `rcwv_page`, the page that says the rated voltage is √(P·R) or the maximum
working voltage, whichever is less.

Capacitor rows give `voltage` (the rated voltage) instead of `power`. Inductor and ferrite-bead rows give
`current` (the rated current, in amps); the part gets that as a current limit in either direction, and a voltage
limit of `none`, since the voltage across it is set by the current through it. A bead series' table usually
lists one part number per row, so each row's `when` names every code that picks it out. A field can list its codes and give
`values` for only some of them: a capacitor tolerance in picofarads (B, C, D) or +80/−20 % (Z) gives no fraction,
so the part gets no tolerance. Each rating comes from the one row that covers the part number and gives it: no
row leaves it out (power, rated voltage or rated current left out makes the part unchecked, with the reason), and two rows are an
error. So is a missing temperature coefficient when the family has `[[tcr]]` rows, and a part number that splits
into fields more than one way. A resistor's rated voltage is √(P·R) from the decoded value, capped by the maximum working voltage. A 0 Ω
jumper isn't rated, since it has a current rating rather than a power one.

A part file written by hand for the same part number always wins, wherever it is (a built-in file beats a
board's own family); of two families, the one in the later folder wins. `gerberus part show` runs the same checks
as `check` on what it prints. `cargo test` checks that every hand-written
file a family also decodes agrees with it. The built-in families are Yageo RC_L resistors (`yageo-rc.toml`),
UNI-ROYAL thick-film resistors (`uniroyal-thick-film.toml`, such as 0402WGF1001TCE, most of the JLCPCB basic
resistors), Samsung CL capacitors (`samsung-cl.toml`), Yageo CC X7R capacitors (`yageo-cc-x7r.toml`) and
Fenghua general MLCCs up to 50 V (`fenghua-general.toml`, such as 0603B104K500NT), TDK MPZ1608 power-line beads
(`tdk-mpz1608.toml`) and Murata BLM18H beads (`murata-blm18h.toml`, such as BLM18HG601SN1D). The capacitor families
rate a part by its voltage code; they don't check that the datasheet's capacitance tables list that value.

### Derating policy

Datasheet limits are facts; derating is project judgement. It lives in `policy.toml` at the root of the Gerberus
repository, never in a part file, and is built into gerberus: a board can't change it yet. Its format:

```toml
[[derate]]
class = "capacitor"     # or mpn = "..." for one part, which wins over a class rule
quantity = "voltage"
factor = 0.8            # use at most 80 % of the rating
reason = "..."

[junction]
margin = 20             # keep every junction this many °C below its maximum
reason = "..."
```

`quantity` is `voltage` (capacitors), `power` (resistors) or `current` (diode and LED forward current). These
shares and the junction margin are the reliability derating: a FAIL against them is a warning, while a FAIL
against the rating itself is an error.

### Board inputs

Each board has an `inputs.toml` next to its schematic (see `tests/fixtures/*/inputs.toml`). It gives the
voltages the outside world puts on its connector pins. `gerberus init board.kicad_sch` starts one: an entry for
each connector pin whose net name gives a guess (USB VBUS; D+/D− with a short-to-VBUS fault case and CC, both only
on a USB connector; a supply named like `12V` or `3V3` at ±10 %, unless the name also says it is a signal such as
`EN_12V`; a net the board drives itself, as `[[undriven]]`), each with a `source` that starts "GUESS by gerberus
init", and a comment for each pin it can't guess. It also writes ground as a 0 V reference (an `[[input]]` whose
`pin` and `to` are the same ground pin, the one place that is allowed) and test points as `[[undriven]]`; both say
"assumed by gerberus init" and aren't counted as guesses. Replace each guess's source
with where the range really comes from; until then `check` says how many guesses its results rest on and
`doctor` lists them first. It won't replace an existing file unless `--force`. Without an inputs.toml, `check` and
`voltages` still run but say so. A net named `GND`, `0V` or `VSS` is taken as 0 V, every net only a connector reaches is UNKNOWN,
nothing is checked for ESD, reverse polarity or faults, and `stress` has no supply to switch on.

```toml
board = "can-encoder rev 2"

[[input]]
name = "VBUS"
pin = "J1.1"        # REF.PIN
to = "J1.4"
min = 7
max = 26
source = "hardware/can-encoder/DESIGN.md: input range 7–26 V"
```

#### Signals

A `[[signal]]` is an AC signal brought in at a connector, for `gerberus signal`: up to `peak` volts either side
of the pin's DC bias, across `frequency` (20 Hz to 20 kHz when it isn't given). By default the source is
DC-coupled, as a pickup or a line driver into a DC path is: it holds the pin at the DC of `to`, or, when the pin has
an `[[input]]`, anywhere in that input's range less the peak at each end (the input's range is the pin's whole
voltage, the swing included). With `ac_coupled = true` it reaches the pin through a capacitor, so the board sets the
pin's bias. Two DC-coupled signals can't hold one pin:

```toml
[[signal]]
name = "guitar"
pin = "J1.T"
to = "J1.S"
peak = 1.5                  # volts, half the peak-to-peak
frequency = [20, 20000]     # hertz
ac_coupled = false          # true: the source is capacitor-coupled, the board biases the pin
source = "DESIGN.md: passive pickups, 1.5 V peak (3 Vpp)"
```

## Net voltages

`gerberus voltages board.kicad_sch` prints the worst-case range of every net; `--explain NET` prints why,
down to the inputs it comes from. It starts from `inputs.toml` (the `to` pins are 0 V) and follows:

- **Diodes** (`forward`): the cathode is the anode minus the drop. A drop with no minimum is taken as 0 V.
- **Regulators** (`regulates`): a `feedback` pin sets the output net through its two-resistor divider to
  ground. Each resistor counts its `tolerance` plus `tcr` drift over the board's temperature range, or its own
  rated range when `inputs.toml` gives none. Anything else on the feedback net makes the output UNKNOWN.
  Without `feedback`, the drive sets the pin's own net. The range holds only while the supply stays inside the
  supply pin's recommended range and meets the `headroom`. Below that, or with no `headroom`, the low end is
  0 V, marked as only a floor: checks can pass on it but not fail on it, and nets fed from it inherit the mark.
  Above the recommended maximum, the top end follows the supply. Line and load regulation aren't modelled.
- **Switch nodes** (`switches`): whatever their range gives; both ends are UNKNOWN on today's parts.
- **Biased inputs** (`biases`): an input reached only through capacitors sits at its bias range.
- **MOSFET channels** (`channel`): off, fully on or maybe on, from the gate against both ends. A channel that
  isn't off pulls each end toward the other, no further than the gate; one that is off pulls nothing.
- **Net ties (any number of pins), fuses and ferrite beads** are DC shorts: the net on the far side takes the
  driven net's range.
- **Everything else**: the union of what can pull the net: nets reached through resistors, inductors, LEDs,
  jumpers and two-pin switches (capacitors count as open, TVS diodes as off), and IC output pins, which reach at most their part's
  highest supply. A pin of any other part that isn't an input makes the net UNKNOWN. A switch passes its range
  only while it is closed, so a net only switches reach isn't proven driven: an input there is UNKNOWN for
  `floating-input`, as a pushbutton with no pull resistor leaves it floating.

A net with a connector pin that `inputs.toml` doesn't describe is UNKNOWN, as is anything that needs a number
nobody gave. Each unknown end says why. An input sets its whole net, so other connector pins on that net (a
daisy-chain connector) share its range. A pin the outside world doesn't drive, such as a debug probe's
target-voltage sense pin, is listed as `[[undriven]]` with its source:

```toml
[[undriven]]
pin = "J3.5"
source = "hardware/can-encoder/DESIGN.md, J3 row: 5 3V3. Assumes the probe only senses it"
```

An input's `max` can follow another pin instead of being a number. A debug probe drives SWDIO, SWCLK and NRST
at up to the target voltage it senses:

```toml
[[input]]
name = "SWDIO (debug probe)"
pin = "J3.2"
to = "J3.1"
min = 0
max = { pin = "J3.5" }   # the net J3.5 is on; `plus = 0.3` would add 0.3 V
source = "..."
```

Two inputs that move together, such as a CAN bus's two lines, can limit the voltage between them. Checks use it
wherever they compare the two nets, such as a termination resistor across them:

```toml
[[differential]]
pin = "J1.2"   # V(J1.2) − V(J1.3) stays in min..max
to = "J1.3"
min = -0.12
max = 3
source = "..."
```

A **fault case** is a set of inputs with the same `fault` name. They are left out of normal operation. Each case
is worked out on its own: its inputs replace the normal inputs on the nets they touch (and any `[[differential]]`
on those nets), and `gerberus check` runs the damage rules on it: abs max, TVS breakdown, and capacitor ratings
without derating. Resistor power and other passives' ratings run too, but a FAIL there becomes UNKNOWN. Only one
fault is assumed at a time. A `[[differential]]`'s pins must both be on nets a normal input drives.

```toml
[[input]]
name = "CANH shorted to VBUS"
fault = "VBUS shorted onto CANH"
pin = "J1.2"
to = "J1.4"
min = 7
max = 26
source = "..."
```

Every supply input (a normal input with `min` above 0) is also checked connected backwards: the fault case
`<name> reversed` puts `-max` to `-min` on its pin. A connector that can't be plugged in backwards says so with
`reverse = "reason"` on the input, which skips the case.

`inputs.toml` can also give `temperature = { min = 0, max = 70, source = "..." }` for the board's ambient.

## What's here

Datasheets are LCSC's copy for the exact LCSC part number unless the file names another URL. Values were checked
against the PDF at that URL.

| Board | Parts |
| --- | --- |
| sniffer | ME6211C33M5G-N, STM32G030F6P6TR, SCD41-D-R2, KT-0805G, SM04B-SRSS-TB, CL21A106KAYNNNE, CL10A475KO8NNNC, CC0603KRX7R9BB104, 0603WAF1002T5E, 0603WAF1001T5E |
| can-encoder rev 2 | TPS560430YFDBVR, STM32F042G6U6TR, MT6835GT-STD-R, TCAN1044AVDDFRQ1, XC6206P332MR-G, SMF26A, B5819WS, KT-0603R, SM04B-GHS-TB, OT201616MJBA4SL, SWPA252012S100MT, CC0805KKX7R9BB225, CL05B104KB54PNC, CL05B104KO5NNNC, CL21A226MAQNNNE, CL05A105KA5NQNC, 0402WGF4702TCE, 0402WGF1202TCE, 0402WGF1001TCE, 0402WGF1200TCE, 0402WGF1002TCE |
| can-encoder rev 1 | AP63200WU-7, STM32G0B1KBT6TR, MT6701CT-STD-R, TCAN330GDCNR, NUP2105LT1G, SMAJ18A, B5819W SL, KT-0805G, SM04B-GHS-TB, OT201616MJBA4SL, FNR4030S6R8MT, GRM21BR61H106KE43L, 0402CG101J500NT, CL05B104KO5NNNC, CL21A226MAQNNNE, CL05A105KA5NQNC, 0402WGF4702TCE, 0402WGF1502TCE, 0402WGF1001TCE, 0402WGF1200TCE, 0402WGF1002TCE |
| kitchen-sink | the parts above, plus USB4105-GF-A, 0402WGF5101TCE, TPD1E05U06DPYR and TVS0500DRVR |
| audio board rev 2 (B1narySolutions/amp-hardware, not a fixture) | TLV9061QDBVRQ1, AO3401A, 2N7002NXAKR, PESD5V0S1BA, SJ3-35063AG |
| amp board (B1narySolutions/amp-hardware, not a fixture) | TPA3118D2DAPR, TPA6130A2RTJR, TPS3808EG50DBVR, AO3401A, NSVBAT54HT1G, BZT52C12-TP, MPZ1608S221ATA00, SRP1038A-100M, SJ3-35063AG, PJ-102AH, IRF4905PBF |
| TI parts, on no board yet | TPS3808G50DBVR (the amp board uses its E successor) |
| JLCPCB basic and common parts, on no board yet | AO3400A, AMS1117-3.3, USBLC6-2SC6, LM358DR2G; the resistor and capacitor families above |
| JLCPCB basic parts, batch 1 (each a basic part at LCSC when added) | Schottky SS14, SS34, SS210; switching 1N4148W, 1N4148WS; rectifier SM4007PL; TVS P6SMB6.8CA; MOSFET 2N7002 (JSCJ); transistors MMBT3904, MMBT5551, MMBT5401, S8050, SS8050, S9013; LEDs KT-0805Y, KT-0603W, NCD0805R1; RS-485 SP485EEN-L/TR |

Not covered: footprint-only parts with no part number (Tag-Connect pads, SWD pads, solder jumpers).

### Known gaps

Found by the blind second reading of every file (November 2026):

- **Resistors' rated voltage** is the rated continuous working voltage √(P·R), capped by the size's maximum
  working voltage (UNI-ROYAL p5), not the maximum working voltage alone. The 120 Ω 0402 is rated 2.74 V.
- **CL05B104KB54PNC** isn't in the Samsung catalogue LCSC serves for it: its "4" and "P" codes aren't defined
  there. Its 50 V comes from voltage code B (p4). Worth checking against Samsung's own page for the part.
- The Samsung catalogue's text layer drops the minus signs in its temperature table (p5), so `datasheet verify`
  reports their −55 °C as NOT FOUND; the rendered page shows them.
- **MT6701CT**: its electrical parameters (p7) give VDD 3.0 to 5.5 V, but its pin table and front page say 3.3 to
  5.0 V. The file uses the table.
- **KT-0603R**: Pd 40 mW is lower than 25 mA × its 2.4 V maximum VF, so power may limit it before IF does.
- **B5819W**: IFRM 1.5 A applies to repetitive peaks only, so the DC current limit stays at IO, 1 A.

- **SWPA252012S100MT** (rev 2 L1): no datasheet reachable. LCSC has none for C37428 and sunlordinc.com is blocked
  from the cloud environment, so every value is unknown.
- **MT6701CT**: the datasheet gives only the VDD rating, so its signal pins are unknown.
- **OT201616MJBA4SL**: LCSC's datasheet for it is the YXC YSO110TR series sheet, which has no absolute maximum
  ratings.
- The CAN bus inputs assume every node's driver (the C610s' too) stays inside the ISO 11898-2 limits the TI
  transceiver datasheets give, which wasn't checked against the C610's own documents. No
  particular debug probe is used, so the probe inputs cover one that follows the target voltage and one fixed at
  3.3 V, assuming none drives above 3.3 V. Two more assumptions aren't checked: the probe only senses the
  board's VCC pin and doesn't power the board through it, and the board is powered while a probe is connected.
  A fixed 3.3 V probe on an unpowered board would drive its pins far above VDD + 0.3 V.
- **MOSFETs**: AO3401A's and IRF4905PBF's body diodes are a `forward` drive from D to S, which fits their use as
  reverse-polarity switches (supply into D) and would be wrong in a high-side switch fed at S. All four MOSFETs
  have a `channel` drive; `stress` still models a forward drive's part as its channel switched on (0.05 Ω).
- **TPS3808EG50DBVR** (TPS3808E datasheet SNVSCE0B): RESET is open drain, which no drive kind describes, so the
  net it pulls low comes out UNKNOWN. The CT pin's recommended minimum isn't given, and the supply current
  adds the MR pull-up current at its 90 kΩ typical, the only value given.
- **TPA3118D2DAPR, TPA6130A2RTJR**: the datasheets give no bias voltage range for the audio inputs (TPA3118: "biased
  at 3 V", no min or max; TPA6130A2: none), no output or charge-pump rail range, and no supply current under load,
  so those stay UNKNOWN.
- **BZT52C12-TP** is class `diode` with a `breakdown` range, not `tvs`: the datasheet rates power, not a stand-off
  voltage, and on the amp board it is a resistor-fed clamp, which the `tvs` rules would misread.
- **PESD5V0S1BA**: the audio board rev 2 orders PESD5V0S1BAF, which the PESD5V0S1BA datasheet doesn't list, so
  it isn't a variant here. Its jacks' ESD checks stay UNKNOWN until a datasheet names the BAF.
- Resistor power can't FAIL in fault cases: a fault drives the two ends of a resistor from independent ranges,
  which can overstate the voltage across it, so a FAIL there is UNKNOWN.
- Each CAN wire's range is the ISO common-mode range ± half the differential, which is wider than any one wire
  is likely to reach.
- **AP63200WU-7**: no headroom figure (the datasheet only says the duty cycle approaches 100 % near dropout), so
  rev 1's +3V3 has a 0 V low end. Its VFB range is for CCM; the AP63200 runs PFM at light load.
- The LDO output ranges (ME6211, XC6206) are specified at 25 °C.
- Logic inputs on the CAN transceivers have no recommended range in their datasheets (only VIH/VIL thresholds).
