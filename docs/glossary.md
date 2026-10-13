# Glossary

This page explains the words the rux documentation uses. Each entry is one or two plain sentences. The
terms are grouped by topic, and each group is in alphabetical order.

| Group | What it covers |
|---|---|
| [The design](#the-design) | the visual language the components are drawn in |
| [Drawing](#drawing) | how a component becomes pixels |
| [The library](#the-library) | the types a program uses |
| [The repository](#the-repository) | the programs and files around the library |

## The design

| Term | What it means |
|---|---|
| **Accent** | One of the eight named colours a theme has for emphasis, such as `accent.blue` and `accent.mint`. A component never chooses a colour of its own. It names an accent. |
| **Elevation** | A list of up to six shadows that makes a surface look raised above the page or pressed into it. Each elevation is a CSS `box-shadow` declaration copied from `reference/neumorphic-tokens.css`, such as `--e-raised`. |
| **Ground** | The colour of the page behind everything. `Rux::ground` paints it. |
| **Ink** | The six text and mark colours a theme has, from `ink.i900` (the strongest) to `ink.i200` (the faintest). |
| **Inset shadow** | A shadow drawn inside a shape instead of around it. It makes a shape look pressed into the page. Every well, field and pressed button uses one. |
| **Instrument language** | The style Unluminous's chat answers use: plates, recessed screens, key caps that sink and small lit LEDs, after studio hardware. `components/instrument.rs` holds its five parts. |
| **LED** | In the instrument language, a small lit dot with a soft glow. It is where colour appears on an instrument control. |
| **Neumorphic** | A style where every control is made of the page's own colour, raised by a light shadow above and to the left and a dark shadow below and to the right, or pressed in by the same two shadows inset. |
| **Plate** | In the instrument language, a raised panel with a lit line along its top edge. |
| **Pressed** | Drawn with an inset shadow, so it looks pushed into the page. |
| **Raised** | Drawn with two outer shadows, so it looks lifted off the page. |
| **Screen** | In the instrument language, a dark recessed area with a faint dot grid, where data is shown. Not to be confused with `rux::Screen`, which describes the device. |
| **Token** | One named value in a theme: a colour, a radius, a shadow or a gradient. The list of token names is fixed. A theme gives each name a value. |
| **Well** | A surface pressed into the one it sits on. A field, a prompt box and a lane on the task board are wells. |

## Drawing

| Term | What it means |
|---|---|
| **Atlas** | One texture that holds many small pictures. rux keeps every icon it has drawn in an atlas, so drawing an icon again costs nothing. |
| **Canvas** | The `vello_cpu` picture behind one layer. `Canvas::texture_for` draws it again only when the `Decor` list changed. |
| **Chrome** | The recorder a component draws its decoration through: `rux.chrome.surface(...)`, `rux.chrome.disc(...)`. It writes `Decor` values into a list. |
| **Decor** | One piece of decoration: a rounded rectangle, a blurred shadow, a disc, a ring, an arc, a line or a clip. A plain value, so a test can compare two lists. |
| **Decoration** | Everything `epaint` cannot draw: inset shadows, diagonal gradients, glows and rounded clips. rux draws it with `vello_cpu`. |
| **`egui`** | The Rust immediate mode user interface library rux is built on. It handles input, hit testing, scrolling, text, the caret, the clipboard and accessibility. |
| **`epaint`** | The part of `egui` that turns shapes and text into triangles. It draws flat fills, lines and text well, and it cannot draw an inset shadow or a diagonal gradient. |
| **Gaussian blur** | The soft fall off a shadow has. CSS describes it with a blur radius, and rux converts that radius to the blur's standard deviation by halving it. |
| **Immediate mode** | A style of interface where the program describes every widget again on every frame instead of keeping a tree of widget objects. `egui` works this way, so rux does too. |
| **Layer** | One `egui` layer with one decoration texture behind its widgets. The page is a layer, and every dropdown menu and dialog opens a layer of its own. |
| **Rasterise** | Turn a list of shapes into pixels. The design rule of rux is that a frame where nothing moved rasterises nothing. |
| **Slot** | A place in `egui`'s list of shapes, reserved before any widget draws, that the layer's decoration texture is put into at the end of the frame. |
| **`vello_cpu`** | A Rust 2D renderer that runs on the processor. rux uses version 0.1, which is the copy `epaint` already depends on. |

## The library

| Term | What it means |
|---|---|
| **`measure`** | The method every component has that answers how big it wants to be, before anything places it. |
| **Outcome** | The struct a component's `show` returns when a plain `egui::Response` cannot say everything that happened, such as `SelectOutcome`, which also says which option was chosen. |
| **`Rux`** | The value a component draws with. `rux::layer` makes one for the length of a frame. It carries the `egui::Ui`, the `Chrome`, the theme and the screen. |
| **`RuxState`** | The value an application keeps between frames: the theme, the canvases, the icon atlas, the screen, the zoom and whether controls animate. |
| **`Screen`** | What a component may know about the device: its size, the safe area insets, and whether the pointer is a mouse or a finger. |
| **`show`** | The method every component has that draws it into the rectangle it is given and answers what happened. |
| **Size class** | `Compact`, `Medium` or `Expanded`, from the width of the screen. Compact is under 600 points and medium is under 840. |
| **Touch target** | The area a finger can press. On a phone every control's `measure` answers at least 44 points. |

## The repository

| Term | What it means |
|---|---|
| **Design canvas** | The Claude Design canvas "Unluminous", whose pages Agent-Chat, Agent-Tasks, Editing code and Settings are the source for the parts Unluminous uses. |
| **Mobile preview** | `rux-mobile-preview`, a desktop program that draws the mobile application inside a phone frame at a real phone's size and safe area. |
| **Reference** | The files in `reference/`: the stylesheets and React source of the AI Studio design that the original components were copied from. `tests/reference.rs` reads them. |
| **Story** | One page of the storybook: a component and its named states, drawn live. |
| **Storybook** | `rux-storybook`, the desktop program that shows every story in both themes. |
