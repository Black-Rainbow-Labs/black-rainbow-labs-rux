# Agent-Chat parts

These are the parts Unluminous's chat pane is drawn with: the charts in an answer, the readouts, the
buttons, the status wells, and the cards for a command that ran, a block of code, a change to apply, a
request for permission and an error. They were drawn to the Agent-Chat page of the Claude Design canvas
"Unluminous" in task-2235. Unluminous 0.72.0 draws every answer component with them.

Storybook: **Components → Chat Parts**.

![The Chat Parts story in the dark theme: charts, readouts, pills, buttons in five states, status wells and cards](../images/chat-parts/dark-neumorphic.png)

<details><summary>The Chat Parts story in the light theme</summary>

![The Chat Parts story in the light theme](../images/chat-parts/light-neumorphic.png)

</details>

## The design's rules

- **One light, at the top left.** Cards and controls are raised out of the pane. Wells and tracks are
  carved into it.
- **Everything a person presses is round or a pill.**
- **State is a mark in a carved well.** There are no status dots.
- **An amount is a dial, a ring, a slider or a line.** There are no segmented meters.
- **A press sinks.** Held down, a raised button becomes a well carved into the pane with its ink at full
  strength. That is the one change that rasterises the layer again. A hover changes only the ink.

Every part reads `theme.chat`, so the parts follow whichever pane they are drawn on.
[Themes](../themes.md#the-agent-chat-values) explains how those values are worked out from the pane.

## Charts

### Dial and Ring

```rust
let chat = rux.theme().chat;
Dial::new(128.0, 50.0)                        // 128 before, 50 added
    .centre("178", " s")
    .caption("+39%", chat.slower)
    .label("Release build, 178 seconds, 39% slower than last week")
    .show(rux, dial);

Ring::new(vec![
    Slice::new(131.0, SlicePaint::Action),
    Slice::new(38.0, SlicePaint::Colour(chat.slices[0])),
    Slice::new(9.0, SlicePaint::Colour(chat.slices[1])),
])
.centre("74%", "")
.caption("app", rux.theme().ink.i400)
.label("Where the release build's time goes")
.show(rux, ring);
```

The dial and the ring chart are one drawing: a raised rim, a channel carved into it, arcs 10 points wide
with round ends laid in the channel, and a raised disc in the middle carrying the number. A dial is a ring
of two parts, the amount before and the amount added. `.diameter(f32)` changes the size, which is 150
points by default.

The gap between two arcs is the same everywhere, and a short slice keeps its share. A round cap reaches
half the stroke past the end of its arc, so each arc is shortened by the stroke and the gap. A slice whose
share is shorter than the stroke cannot carry two round caps, so it is drawn with square ends at exactly
its length. `arc_segments(values, radius, width, gap)` is that arithmetic with no window behind it, and it
has its own tests.

### Track

```rust
Track::new(131.0, 140.0).marker(81.0).label("unluminous-app, 81 to 131 seconds").show(rux, track);
```

A groove filled with the action gradient up to the value, a raised knob at the end of the fill, and a
quiet mark where the value was before. It replaces a segmented bar. `.without_knob()` leaves the knob
out, and `Track::fraction(value, max)` answers the share from 0 to 1. A track is 22 points tall.

### AreaChart

```rust
let chart = AreaChart::new(&days)
    .range(110.0, 185.0)
    .marker(10, "rux added")
    .ends("26 Sep", "9 Oct")
    .label("Release build over the last 14 days");
let size = chart.measure(rux);
chart.show(rux, inner);
```

A smooth curve in a well with a fill that fades downwards, a dashed line at a named moment, and the last
value marked. `.compare(&values)` draws an earlier series as a quiet line. `smooth(points)` is the curve:
cubic spans whose handles sit a sixth of the way towards each neighbour, flattened to short lines.

## Readouts and pills

```rust
let readout = Readout::new("Total", "178", " s").delta("+39%", chat.slower).spark(&last_fourteen_days);
let height = readout.height(rux);
readout.show(rux, Rect::from_min_size(at, Vec2::new(180.0, height)));

let thinking = ThinkingPill::new("claude is thinking", "0:04", chat.sky).running(true);
let note = NotePill::new("Context compacted · 41k → 9k tokens");
let file = FileChip::new("Cargo.toml");
// each measures itself: let size = thinking.measure(rux); thinking.show(rux, Rect::from_min_size(at, size));
```

| Part | What it is |
|---|---|
| `Readout::new(label, value, unit)` | A card with a label, a 24 point number, an optional change (`.delta(text, colour)`), a note (`.note`) and a sparkline (`.spark`) |
| `Sparkline::new(values, colour)` | The small line on its own, 26 points tall. `spark_points(values, rect)` answers its points |
| `Legend::new(&[LegendRow])` | The rows beside a ring: a swatch, a name and a value for each slice |
| `ThinkingPill::new(words, time, colour)` | The agent's avatar with an arc going round it, what it is doing and how long. The arc holds still when `.running(false)` |
| `NotePill::new(words)` | A quiet note in a well, such as a context compaction |
| `FileChip::new(name)` | A raised pill with a file mark and a name. `.label(...)` is what a screen reader says |

## Buttons

`PillButton` has words. `RoundButton` has a mark. Each has five looks: at rest, under the pointer, held
down, focused from the keyboard, and unavailable. The grid in the picture above shows all thirty.

```rust
if PillButton::new("Apply change").primary().show(rux, rect).clicked() { /* ... */ }
if PillButton::new("Open in editor").show(rux, rect).clicked() { /* ... */ }
if PillButton::new("Copy").ghost().show(rux, rect).clicked() { /* ... */ }

if RoundButton::mark(Icon::Clock, 20.0, "History").show(rux, rect).clicked() { /* ... */ }
if RoundButton::send("Send").size(RoundSize::Send).show(rux, rect).clicked() { /* ... */ }
```

| `PillVariant` | When to use it | Examples |
|---|---|---|
| `Primary` | the one thing the card is for. The action gradient | Apply change, Allow once |
| `Secondary` (the default) | another useful action. A raised card | Open in editor, Retry |
| `Ghost` | the way out. No surface until pressed | Copy, Deny, Reject |

`PillButton` builders: `.primary()`, `.ghost()`, `.variant(PillVariant)`, `.mark(Icon)`, `.large()` (the
Agent-Tasks header size, 42 points tall), `.enabled(bool)`, and `.showing(hovered, held, ring)`, which
draws a state regardless of the pointer for a sheet of states like the one above.

| `RoundSize` | Diameter | Used for |
|---|---:|---|
| `Small` | 30 | the gear, a card's chevron, copy, stop |
| `Close` | 34 | closing the chat settings |
| `Header` (the default) | 38 | history, a new conversation |
| `Send` | 44 | send and stop |

| `RoundButton` constructor | What it draws |
|---|---|
| `RoundButton::mark(icon, size, label)` | a mark at this size |
| `RoundButton::send(label)` | the paper airplane on the action gradient |
| `RoundButton::stop(label)` | a small square in the slower colour on a raised card |
| `RoundButton::action(icon, size, label)` | a white mark on the action gradient |
| `RoundButton::new(RoundContent, label)` | any of the above, from a `RoundContent` |

`.on(true)` draws a round button pressed in, because what it opens is open. `.ghost()` gives it no
surface at rest.

### PillSwitch

```rust
if let Some(index) = PillSwitch::new(&models, Some(self.model), "Model").show(rux, rect) {
    self.model = index;
}
```

A carved track with one pill for each choice, and the chosen pill raised out of it. The chat settings
use it to choose a model. `.small()` is the 38 point version the ticket form uses for priority.

## Status

```rust
StatusWell::new(Status::Running).show(rux, well);
```

| `Status` | Mark | Colour |
|---|---|---|
| `Waiting` | a clock | the quiet ink |
| `Running` | an arc turning in a track | the sky colour |
| `Done` | a tick | the faster colour |
| `NeedsOk` | a key | the waiting colour |
| `Failed` | a cross | the slower colour |

A status well is a small round well carved into a card, 34 points by default, holding one mark.
`Status::label()` answers the words for each state. The running arc turns as a texture, the same way the
spinner does, so a running command does not rasterise its card sixty times a second.

## Cards

```rust
let (_, outcome) = CommandCard::new("cargo build --release", "Compiling 212 of 580 · 0:47", Status::Running)
    .progress(Some(0.36))
    .show(rux, rect);
if outcome.stopped {
    self.stop_build();
}

let diff = [
    Line::plain("use egui::{Color32, Pos2, Stroke};").number("3"),
    Line::plain("use kurbo::BezPath;").number("4").sign(Sign::Removed),
    Line::plain("use rux::kurbo::BezPath;").number("4").sign(Sign::Added),
];
let card = DiffCard::new("icon.rs", "1 line changed", &diff);
let height = card.height(rux);
let (_, outcome) = card.show(rux, Rect::from_min_size(at, Vec2::new(320.0, height)));
if outcome.primary {
    self.apply_change();
}
```

Every card is built the same way: raised out of the pane with a 22 point radius and 12 points of
padding; a header row of a status well, a title and a detail line, and sometimes a small round button; a
well carved into the card holding monospaced lines; and a row of pills with the way out on the left and
the thing to do on the right.

A card decides nothing. It answers which of its buttons was pressed, and the caller acts. Each card
answers how tall it is with `height(rux)`, and `show` returns `(Response, CardOutcome)`.

| Card | What it shows | Its pills |
|---|---|---|
| `CommandCard::new(title, detail, status)` | a command that ran or is running. `.progress(Some(f32))` adds a track and a stop button. `.lines(&lines, open)` adds its output under a chevron. `.words()` sets the title in the ordinary face, such as "Ran 3 commands" | none |
| `CodeBlock::new(name, detail, &lines)` | a block of code with line numbers and a copy button | none |
| `DiffCard::new(name, detail, &lines)` | a change to apply, with removed and added lines | Reject, Open in editor, Apply change. `.buttons(...)` changes or removes them |
| `PermissionCard::new(title, detail, &command)` | a request to run something | Deny, Always allow, Allow once |
| `ErrorCard::new(title, detail, &log)` | a command that failed | Show full log, Retry. `.without_retry()` removes Retry |

| `CardOutcome` field | Set when |
|---|---|
| `toggled` | the header chevron was pressed |
| `stopped` | the header stop button was pressed |
| `copied` | the header copy button was pressed |
| `ghost` | the left pill was pressed |
| `secondary` | the middle pill was pressed |
| `primary` | the right pill was pressed |

A `Line` is a run of words. `Line::plain(text)` and `Line::coloured(text, colour)` make one, and
`.number(n)`, `.sign(Sign::Removed)` or `.sign(Sign::Added)`, and `.trailing(text)` add a line number, a
diff sign and quieter words at the right, such as a duration. `Line::text()` answers the whole line for
copying.

### Card surfaces on their own

For a part of an answer that is not one of the cards above, the surfaces are functions:

| Function | What it draws |
|---|---|
| `chat_card(rux, rect, radius)` | a card raised out of the pane, `--out` |
| `chat_raised_sm(rux, rect, radius)` | a small raised surface: a pill, a chip, a bubble, `--out-sm` |
| `chat_well(rux, rect, radius)` | a well carved into a card, `--in-sm` |
| `chat_well_deep(rux, rect, radius)` | a deeper well, `--in`, for the prompt and a ring's channel |
| `card_header(rux, row, lead, title, detail, colour)` | a card's header row |

`CARD_RADIUS` is 22, `CHART_RADIUS` is 26 and `WELL_RADIUS` is 16.

## The marks

The Agent-Chat design added eight marks, which are in `Icon::CHAT`: `Clock`, `Key`, `Tick`, `Cross`,
`File`, `Rerun`, `Photo` and `Send`. The first seven are Unluminous's own window marks, and `Send` is the
paper airplane the canvas draws. [Icons](../icons.md) shows them all.

## Where to go next

- [Agent-Tasks parts](agent-tasks.md): the task board, drawn in the same values.
- [Instruments](instruments.md): the controls Unluminous's answers used before this design.
