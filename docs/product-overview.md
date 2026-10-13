# Product overview

rux is the Black Rainbow Labs component library for Rust. It draws neumorphic components, in which
every control is made of the page's own colour and raised or pressed by soft shadows, on top of `egui`.
Unluminous, the Black Rainbow Labs code editor, draws its chat pane, its task board, its editor chrome
and its Settings window with it.

![The rux storybook in the dark theme, showing the Agent-Chat parts](images/storybook--dark-neumorphic.png)

## Who it is for

A Rust programmer who is building a desktop or phone application with `egui` and wants it to look like a
designed product. rux gives `egui` the four things its renderer cannot draw, which are most of what the
neumorphic style is made of: inset shadows, diagonal gradients, glows round a circle and clips with
rounded corners.

## What is in it

| | |
|---|---|
| **Themes** | `light-neumorphic` and `dark-neumorphic`. Each is a fixed list of surfaces, shadow tones, ink, accents, semantic colours, radii, twelve elevations and twelve gradients, copied from ai-service's stylesheets |
| **Components** | the components copied from the AI Studio design, the instrument controls, and the Agent-Chat and Agent-Tasks parts Unluminous uses. 106 public types in all |
| **Icons** | 63 marks, with their path data copied exactly, drawn from a texture atlas |
| **Type** | Inter 400, 500, 600 and 700 and JetBrains Mono 400, 500 and 600, included in the crate, with `letter-spacing` and `line-height` carried through |
| **Layout** | a flex and grid solver with gap, padding, alignment, `flex-shrink: 0` and `repeat(auto-fill, minmax())` |
| **Phones** | the same components on iOS and Android, with 44 point touch targets, safe areas and bottom sheets |
| **Storybook** | a desktop program that shows every component in every state in both themes |

## How it compares

| | rux | `egui` on its own | A web view |
|---|---|---|---|
| Inset shadows, diagonal gradients, glows, rounded clips | yes, drawn by `vello_cpu` | no | yes |
| Input, text, scrolling, accessibility | `egui`'s | `egui`'s | the browser's |
| Runs on iOS and Android with the same code | yes | yes | needs a web view on each |
| Look checked against the source stylesheet | yes, by `tests/reference.rs` | no stylesheet | the stylesheet itself |

Other Rust interface libraries, such as Iced, Xilem and Masonry, each own their widget set and their look.
rux exists so Black Rainbow Labs owns the look, while `egui` does the work a component library should not
redo.

## How it stays faithful to the design

The original components, the themes and the first thirty seven icons were copied from the AI Studio
stylesheets and React source in `reference/`. `tests/reference.rs` reads those files when it runs, parses
them, and compares every colour, radius, shadow, padding, gap and type size with the declaration it came
from. A changed token fails a test that names the declaration.

The Unluminous parts were drawn to the Claude Design canvas "Unluminous". Their colours are worked out
from the pane they sit on, so they follow either theme.

## How fast it is

A frame where nothing moved costs one comparison of the decoration list and draws nothing. A hover
changes a vertex colour. A page with three spinners turning at 60 frames a second used 253 ms of
processor time a second, with its decoration drawn once. [How rux draws](architecture.md) explains each
rule and where each number was measured.

## Licence

rux is dual licensed under MIT and Apache 2.0. The fonts are under the SIL Open Font License.

## Where to go next

- [Getting started](getting-started.md): a first window.
- [Components](components/README.md): every component, with pictures.
