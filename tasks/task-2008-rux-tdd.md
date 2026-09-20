# black-rainbow-labs-rux — technical design

`task-2008`. A private Black Rainbow Labs component library, written in the same user interface
technology the Unluminous editor is written in: Rust, `egui`/`eframe` for input and text, and
`vello_cpu` for the decoration `epaint` cannot draw. It ships themes, modular components, an example
page that recreates the AI Studio Image Creator design, and a desktop storybook that shows every
component in every state.

---

## 1. What is being built, and what it is measured against

`ai-service/ux-design/style-guide-and-page-designs/AI Studio Image Creator _standalone_.html` is the
reference. It is a self-unpacking bundle; `reference/` in this repository holds what was taken out of
it, so the numbers in this design can be checked without re-running the unpacker:

| File | What it is |
|---|---|
| `reference/image-creator.css` | the page's own stylesheet, 1,949 lines, every measurement in it |
| `reference/image-creator-page.jsx.html` | the page's React source |
| `reference/shared-components.jsx` | `SbIcon`, `ProjectDropdown`, `TopNav` — the shared pieces |
| `reference/neumorphic-tokens.css` | ai-service's own token file, the light theme's source of truth |
| `reference/incognito-theme.css` | ai-service's incognito override, the dark theme's source of truth |

Three things follow from the reference being CSS rather than a picture.

**Every number is readable rather than estimated.** A radius is `18px`, a shadow is
`-6px -6px 14px #FFFFFF, 6px 6px 14px #B8BFCC`, a label is `9.5px` JetBrains Mono at
`letter-spacing: 0.14em` uppercase. So "pixel perfect" here means *the library's tokens are the
stylesheet's tokens*, and a component's geometry is the rule's geometry. It is not a screenshot
comparison, and it is better than one: a screenshot can only say two pictures differ, where a token
says which declaration is wrong.

**The themes are a token swap and nothing else.** `incognito-theme.css` overrides surfaces, shadow
tones, ink, accents and gradients and changes no component rule at all — which is possible because
every rule is written in terms of the variables. The library keeps that property: a `Theme` is a
value, a component reads it, and there is no per-theme branch anywhere in `components/`.

**The elevation recipes are CSS box-shadows, so that is what the renderer draws.** Not an
approximation of them. See §4.

---

## 2. Why Rust, egui and vello_cpu

The ticket asks for "the same ui that our unluminous ide is written in (vello)", and that is the right
answer for this library for the reason it was the right answer there. `tasks/agent-tasks-vello-ui-tdd.md`
in the Unluminous repository is the long form; the short form is that **four of the things this design
is made of cannot be drawn by `epaint` at all**:

1. an **inset shadow** — every well, field, chip and pressed button in the reference has one
   (`--e-pressed`, `--e-pressed-sm`), and `epaint` has no inverted blur;
2. a **diagonal gradient** — `linear-gradient(160deg, …)` is on the project icon, the primary button,
   the dice toggle, the preset chip and the active nav item, and `epaint`'s texture brush is
   axis-aligned;
3. a **glow round a circle** — `box-shadow: 0 0 6px var(--accent-coral)` under every status dot;
   `epaint`'s `blur_width` is a rectangle's;
4. **clipping to rounded corners** — `egui`'s clip rectangle is axis-aligned and square, so a list
   scrolled inside a `24px`-radius panel pokes a square corner out of it.

`vello_cpu` draws all four. The version is **`0.1`, and it has to be**: `epaint` 0.36 already depends
on `vello_cpu` 0.1 to rasterise the glyphs in its own atlas, so using that copy adds a renderer to the
build and asking for 0.2 would add a second renderer. The GPU Vellos cannot be used: `vello` 0.10 and
`vello_hybrid` 0.2 pin wgpu 29 while `eframe` 0.36 pins wgpu 30, so their devices and textures are
unrelated types and no GPU Vello can write into a texture `eframe` can read. `vello_cpu` has no GPU
dependency, which also makes it deterministic — the same input gives the same pixels on every machine,
which is what makes a rendering test possible.

**And `multithreading` stays off.** Cargo features are additive across the graph, so enabling it
changes `RenderSettings::default()` for `epaint` too, and `epaint`'s glyph rasteriser never calls
`flush()` — every draw would panic with "attempted to rasterize before flushing". Measured in
Unluminous, not guessed.

`egui` is kept for what it is genuinely good at and what writing by hand would be a mistake: input
routing, hit testing, scroll areas, text layout and shaping, the caret and the selection in a text
box, IME, the clipboard, and the accessibility tree. The library is therefore an **egui component
library with a Vello decoration layer**, not a second immediate-mode framework.

### 2.1 What was weighed and refused

| Option | Why not |
|---|---|
| Pure `egui`/`epaint` | the four shapes above. The design *is* those shapes; without them it is a flat theme wearing the same colours. |
| `vello` (GPU) + custom wgpu integration | wgpu 29 against eframe's 30. Two wgpu versions in one process, two devices, and no shared texture type. |
| Iced / Xilem / Masonry | each owns its own widget set and its own look; the point here is that *we* own the look. Xilem's retained tree would also make "reserve a slot, draw into it, fill it in" — §4.3 — awkward. |
| A web view rendering the real CSS | it would be exact and it would be a browser. The ticket asks for a Rust component library. |
| Re-implementing text layout | `epaint`'s `Galley` already shapes, wraps, measures and hit-tests, and carries `extra_letter_spacing` and `line_height`, which is precisely the two CSS properties this design leans on. |

---

## 3. The shape of the repository

```
black-rainbow-labs-rux/
├─ Cargo.toml                  virtual workspace
├─ CLAUDE.md                   the conventions a later change has to follow
├─ README.md
├─ reference/                  what the design is measured against (see §1)
├─ tasks/task-2008-rux-tdd.md  this document
├─ crates/rux/                 THE LIBRARY — the only crate a consumer depends on
│  ├─ assets/fonts/            Inter 400/500/600/700, JetBrains Mono 400/500/600, + licences
│  └─ src/
│     ├─ lib.rs                `Rux`, `RuxState`, the prelude
│     ├─ theme/                tokens, elevations, gradients, the two built-in themes, the registry
│     ├─ render/               Decor, Chrome, Canvas, the layer/slot helper
│     ├─ text.rs               weighted families, the type scale, letter-spacing, measurement
│     ├─ icon/                 the icon set, the SVG path data, the tinted texture atlas
│     ├─ layout.rs             flex row/column/grid — measured and arranged with no window
│     └─ components/           one file a component
├─ examples/                   crate `rux-examples`, `[lib] path = "pages/mod.rs"`
│  └─ pages/image_creator.rs   the reference page, rebuilt out of the components
└─ storybook/                  crate `rux-storybook`, the desktop app
```

`examples/pages/` is a folder because the ticket names it, and Cargo is told where the library root is
rather than the sources being moved under a `src/` nobody asked for.

**The dependency points one way.** `rux` depends on `egui`, `eframe`'s re-exports and `vello_cpu`, and
on nothing in this repository. `rux-examples` depends on `rux`. `rux-storybook` depends on both. So a
consumer takes `rux` alone, and neither the storybook nor the example pages can add anything to the
library by being where they are.

---

## 4. The renderer

This is `crates/rux/src/render/`, and it is the part carried over from Unluminous — with one
deliberate change, described in §4.2.

### 4.1 Three types, and the middle one is the seam

**`Chrome`** is what a component talks to. It says what kind of surface it is drawing — a rounded
rectangle, a surface at an elevation, a disc, a ring, a line, a rounded clip — and records it. A
component never holds a shadow offset.

**`Decor`** is what that records: seven kinds of shape and a clip, in points, absolute in the
window's coordinates. A plain `Copy` value with no behaviour, so a drawing can be asserted on with no
window, no graphics card and no fonts, and so the cache in `Canvas` is a comparison of two lists.

**`Canvas`** replays a `Decor` list into a `vello_cpu::RenderContext`, rasterises into a `Pixmap` of
premultiplied RGBA8 — which is `egui::Color32` byte for byte — and uploads it as one texture painted
behind that layer's widgets.

### 4.2 The elevation is the CSS box-shadow, not an approximation of it

Unluminous derives its elevation: a black shadow at an alpha below and right, and the surface's own
colour *lifted* above and left. That was right there, because its reference was a picture and its
palette had to stay closed.

Here the reference is the stylesheet, and the stylesheet says exactly what the shadow is:

```css
--e-raised:    -6px -6px 14px var(--shadow-light), 6px 6px 14px var(--shadow-dark);
--e-pressed:   inset 4px 4px 8px var(--shadow-dark), inset -4px -4px 8px var(--shadow-light);
```

Both colours are **fully opaque** — `#FFFFFF` and `#B8BFCC` in the light theme, `#2F353E` and
`#14171B` in the dark one. Neumorphism works because the Gaussian falls off, not because the colour is
faint. So the library stores the declaration and draws the declaration:

```rust
pub struct Shadow { dx: f32, dy: f32, blur: f32, spread: f32, colour: Color32, inset: bool }
pub struct Elevation { shadows: [Option<Shadow>; 4] }   // Copy; the deepest recipe here has four
```

Four conversions are all that stand between a CSS declaration and a `Decor`:

- **blur radius → standard deviation is `R / 2`.** CSS defines the blur radius as twice the Gaussian's
  standard deviation, and `fill_blurred_rounded_rect` takes the standard deviation. `--e-raised`'s
  `14px` is therefore `σ = 7`, which is the number Unluminous arrived at independently.
- **spread expands the shape before the blur**, so the rectangle handed to the renderer is
  `rect.translate(dx, dy).expand(spread)`.
- **order is back to front, first on top.** CSS paints the first shadow in the list above the ones
  after it, so the list is replayed in reverse.
- **an inset shadow is clipped to the border box.** `fill_blurred_rounded_rect`'s inverted form paints
  the *complement* of the blur — opaque outside the rectangle, fading to nothing inside it — so an
  unclipped well washes its colour across the whole layer. CSS clips an inset shadow to the border box
  for exactly this reason.

And one optimisation that is only sound when it is checked: **an outer shadow is cut to the band
around its own shape** — a `Clip { outside: true }` with an even-odd frame — because the surface is
painted over it. Unluminous measured a Gaussian evaluated over a third of a million pixels that were
then covered up at 3.3 ms for a single lane. The cut is only applied when the fill is **opaque**; a
translucent surface really does show its own shadow through itself, and `Chrome::surface` asks the
fill rather than assuming.

### 4.3 A layer reserves its slot before it draws into it

The decoration is one texture painted *behind* a layer's widgets, so the shape has to be reserved in
the painter's list before any widget is added and filled in afterwards, once the drawing is finished
and the `Decor` list is complete. That is `egui::Shape::Noop` and `Painter::set`, and it is what
`rux::render::layer` does:

```rust
rux::layer(ui, state, id, area, |rux| { /* draw components */ })
```

It reserves the slot, makes a recording `Chrome`, runs the closure, rasterises, and fills the slot in.

**Every popup and every modal is its own layer**, because `egui` draws them on foreground layers the
parent's painter cannot reach and `Painter::set` writes to the layer its own painter belongs to. So a
dropdown menu, a context menu and a modal each open an `egui::Area` and call `layer` inside it. That is
`ChromeSlot` in Unluminous, made the ordinary path rather than a special case for one modal.

### 4.4 Nothing that runs once a frame may do the work twice

The `Decor` list is rebuilt every frame — a few hundred `Copy` values into a `Vec` — and the
**rasterisation** happens only when that list, the layer's rectangle or the pixel density changed. The
list is kept and compared rather than hashed: a 64-bit collision would serve stale pixels, and
comparing a few hundred `Copy` values costs less than the rasterisation it is deciding to skip.

Two rules follow, and both are about what must *not* reach the `Decor` list:

- **A flat fill with no shadow, no gradient and no rounded clip is drawn by `egui`**, because `epaint`
  draws those perfectly well and an `egui` shape costs nothing to change. Every hover wash in the
  reference — `.proj__item:hover { background: rgba(0,0,0,0.025) }` — is one of these, so moving the
  pointer down a menu does not rasterise anything.
- **An icon is a tinted texture, not a path in the canvas.** See §6. A hover that changes an icon's
  colour is then a vertex colour and not a redraw.

What legitimately does rasterise again is a `:active` press (the shadow really changes), a menu opening
or closing, a scroll, and a resize.

### 4.5 The cap on resolution

`MAX_SCALE = 1.5` pixels a point. The decoration's highest-frequency feature is the edge of a rounded
rectangle and everything else in it is a Gaussian or a gradient — exactly the content that survives
being drawn slightly coarser and filtered up. The text over it is unaffected, because `egui` draws the
text at the display's own resolution. Rasterising is per pixel, so the cap is the difference between a
board that drops frames on a 2× display and one that does not.

---

## 5. Text

`epaint` lays out and shapes the text; the library decides which face, at what size, with what
spacing.

**Seven families are registered, one per weight**, because `ab_glyph` has no variable-font axes and
the design uses four weights of Inter and three of JetBrains Mono as distinct faces:
`rux-sans-400/500/600/700` and `rux-mono-400/500/600`. The static TTFs are vendored under
`crates/rux/assets/fonts/` with their licences (Inter and JetBrains Mono are both OFL) and embedded
with `include_bytes!`, so a consumer needs nothing installed and nothing is ever fetched.

**`letter-spacing` and `line-height` are carried through**, which is what makes the typography match
rather than merely resemble. `TextFormat::extra_letter_spacing` takes points, and CSS gives `em`, so
the conversion is `em × size`; `--font-mono` kickers at `0.14em` on a `9.5px` face are `1.33` points of
extra tracking, and the body's `-0.005em` is a hair of negative tracking. `TextFormat::line_height`
takes points too, so `line-height: 1.6` is `1.6 × size`.

`rux::text::Style` is one value carrying family, weight, size, tracking, line height, colour, and
whether the text is upper-cased — so a "kicker" is a named style rather than five arguments repeated at
forty call sites. The named styles are the reference's own: `display`, `h1`, `h2`, `h3`, `body`,
`label`, `caption`/`kicker`, `mono`, `mono_small`.

---

## 6. Icons

The reference's icons are `SbIcon`: a 24×24 `viewBox`, `fill="none"`, `stroke="currentColor"`,
`stroke-width` defaulting to `1.8`, round caps and round joins. Forty-odd of them, each a small piece
of SVG path data.

**The path data is kept verbatim.** An icon redrawn from a recollection of what it looked like is a
drawing of something else wearing the right name — which is a rule Unluminous's `task-1949` had to
learn the hard way. So `icon/paths.rs` holds the `d` strings exactly as `shared-components.jsx` has
them, and the handful of `<circle>` and `<rect>` children keep their own typed shapes rather than being
converted by hand into path data. A part records whether it is stroked or filled, because `play`,
`pause`, `dots` and `grip` fill and everything else strokes.

**An icon is rasterised once per (icon, size, stroke width, pixel density) into an alpha texture, then
tinted.** That is how an icon font works, and it buys the two things §4.4 asks for: a hover that
changes an icon's colour costs a vertex colour rather than a rasterisation, and the same icon at the
same size anywhere in the window is one texture. The rasteriser is `vello_cpu`, so the caps, the joins
and the curve flattening are the renderer's own and match everything else drawn here.

The one icon that cannot be tinted is the spinner's arc, whose stroke is a **gradient** and which
**rotates**. It is rasterised in colour, and the rotation is applied to the four vertices of the mesh —
which is exactly what the CSS animation does, since it rotates the whole `<svg>` and the gradient with
it.

---

## 7. Layout

The reference is flexbox and grid, so the library has a small solver rather than hand-computed
coordinates. It is a measure-and-arrange pass that produces rectangles, and it runs with no window,
which is what makes it a unit test.

```rust
let [icon, body, chev] = Flex::row(area)
    .pad(Pad::sides(8.0, 14.0, 8.0, 8.0))
    .gap(12.0)
    .cross(Cross::Centre)
    .fixed(36.0)
    .fill(1.0)
    .hug(chev_width)
    .solve();
```

`Length` is `Fixed`, `Hug` or `Fill(weight)`, which is `flex-basis`/`flex-grow` with the names a reader
of the CSS will recognise. `Grid` covers `grid-template-columns: 1fr 1fr` with row and column gaps —
the controls panel and the results canvas are both grids. `Pen` is a vertical cursor for a column of
sections, which is what a storybook page and a settings page are.

Nothing about the solver is clever. It exists so that the page in `examples/pages/` reads like the JSX
it was built from instead of like a list of magic numbers, and so that a component can answer how big
it wants to be before anybody decides where to put it.

---

## 8. The components

Every component is the same shape, and that shape is the library's contract:

```rust
Button::new("Create")
    .variant(ButtonVariant::Primary)
    .icon(Icon::Spark)
    .size(ButtonSize::Large);

fn measure(&self, rux: &Rux) -> Vec2;                 // what it wants, before anybody places it
fn show(self, rux: &mut Rux, rect: Rect) -> Response; // draws into the rect it is given
```

Three rules hold for all of them:

- **A component decides nothing about the application.** It draws, and it answers what happened —
  `Response` is `egui`'s own, with the component's extra outcome beside it where there is one (which
  row of a menu was chosen, whether a collapsible was toggled). State that has to survive a frame is
  the caller's, passed in by `&mut`, which is what makes a component usable twice on one screen.
- **A component reads the theme and never a colour.** There is no `Color32::from_rgb` anywhere in
  `components/`. A name a theme does not have is a name the theme should gain.
- **A component sets the cursor the stylesheet sets.** `cursor: pointer` is
  `CursorIcon::PointingHand`, `cursor: text` is `CursorIcon::Text`, `cursor: grab` is
  `CursorIcon::Grab`. The ticket asks for the correct cursor and the stylesheet says which.

### 8.1 The set, and the rule in the reference each comes from

| Component | Reference rule | States |
|---|---|---|
| `TopNav` | `.topnav`, `.topnav__item` | item active (gradient, label shown) / inactive (icon only) / hover |
| `Button` | `.ic-create`, `.render-btn`, `.ic-mini-btn`, `.add-scene`, `.ic-fbtn`, `.ic-clear` | primary / secondary / mini / ghost / danger / mint · default, hover, active, disabled · with and without icon |
| `IconButton` | `.toolbtn`, `.iconbtn`, `.dock__btn`, `.ic-chev`, `.ic-prompt-shuffle`, `.ic-modal__close` | sizes 24 / 28 / 32 / 40 / 48 · raised / primary · with dot · with count badge |
| `Select` | `.ic-select`, `.ic-select__menu`, `.ic-select__item` | closed / open / open upwards / mono / empty placeholder / long value elided |
| `ProjectMenu` | `.proj`, `.proj__menu` | closed / open, selected row, footer action |
| `Modal` | `.ic-modal-backdrop`, `.ic-modal`, head / body / foot | closed / open · with a sidebar · with tabs · with a footer |
| `TextInput` | `.ic-form-input`, `.search-field`, `.scene__title-input` | empty / filled / focused / with a leading icon |
| `TextArea` | `.ic-prompt`, `.ic-form-textarea` | empty / filled / with a character count / with a corner button |
| `Stepper` | `.ic-count`, `.num-input` | at minimum / mid / at maximum · both ticks |
| `Loader` | `.ic-spinner`, `.ic-tile--loading`, `.ic-elapsed` | spinner / tile with a spinner and an elapsed chip / progress bar / skeleton |
| `Chip` | `.meta-chip`, `.volume-chip`, `.scene__badge`, `.ic-preset-chip`, `.ic-saveto` | sunken / raised / accent-gradient with a dismiss / with a glowing dot |
| `Segmented` | `.view-toggle`, `.ic-modal-tabs` | two / three / four items, each index active |
| `Toggle` | `.ic-dice` | off / on |
| `Panel` | `.ic-panel`, `.scene`, `.ic-prompt-wrap`, `.timeline` | raised card / sunken well / side panel expanded and collapsed |
| `SubGroup` | `.ic-sub` | open / collapsed, four accent variants |
| `Labelled` | `.ic-labelled`, `.ic-ctrl` | label above a control, with and without a trailing toggle |
| `Avatar` | `.avatar` | with and without a status dot |
| `ListItem` | `.ic-modal__list-item`, `.proj__item` | default / hover / active / with a trailing action |
| `Tile` | `.ic-tile`, `.scene__image` | empty / loading / with an image / dark dropzone with hover overlay |
| `Scrollbar` | `.ic-panel__scroll::-webkit-scrollbar` | idle / hovered |

"etc." in the ticket is what the last nine rows are: the reference has them, so the library has them.

### 8.2 The things a component library has to get right that a page does not

A page can hard-code a width; a library cannot. So each component measures itself, elides text that
does not fit (`.ic-select__value` and `.proj__name` both say `text-overflow: ellipsis`), keeps its
minimum (`.ic-count` says `width: 66px; flex-shrink: 0` and means it), and draws its focus ring from
the theme rather than from `egui`'s default.

---

## 9. The themes

`Theme` is a plain value: surfaces, shadow tones, ink, accents, semantic colours, radii, elevations,
gradients, and the font families. Two ship, and both are transcriptions rather than interpretations:

- **`light-neumorphic`** — `reference/neumorphic-tokens.css`, which is ai-service's normal mode.
- **`dark-neumorphic`** — the same file with `reference/incognito-theme.css` applied over it, which is
  ai-service's incognito mode. Only the values differ; not one recipe does.

A theme is named and registered, so the storybook's dropdown is a list of what exists rather than a
list written twice. `Theme::named("dark-neumorphic")` is how a consumer picks one, and a name that does
not exist is refused with the list — which is the rule Unluminous keeps everywhere a manifest names
something built in.

**A theme cannot add a token.** The struct is the closed list, which is what makes it true that a
component reads a name and gets the right colour in both themes. A token the design needs and the
struct has not got is a change to the struct and to both themes, in one commit.

---

## 10. The example page

`examples/pages/image_creator.rs` rebuilds the reference page out of the components: the top bar with
the project dropdown, the nav and the save-to chip; the two collapsible panels with the prompt well,
the mini buttons, the dimension row, the stepper, the workflow select and the Create button; the
controls panel with its preset chip, its four collapsible sub-groups and its sixteen labelled selects
with dice toggles; the results canvas with its tiles, spinners and elapsed chips; and the Prompt
Manager modal.

It is a page rather than a picture: the selects open, the panels collapse, the sub-groups fold, the
stepper counts, the dice toggle, the modal opens on `Prompts` and closes on `Escape`, and the prompt
text is editable with its character count following it. That is what makes it a test of the components
rather than a screenshot of them, and it is the thing that finds the component that cannot do what the
page needs.

**It holds its own state and takes it by `&mut`**, so the storybook can show it without the library
owning a page's model.

---

## 11. The storybook

A desktop application, `storybook/`, and the design is deliberately the one React's Storybook settled
on, because it is the one people already know how to read.

```
┌───────────────────────────────────────────────────────────────┐
│ [ theme ▾ ]  RUX Storybook                    42 stories      │  ← theme dropdown, top left
├───────────────┬───────────────────────────────────────────────┤
│ [ filter    ] │  Button                                       │
│ FOUNDATIONS   │  .ic-create / .ic-mini-btn / .ic-fbtn         │
│   Colour      │                                               │
│   Type        │  ── Primary ───────────────────────────────   │
│   Elevation   │   [ ✦ Create ]   [ ✦ Create ]   [ ✦ Create ]  │
│   Icons       │    default        hover          disabled     │
│ COMPONENTS    │                                               │
│   Button    ◂ │  ── Mini ──────────────────────────────────   │
│   Select      │   [ 🗀 Prompts ]  [ ⭳ Save ]                   │
│   …           │                                               │
│ PAGES         │                                               │
│   Image Cre…  │                                               │
└───────────────┴───────────────────────────────────────────────┘
```

- **The theme dropdown is at the top left**, which is where the ticket asks for it, and it is the
  library's own `Select` — so the storybook is its own first consumer and a fault in `Select` is
  visible the moment the application starts.
- **The filter is the library's `TextInput`**, and it matches a story's name, its section and its
  states by subsequence, so `crtbtn` finds `Create Button`. A filtered tree hides empty sections.
- **A story is a list of named states**, each drawn live and each interactive. A state is a closure, so
  "open" really is the open dropdown and not a picture of one.
- **The pages section hosts `examples/pages/`**, full width, interactive, so the page can be used
  rather than looked at.
- **Foundations pages** show the theme itself: every colour with its token name and hex, the whole type
  scale at its real size and tracking, every elevation on a surface, and every icon at four sizes with
  its name — which is how a mark is judged at the size it is seen at rather than at eight times life
  size.

State lives in the storybook, keyed by story and state name, so two instances of the same component on
one screen keep their own.

---

## 12. Performance and memory

The ticket asks for both, and the design's answers are the ones measured in Unluminous:

| Concern | The answer |
|---|---|
| A still frame | Costs a `Decor` list comparison and nothing else. No rasterisation, no allocation, no texture upload. |
| A hover | Costs a vertex colour. Icons are tinted textures and flat hover washes are `egui` shapes; neither reaches the canvas. |
| A changed frame | One rasterisation of the layer's own bounding box — not the window's — cut to `MAX_SCALE` pixels a point. |
| Outer shadows | Cut to the band around their own shape when the fill is opaque, which is the 3.3 ms a lane Unluminous measured. |
| Icons | One texture per (icon, size, stroke, density), shared by every use. An alpha mask, so one byte a pixel. |
| Fonts | Seven static faces, embedded, ≈2.5 MB, parsed lazily by `ab_glyph`. |
| Popups | A popup that is not open has no layer, no `Chrome` and no canvas. |
| Canvases | One per layer, dropped the frame after its layer stops being drawn. |

`storybook --measure` prints what a frame costs and what the process is holding, so the numbers stay
measured rather than remembered — the same argument `examples/frame_cost.rs` makes in Unluminous.

---

## 13. Testing

Four layers, and the first three need no graphics card:

1. **`layout`** — the solver, asserted on arithmetic a reader can check.
2. **`theme`** — every token in both themes against the values in `reference/`, so a typo in a hex
   string is a failing test rather than a slightly wrong colour.
3. **`render`** — a `Decor` list is a value, so "a raised surface records the stylesheet's two shadows,
   in the stylesheet's order, at the stylesheet's offsets" is a unit test. So is "an outer shadow is
   cut to its band only when the fill is opaque", and "a still list does not rasterise again".
4. **The window** — `egui_kittest` builds the real application, feeds it real events, renders through
   `wgpu` and writes a PNG per test. Those are looked at by a person once and then become the baseline.
   `vello_cpu`'s SIMD level is pinned to `Level::baseline()` in the tests, because the widest level a
   processor has is not promised to be bit-identical to another's.

---

## 14. Deliberately not in this version

Each with its reason, so the next person does not have to rediscover the argument:

- **A light (non-neumorphic) or high-contrast theme.** The ticket names two themes and both are
  neumorphic. The struct is ready for a third; nothing about the design assumes there are two.
- **Animation.** The reference has transitions (`0.12s`–`0.28s` on hover, shadow and panel width) and
  they are worth having. This version draws the end states, because an animation system is a frame
  clock, an easing library and a rule about what a `Decor` cache does while something is moving — and
  getting the static geometry exactly right first is what the animation would then interpolate between.
- **Right-to-left, and scripts outside Latin.** The vendored faces cover Latin; `epaint` would need a
  fallback chain and the layout solver would need a direction.
- **A theme loaded from a file at run time.** A theme is a Rust value, which is what makes the closed
  token list checkable. A file format is a parser, a refusal path and a version.
- **Screen-reader labels on every component.** `egui`'s `accesskit` is on and `Response::widget_info`
  is called where the component has an obvious name, but the full pass — roles, relationships, focus
  order — is its own piece of work.
