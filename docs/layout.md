# Layout

A rux component draws into the rectangle it is given. `rux::layout` is how a page divides its area into
those rectangles. It is a small flexbox and grid solver that produces `egui::Rect` values and nothing
else, so it runs with no window and every rule in it is a unit test.

The solver reads like the CSS the design was written in. A row of `display: flex; gap: 12px;
align-items: center` is `Flex::row(area).gap(12.0).cross(Cross::Centre)`.

## Measure, then place

Every component answers how big it wants to be before anything places it:

```rust
let create = Button::new("Create").primary().icon(Icon::Spark);
let wanted = create.measure(rux);                       // a Vec2, in points

let [field, button] = Flex::row(row)
    .gap(12.0)
    .cross(Cross::Centre)
    .fill(1.0)                // the field takes what is left
    .hug(wanted.x)            // the button takes what it asked for
    .solve();

TextInput::new(&mut self.query).hint("Search").show(rux, field);
if create.show(rux, button).clicked() { /* ... */ }
```

`measure` takes `&self` and `show` takes `self`, so the same builder value is measured and then shown.

## `Flex`

`Flex::row(area)` or `Flex::column(area)`, then one call per item, then `solve()`.

| Item | CSS | What it takes |
|---|---|---|
| `.fixed(points)` | `flex: 0 0 <n>`, or `flex-shrink: 0` | exactly this many points |
| `.hug(points)` | `flex: 0 0 auto` | what the item measured, which is the number you pass |
| `.fill(weight)` | `flex-grow: <weight>` | a share of what is left |
| `.sized(Vec2)` | `align-self: center` with a set size | its length on the main axis and its own size across it, centred |

| Setting | CSS |
|---|---|
| `.gap(points)` | `gap` |
| `.pad(Pad)` | `padding` |
| `.cross(Cross)` | `align-items`: `Stretch` (the default), `Centre`, `Start`, `End` |
| `.align(Cross)` | `align-self`, for the item added just before it |
| `.justify(Justify)` | `justify-content`: `Start`, `Centre`, `End`, `Between`. It only matters when no item is `fill` |

`solve()` returns a fixed size array, so the item count is checked where the rectangles are named:

```rust
let [icon, body, chevron] = Flex::row(area)
    .pad(Pad::sides(8.0, 14.0, 8.0, 8.0))
    .gap(12.0)
    .cross(Cross::Centre)
    .fixed(36.0)
    .fill(1.0)
    .hug(15.0)
    .solve();
```

`solve_all()` returns a `Vec<Rect>` for a row whose length is only known at run time, such as a row of
chips.

The solver shares out space in two steps:

1. A `fill` item shares what is left after the `fixed` and `hug` items, the gaps and the padding. With
   nothing left a `fill` item comes out zero wide. It is never negative, so a collapsed panel gives
   valid rectangles.
2. When the items still do not fit, every `hug` item shrinks in proportion, which is
   `flex-shrink: 1`. A `fixed` item never shrinks, which is `flex-shrink: 0`. `Stepper` depends on that
   for its `width: 66px; flex-shrink: 0` from the reference, because a number squeezed to nothing is a
   number nobody can read.

`.sized(Vec2)` adds a `hug` item with its own size across the axis, centred. A round button in a row of
stretched controls is one.

## `Pad`

| Constructor | CSS |
|---|---|
| `Pad::all(20.0)` | `padding: 20px` |
| `Pad::axes(16.0, 28.0)` | `padding: 16px 28px`, top and bottom then left and right |
| `Pad::sides(top, right, bottom, left)` | `padding: top right bottom left` |
| `Pad::NONE` | no padding |

`pad.inside(rect)` shrinks a rectangle by the padding. `pad.horizontal()` and `pad.vertical()` add up the
two sides.

## `Grid`

```rust
let columns = Grid::auto_fill(area.width(), 260.0, 16.0);   // repeat(auto-fill, minmax(260px, 1fr))
let grid = Grid::new(area, columns).gap(16.0, 16.0).row_height(Tile::height_at(260.0));
for (index, result) in results.iter().enumerate() {
    Tile::new().show(rux, grid.cell(index));
}
let height = grid.height_for(results.len());   // for the scrollbar, before anything draws
```

`Grid::new(area, columns)` is `grid-template-columns: repeat(<columns>, 1fr)`. `cell(index)` counts
across, then down. The example page's results canvas and its controls panel use it.

## `Pen`

A `Pen` runs down a column. A settings page, a storybook page and a panel of stacked sections are all
one.

```rust
let mut pen = Pen::new(area);
let heading = pen.strip(22.0);      // a strip this tall across the whole width
pen.gap(14.0);                      // space
let field = pen.strip(40.0);
let rest = pen.rest();              // whatever is left under the pen
let used = pen.used();              // how far it came, which a scrollbar needs
```

## On a phone

A control that has to be pressed with a finger is at least 44 points tall on a phone. That happens in
`measure`, so a layout built from `measure` is right on both a desktop and a phone with no change. See
[Mobile](mobile.md#a-touch-target-grows-the-layout-box) for why the layout box grows and the hit box does
not.

## Where to go next

- [Components](components/README.md): what to put in the rectangles.
- [Getting started](getting-started.md): a whole window, from `main` to the first frame.
