# Gerberus roadmap

Gerberus checks a PCB design for electrical bugs before it's ordered. It's an open-source, agent-friendly take on the verification tools that big companies pay for.

Each phase reads one more part of the design. It starts with the schematic, then adds parts, power, making any board work, layout, manufacturing and workflow. Every checker shares the same design graph and the same Breakdown report.

## v0.1 — Schematic

- [x] Voltage-aware schematic checks: worst-case voltage range on every net, with an explanation of where each range came from, plus the first 10 rules: `gerberus voltages` and `gerberus check`. *(Copies: Siemens Xpedition schematic analysis)*

## v0.2 — Parts

- [x] Check that symbol, footprint and datasheet pinouts agree (catches swapped or missing pins): `gerberus pinout`. *(Copies: Ultra Librarian / SnapEDA verification)*
- [x] BOM health: stock, LCSC listing status and candidate second sources from LCSC. Manufacturer lifecycle (NRND, EOL) isn't checked: LCSC doesn't give it. *(Copies: SiliconExpert, Octopart / Supplyframe)*
- [x] Datasheet → part model using an LLM, checked by a blind second reading and by finding every value on its cited page (`gerberus datasheet`). *(New: no paid tool does this)*

## v0.3 — Power and stress

- [x] SPICE stress checks on power and analog circuits, with tolerance sweeps. Power is done (`gerberus stress`: hot-plug transients in ngspice, Monte Carlo over part tolerances); analog signal paths moved to v0.4. *(Copies: PSpice Advanced Analysis, smoke and Monte Carlo)*
- [x] Power dissipation → junction temperature for every part. *(Copies: Simcenter Flotherm / Ansys Icepak, simplified)*
- [x] Derating against reliability standards. *(Copies: Relyence, Windchill reliability prediction)*
- [x] ESD and reverse-polarity protection on every external connector. *(Copies: what a human design review checks)*

## v0.4 — Any board

Point Gerberus at a new board and get useful answers in minutes. Every `UNKNOWN` says exactly what to add and where, and adding it takes one command. These items come from running Gerberus on the capstone audio board rev 2 and the amp board (in roccoagain/hardware): 39 of rev 2's 58 parts were unchecked, `check` wouldn't run until someone wrote an `inputs.toml`, and the amp board gave 6 false `floating-input` errors.

- [x] `gerberus doctor <design>`: one to-do list of everything missing, ranked by how many unknown results each item settles: parts with no part file (with the symbol's datasheet link, and a hint when several share one datasheet and could be a family), connector pins with no input range, pins a part file gives no limit or no `[[drive]]`. `--json` for agents. *(New: paid tools assume a librarian already did this)*
- [x] `check` runs without an `inputs.toml`, says what that leaves out, and points at `gerberus init`.
- [x] `gerberus init <design>`: writes a starter `inputs.toml` with an entry for every connector pin it can guess from the net's name, each marked as a guess with its reason: ground, USB VBUS 4.0–5.5 V, D+/D− 0–3.6 V with a "shorted to VBUS" fault case, Type-C CC 0–5.5 V, a net named like `12V` or `3V3` at ±10 %, a net the board drives itself and test points as `[[undriven]]`. Pins it can't guess are listed in comments. `check` says how many guesses remain and `doctor` lists them first, until a person replaces each source.
- [x] Output readable like rustc's: each finding with the parts and nets involved, the numbers compared, and a `help:` line naming the fix.
- [x] Part families: one file covers a whole series by decoding its part number exactly, and writes each part's file with every value cited to the series datasheet (`gerberus part show`). Yageo RC_L resistors and Samsung CL capacitors were the first built in; the 28 resistors on rev 2 are all Yageo RC_L.
- [ ] More families: UNI-ROYAL thick film (01005 to 2512, such as 0402WGF and 0603WAF) is built in; so are Yageo CC X7R and Fenghua general MLCCs, and TDK MPZ1608 and Murata BLM18H ferrite beads. Still to come: Stackpole RMCF, Murata GRM and KEMET, Yageo CC NP0, and power-inductor series.
- [ ] Bulk library: JLCPCB basic parts first, then the most-used extended parts, each through `datasheet verify` and a blind second reading. Track what share of the test boards' parts are covered. Batch 1 (18 diodes, transistors, LEDs, a TVS and an RS-485 transceiver) and the coverage test (`tests/coverage.rs`) are in.
- [x] Test points are skipped instead of `UNCHECKED`. (Mounting holes, fiducials and logos left out of the BOM are already skipped.)
- [x] Net ties join their two nets in the voltage engine and in `stress`, instead of staying `UNCHECKED`. Fuses count as shorts too, and two-pin switches as links that may be closed: the net past one takes its range, but isn't proven driven.
- [x] A limit the datasheet doesn't give is `{ none = "reason" }` and isn't checked, apart from `{ unknown = "reason" }` for one not known yet. An output with no voltage rating or a MOSFET with no recommended operating conditions no longer leaves an `UNKNOWN` that nothing could settle.
- [ ] `doctor` traces every `UNKNOWN` to something to add: part-file values not known yet, thermal and ESD data, and nets the voltage engine can't work out.
- [ ] Modules and dev boards (Daisy Seed, ESP32 and STM32 modules, Raspberry Pi Pico) as parts with their header pinout and limits.
- [ ] A `[[variant]]` the datasheet doesn't list (PESD5V0S1BAF), allowed only with a source from the maker that it is the same part.
- [x] `gerberus part new <design> <REF or MPN>`: writes a part file into the project's `parts/` folder from the schematic (pins from the symbol, class from the reference, every value `{ unknown = "not read from the datasheet yet" }`), ready to fill in and `datasheet verify`; `doctor` lists it until it is filled in. Reading a PDF given with `--pdf` waits for the `datasheets/` folder.
- [x] A `datasheets/` folder next to the schematic for PDFs that can't be downloaded (MAX9722A, Daisy Seed): `datasheet verify` reads `datasheets/<MPN>.pdf` before downloading, and `doctor` names the file to save for each part it lists.
- [x] Ship the agent skills with the binary (`gerberus skill install`): check-board (init, doctor, part files, check) and datasheet-to-part with its format guide, so an agent working in a board's own repo checks the board and adds parts the same checked way.
- [x] Waivers: a `waivers.toml` next to the schematic accepts a finding with a reason, who, and when. Waived findings leave the counts and still show with `--all` and `--json`, marked waived; a waiver that no longer matches anything is flagged. A waiver an agent writes carries `proposed` and doesn't apply until a person deletes that line. *(Copies: Siemens Xpedition waivers)*

Behaviour Gerberus doesn't model yet:

- [x] Inputs that bias themselves: a `biases` drive in the part file with the cited bias range, for pins such as the AC-coupled inputs of the TPA3118 and TPA6130A2. The net then gets a voltage the other rules can check, and `floating-input` passes.
- [x] MOSFET channels in the voltage engine: a part file's `channel` drive makes the part off (gate never past either end), fully on (always past the VGS RDS(on) is given at) or maybe on, and a channel that isn't off pulls each end toward the other, no further than its gate.
- [x] Analog signal paths in SPICE (gain stages, filters), not only power: `gerberus signal` runs each `[[signal]]` from `inputs.toml` through the board with every rail at its lowest and its highest, and checks op-amp outputs against their rails less the datasheet's output swing (`clipping`) and every pin limit a signal reaches (`signal-level`).
- [x] JLCPCB assembly check (`bom --jlcpcb`): every part needs an LCSC number; resistors and capacitors are checked against LCSC's parameters (value, size, tolerance, voltage, power, dielectric) rather than by part number, with checked LCSC equivalents offered for short or under-rated ones; extended parts counted. LCSC's keyword search is blocked from the cloud, so parts without a number get a search link, not a pick.

## v0.5 — Layout

- [x] Layout electrical rules in `gerberus traces`: trace width vs current (IPC-2221), decoupling capacitor distance, driven traces crossing plane splits, and switcher hot-loop length. *(Copies: HyperLynx DRC)*
- [x] Trace impedance and length matching for USB 2.0 high speed, with an eye-diagram simulation (`gerberus si`); line length for SPI and I²S in `gerberus traces`. *(Copies: Polar Si9000, HyperLynx SI)*
- [x] Voltage drop from each supply to every power pin, through the board's copper (`gerberus traces`). *(Copies: HyperLynx PI, Altium Power Analyzer)*

## v0.6 — Build

- [ ] Assembly and manufacturing checks: tombstoning, thermal reliefs, fiducials. *(Copies: Siemens Valor NPI)*
- [ ] Test-point coverage on every net. *(Copies: XJTAG, Valor test planning)*
- [ ] Board-vs-enclosure collision check from STEP files. *(Copies: Altium MCAD CoDesigner)*

## v1.0 — Workflow

- [x] Electrical diff between board revisions (`gerberus diff`). *(Copies: Altium / Cadence design compare)*
- [ ] CI integration and a pass/fail badge for the README.

## Signature features (new, not copies)

- [ ] **Firmware ↔ schematic cross-check.** Read the pin assignments in the firmware (Daisy pin config, STM32CubeMX `.ioc`, ESP-IDF) and confirm they match the netlist.
- [x] **Bring-up plan generator** (`gerberus bringup`). Gerberus already knows every rail and voltage range, so it writes the bring-up checklist: which rail to power first, the current limit to set, and the voltage to expect at each test point.

## Principles

- **Never invent datasheet values.** If a limit is missing, the result is `UNKNOWN` with a reason.
- **Deterministic core.** LLMs only help create part models; they never decide whether a check passes.
- **Every finding is explainable.** It cites the numbers it compared, their datasheet pages, and how the net's voltage was derived.
- **False positives are bugs.** A clean board must produce zero errors.
