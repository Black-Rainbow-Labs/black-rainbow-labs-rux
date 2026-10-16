# Working on black-rainbow-labs-rux

Read this before changing anything. It records the conventions the code already follows, so that a
later change looks like the rest of the code rather than like a second style laid over it.

`tasks/task-2008-rux-tdd.md` is the design and records what was weighed and refused. `reference/` is
what the whole thing is measured against.

## The reference is a dependency, not a folder somebody once looked at

`reference/` holds the stylesheets and source this library was transcribed from:
`image-creator.css`, `image-creator-page.jsx.html`, `shared-components.jsx`,
`neumorphic-tokens.css` and `incognito-theme.css`.

**`crates/rux/tests/reference.rs` reads those files at test time and parses them.** Every colour in
both themes, every radius, every shadow's offsets and blur, the padding and gap of every component
the design names, the whole type scale and every icon name is compared against the declaration it
came from. So:

- **A number in this library comes from a declaration, and the comment beside it names the rule.**
  `// `.ic-count { width: 66px }`` is not decoration; it is how the next reader checks the number.
- **Changing a token means changing the reference or the test fails**, which is the right way round:
  the stylesheet is the source of truth and the library is a transcription of it.
- **Do not "improve" a measurement.** If a padding looks wrong, the reference is wrong and the fix is
  a conversation, not a nudge.

## The three rules every component keeps

1. **A component decides nothing about the application.** It draws, and it answers what happened.
   State that has to survive a frame is the caller's, passed in by `&mut`. That is what lets the same
   component appear twice on one screen, and it is why `Select` takes a `&mut SelectState` rather than
   keeping one.
2. **A component reads the theme and never a colour.** There is no `Color32::from_rgb` anywhere in
   `components/`. A name the theme has not got is a name the theme should gain — which means a change
   to `Theme`, to both themes, and to `reference.rs`, in one commit.
3. **A component sets the cursor the stylesheet sets.** `cursor: pointer` is
   `CursorIcon::PointingHand`, `cursor: text` is `CursorIcon::Text`, `cursor: grab` is
   `CursorIcon::Grab`.

And the shape is always the same:

```rust
fn measure(&self, rux: &Rux) -> Vec2;                 // what it wants, before anybody places it
fn show(self, rux: &mut Rux, rect: Rect) -> Response; // draws into the rect it is given
```

## A theme says what a name means; it cannot add a name

The token list in `theme/tokens.rs` is closed, and that is the whole design: `incognito-theme.css`
re-skins the entire ai-service interface by overriding values and **changing no rule at all**, and
this library keeps that property. `the_dark_theme_changes_no_recipe_at_all` is the test — both themes
must declare identically shaped elevations and identical gradient angles.

So there is no per-theme branch in `components/`, and there must never be one. `Theme::dark` is read
in exactly one place, `Tile`, and only to decide the ink drawn over a dropzone that is dark in both.

## What goes in the canvas, and what does not

The decoration is **one `vello_cpu` texture per layer**, painted into a slot reserved before any
widget. `Canvas::texture_for` compares the whole `Decor` list against the one the current texture was
made from, so a frame where the list is identical costs a comparison and nothing else.

That gives one rule, and it is the most important rule in the repository:

> **Anything that changes every frame must not reach the `Decor` list.**

Three things follow, each measured rather than assumed:

- **An icon is a tinted texture**, rasterised once per (mark, size, stroke, density) into an alpha
  mask. The reference changes an icon's colour on hover in nineteen rules; as decoration, every one of
  those would rasterise the whole layer.
- **The spinner's arc is a texture too, turned by rotating four vertices.** As a `Decor::Arc` whose
  angle moved every frame, a page with three spinners on it cost **822 ms of processor time a
  second** — most of a core to draw three circles. As a texture it is 253 ms/s with the canvas
  rasterised **once**, and the rest is egui's own pass.
- **A flat fill with no shadow is drawn by `egui`.** Every hover wash in the reference is one of
  these, so moving the pointer down a menu rasterises nothing.

What legitimately does rasterise again: a `:active` press, a menu opening, a scroll, a resize, a
theme change.

### And the ground goes **into** the chrome, not onto the painter

`Rux::ground` records a `Decor::Rect`. It looks like it should be a `rect_filled` and it must not be:
the decoration's texture is filled into a slot reserved *before* any widget is added, so anything
painted with `egui` lands on top of it — and a ground painted over the decoration hides every shadow
in the layer. It looks exactly like the renderer not working, which is how it was found.

## Every popup is its own layer

`egui` draws foreground areas on layers the parent's painter cannot reach, and `Painter::set` writes
to the layer its own painter belongs to. So a dropdown menu, a context menu and a modal each open an
`egui::Area` and call `render::layer` inside it — `Rux::overlay` is that, and it is the ordinary path
rather than a special case.

## An elevation is a CSS box-shadow

Not an approximation of one. Four conversions and nothing else:

- **blur radius → standard deviation is `R / 2`**, because CSS defines the radius as twice the
  Gaussian's standard deviation;
- **spread expands the shape before the blur**;
- **the list is replayed in reverse**, because CSS paints the first shadow in a declaration on top;
- **an inset shadow is clipped to the border box**, because `fill_blurred_rounded_rect`'s inverted
  form paints the *complement* of the blur and an unclipped well washes its colour across the layer.

And one optimisation that is only sound when it is checked: an outer shadow is cut to the band around
its own shape **only when the fill is opaque**, because a translucent surface really does show its
own shadow through itself.

## The version of `vello_cpu` is epaint's, and `multithreading` stays off

`epaint` 0.36 already depends on `vello_cpu` 0.1 to rasterise the glyphs in its own atlas, so this
uses the copy that is already in the build; asking for 0.2 would put a second renderer in the binary.
Follow epaint when egui is next upgraded.

Cargo features are additive across the whole graph, so enabling `multithreading` changes
`RenderSettings::default()` for epaint too — and epaint's glyph rasteriser never calls `flush()`,
which the multithreaded dispatcher requires. Measured in Unluminous: every draw panicked with
"attempted to rasterize before flushing", from egui's own text.

## Weight is a face, not an axis

`ab_glyph` has no variable-font axes, so seven static faces are registered, one per weight. A weight
the library does not vendor falls back to the heaviest it has, and `family_name` says so rather than
doing it silently.

`letter-spacing` and `line-height` are carried through. CSS gives `em` and `epaint` takes points, so
the conversion is `em × size` — and without it a 9.5px mono kicker at `0.14em` is a different design.

## The icon path data is verbatim

An icon redrawn from a recollection of what it looked like is a drawing of something else wearing the
right name. `icon/paths.rs` holds the `d` strings exactly as `SbIcon` has them, and
`every_icon_s_path_data_parses` and `every_mark_stays_inside_its_own_view_box` are what keep a
transcription error from being a smudge nobody notices.

**A mark is judged at both sizes or it is not judged.** The storybook's icon sheet draws every mark at
24 points *and* at 13 — the size it is really drawn at inside a control — because a sheet at three
times life size hides exactly the faults that matter.

## Driving the window never takes the keyboard

`--background` opens without taking the foreground, `--story` and `--filter` reach any state, and
`--open` opens the dropdowns and the dialog. Between them a capture pass needs **no synthetic input at
all**, which matters on Windows: activating a window switches the virtual desktop with it and takes
the keyboard out of whatever the person was typing into.

`tools/capture-window.ps1` is `PrintWindow` with `PW_RENDERFULLCONTENT`, so it photographs a window
that is behind another one or has never been activated. `tools/capture-stories.ps1` walks every story
in both themes. Screenshots go under `_agent_output/`, which is gitignored.

## Testing

Four layers, and the first three need no graphics card:

1. **`layout`** — the solver, asserted on arithmetic a reader can check.
2. **`theme` and `reference`** — every token against the file it was transcribed from.
3. **`render`** — a `Decor` list is a value, so "a raised surface records the stylesheet's two
   shadows, in the stylesheet's order, at the stylesheet's offsets" is a unit test. So is "an outer
   shadow is cut to its band only when the fill is opaque" and "a still list does not rasterise
   again".
4. **The window** — build it, run it, and look. `pwsh tools/capture-stories.ps1`.

`Canvas::deterministic` and `IconAtlas::deterministic` pin `vello_cpu`'s SIMD level to
`Level::baseline()`, because the widest level a processor has is not promised to be bit-identical to
another's — so a picture accepted on one machine would fail on another for a reason that is not a
fault here.

## Writing

Plain sentences. Say what the code does and why a decision was made, once. Every module has a comment
at the top saying what it is for, and a decision a reader might disagree with is recorded where it was
made rather than left to be rediscovered.

British spelling in prose, and the American spelling where a name in the code already uses it, such as
`color` in `egui`.

## The phone is a `Screen`, and it is the only thing a component may ask about the device (task-2032)

`crates/rux/src/screen.rs` holds three facts — how wide, what is covering the edges, whether the thing
pointing at it is a finger — and an application sets them once a frame:

```rust
state.set_screen(Screen::phone(size, insets));   // or Screen::desktop(size), or Screen::read(ctx)
```

**`Screen::desktop` is exactly the behaviour this library had before it knew about phones**: a mouse, no
insets, `min_target() == 0.0`, and every `target_height(x)` returning `x`. That is why
`crates/rux/tests/touch.rs` can assert every desktop measurement against the number it was, as a table.
Keep that property: a change that makes a desktop measurement move is a change to the reference.

Two rules follow from it, and both are easy to get wrong:

1. **A touch target grows the LAYOUT box, not the hit box.** In touch a control's `measure()` returns at
   least 44 points and what is drawn does not change — a 28 point circle stays a 28 point circle, centred
   in a 44 point box. Widening only the rectangle handed to `interact` makes two neighbouring controls
   claim overlapping targets, and egui gives an overlapping point to whichever widget asked last, so the
   second one silently eats a strip of the first.
2. **`Visual::of` takes the pointer**, and `Pointer::Touch` never answers `Hover`. Anything that reads
   `Response::hovered` to decide a colour goes through `components::hovering` instead, for the same
   reason: a finger does not hover, and egui keeps a touch's last position after the finger lifts.

## Run the tests, and remember they are a DEBUG build (task-2032)

`crates/rux/tests/draw.rs` draws every component, in both themes, on a desktop screen and on a phone
screen, with no window. It exists because a defect lived in `rux::icon` from the day it was written and
**no release build could show it**: `epaint::Mesh::colored_vertex` carries a `debug_assert` that the mesh
has no texture, and both places that draw a turned mark called it and wrote the texture coordinate over
the vertex afterwards. Right picture with assertions off; dead process with them on.

Every test here ran the geometry rather than the drawing, and every program that ran was `--release`, so
nothing had executed that line with assertions on until the Android application's first frame did. If a
component gains a drawing path, it goes in that file.

## The Agent-Chat parts are the one place the design is not the reference (task-2235)

Unluminous's chat pane is drawn to a Claude Design canvas ("Unluminous", page Agent-Chat), not to
`reference/`. Its parts live in `components/round.rs`, `status.rs`, `dial.rs` and `cards.rs`, and its values
are `theme::Chat`. Two rules keep that from becoming a second style:

- **`Chat` is worked out, not transcribed.** `Chat::lit_from(ground, dark)` writes each canvas colour as a
  step from the canvas's own pane (`#1f232a`), so on that pane it is exactly the canvas's hex and on any
  other pane it is the same steps. `Theme::retoned` works it out again from the new `surface.s1`. It holds
  colours only; the shadows are its methods, so both themes still share every recipe.
- **The design's marks are `Icon::CHAT`, beside `Icon::ALL`.** `ALL` is still exactly `SbIcon`'s set and
  `tests/reference.rs` still checks it; the chat marks are transcribed from Unluminous's `theme::icon` SVGs
  and the canvas, and keep their own stroke through `Icon::stroke`.

A slice of a `Ring` too short for two round caps is drawn with square ends at exactly its share, and the
gap between slices is the same everywhere; `dial::arc_segments` and its tests are that arithmetic.

## The documentation is part of the change (task-2259)

`docs/` is the reference for anybody using rux, and the repository is public. A change that alters what a
component looks like or how it is called updates, in the same commit:

- the component's page under `docs/components/`, and the table in `docs/components/README.md`;
- its pictures, with `pwsh tools/capture-docs.ps1 -Stories <story>`, which takes both themes and the open
  states. Never edit a picture by hand;
- the storybook story, so the picture shows the new state. A new component gets a story, a page and a row
  in the table.

`docs/writing-style.md` is how the pages are written. `crates/rux/examples/first_window.rs` is the program
`docs/getting-started.md` walks through, and it is built by `cargo build -p rux --examples`, so a change to
the API that breaks that page breaks the build.

## Glass is a material, and the one place a theme changes a recipe (task-2281)

`Theme::glazed` makes a theme whose raised translucent surfaces bend the picture behind the window.
`docs/glass.md` is the page and `render/glass.rs` is the arithmetic. Three rules keep it from becoming a
second style:

- **No component asks whether the theme is glass.** `Chrome::surface` records `Decor::Glass` before the
  fill when `theme.glass` is set, the surface has an outer shadow and its fill is translucent. The rule that
  `components/` never branches on the theme still holds.
- **The recipes that change are `Chat`'s shadow methods and `Theme::elevation`**, both in `theme/`, and only
  when `Chat::glass` is set. `the_dark_theme_changes_no_recipe_at_all` is about the two shipped themes and
  still passes; a glazed theme is a third thing on purpose.
- **Glass bends a picture the application hands over** with `render::set_backdrop`, never the frame. A layer
  with glass in it is cut at each piece of glass into runs, each rasterised as its own texture, so the order
  of decoration is kept. A layer with no glass, or with no picture set, is one texture exactly as before.
  `tests/glass.rs` holds those rules and `tests/draw.rs` draws every component in a glazed theme.
