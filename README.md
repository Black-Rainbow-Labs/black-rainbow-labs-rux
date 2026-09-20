# black-rainbow-labs-rux

A private Black Rainbow Labs component library for Rust, written the way the Unluminous editor is
written: `egui` for input, hit testing, scrolling and text, and `vello_cpu` for the decoration
`epaint` cannot draw.

It ships two neumorphic themes transcribed from ai-service's own stylesheets, nineteen components
transcribed from the AI Studio design, an example page that rebuilds that design out of those
components, and a desktop storybook that shows every component in every state.

```
crates/rux/          the library — the only crate a consumer depends on
examples/pages/      the Image Creator page, rebuilt from the components
storybook/           the desktop storybook
reference/           the stylesheets and source everything here is measured against
tasks/               the design
```

## Running it

```sh
cargo run --release -p rux-storybook
cargo run --release -p rux-storybook -- --theme light-neumorphic
cargo run --release -p rux-storybook -- --list
```

The storybook has a theme picker at the top left, a filter over every story and state, a page per
component and a page per example, all live and interactive. `--help` lists the rest of the switches;
`--story`, `--filter`, `--open` and `--background` exist so a capture pass can reach any state
without touching the keyboard or the pointer.

## Using it

```rust
use rux::prelude::*;

// Once, before the first frame.
rux::text::install(ctx);
let state = RuxState::new(Theme::named("dark-neumorphic").unwrap());

// Every frame. `layer` reserves the slot the decoration is painted into, hands you a `Rux`, and
// fills the slot in once you are done.
rux::layer(ui, &state, egui::Id::new("page"), area, |rux| {
    rux.ground(area);
    let [left, right] = Flex::row(area).pad(Pad::all(20.0)).gap(12.0).fill(1.0).hug(140.0).solve();
    if Button::new("Create").primary().icon(Icon::Spark).show(rux, right).clicked() {
        // …
    }
});
state.end_frame();
```

Every component is the same shape:

```rust
fn measure(&self, rux: &Rux) -> Vec2;                 // what it wants, before anybody places it
fn show(self, rux: &mut Rux, rect: Rect) -> Response; // draws into the rect it is given
```

and keeps three rules. A component **decides nothing about the application** — it draws, it answers
what happened, and state that has to survive a frame is the caller's, which is what lets the same
component appear twice on one screen. It **reads the theme and never a colour**; there is no
`Color32::from_rgb` anywhere in `components/`. And it **sets the cursor the stylesheet sets**.

## What is in it

| | |
|---|---|
| **Themes** | `light-neumorphic` and `dark-neumorphic`, each a closed set of surfaces, shadow tones, ink, accents, semantic colours, radii, twelve elevations and twelve gradients |
| **Components** | top nav, button, icon button, select, project menu, text input, text area, stepper, spinner / progress / skeleton, chip, segmented control, dice toggle, switch, panel, well, collapsible group, divider, list item, avatar, tile, scrollbar, modal |
| **Icons** | all thirty-seven `SbIcon` marks, path data verbatim, rasterised into a tinted texture atlas |
| **Type** | Inter 400/500/600/700 and JetBrains Mono 400/500/600, vendored, with `letter-spacing` and `line-height` carried through |
| **Layout** | a flex row/column/grid solver with gap, padding, align, justify, `flex-shrink: 0` and `repeat(auto-fill, minmax())` |

## Pixel perfect, and how that is checked

`reference/` holds the stylesheets this was built from: the AI Studio Image Creator page's own CSS,
its React source, the shared `SbIcon` / `TopNav` / `ProjectDropdown` source, and ai-service's
`neumorphic-tokens.css` and `incognito-theme.css`.

`crates/rux/tests/reference.rs` **reads those files at test time and parses them**. Every colour in
both themes, every radius, every shadow offset and blur, the padding and gap of every component the
design names, the whole type scale and every icon name is compared against the declaration it was
transcribed from. That is stronger than comparing two pictures: a screenshot can only say two images
differ, where this says which declaration is wrong.

```sh
cargo test                                   # 59 tests, none of which needs a graphics card
pwsh tools/capture-stories.ps1               # every story, both themes, into _agent_output/
```

## Measured

Release build, 1440×920, on this machine:

| | |
|---|---:|
| Working set, a component story | 188 MB |
| Working set, the example page | 194 MB |
| Idle, nothing animating | 0–8 ms of processor time a second |
| The example page, three spinners at 60 fps | 253 ms/s, with the canvas rasterised **once** |
| Decoration texture, full window | 5,175 KB |
| Icon atlas, thirty marks | 36 KB |
| Binary | 16.7 MB |

Most of the working set is the graphics driver rather than anything here — Unluminous measured a DX12
device and egui's shader at 135 MB of a 223 MB window, and this is the same stack.

The number that matters is the third row: a page animating three spinners at sixty frames a second
rasterises its decoration **once**, because nothing that moves is allowed into the `Decor` list. The
spinner's arc and every icon are textures, and flat hover washes are `egui` shapes.

## The conventions

`CLAUDE.md` is what a later change has to follow. `tasks/task-2008-rux-tdd.md` is the design, and
records what was weighed and refused.

## Licences

The library is private to Black Rainbow Labs. The vendored faces are not: Inter and JetBrains Mono
are both SIL Open Font License, and their licences are in `crates/rux/assets/fonts/`.
