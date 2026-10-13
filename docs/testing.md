# Testing

rux is tested in four layers. The first three need no graphics card and no window. The fourth is the
storybook, built and photographed.

```sh
cargo test -p rux                # 109 tests, none of which needs a graphics card
pwsh tools/capture-stories.ps1   # every story in both themes, into _agent_output/
pwsh tools/capture-docs.ps1      # the pictures in docs/images/
```

The count is from `cargo test -p rux` on 2026-10-13: 78 unit tests in `src/`, 29 integration tests in
`tests/`, and 2 doc tests.

## The four layers

| Layer | Where | What it checks |
|---|---|---|
| Layout | `src/layout.rs` | The flex and grid solver, asserted on arithmetic a reader can check |
| Theme and reference | `src/theme/`, `tests/reference.rs` | Every token in both themes against the CSS file it was copied from |
| Render | `src/render/` | A `Decor` list is a value, so what a surface records is a unit test |
| The window | `tests/draw.rs`, the storybook | Every component drawn, in both themes, on a desktop and on a phone |

## The reference is read at test time

`tests/reference.rs` opens the files in `reference/` and parses them. Every colour in both themes,
every radius, every shadow's offsets and blur, the padding and gap of every component the design names,
the whole type scale and every icon name is compared with the declaration it came from.

That is a stronger check than comparing two pictures. A picture comparison can only say that two images
differ. `reference.rs` says which declaration is wrong:

| Test | What it compares |
|---|---|
| `every_colour_in_the_light_theme_is_the_token_file_s` | `theme::light()` against `neumorphic-tokens.css` |
| `every_colour_in_the_dark_theme_is_the_incognito_file_s` | `theme::dark()` against `incognito-theme.css` |
| `the_radii_are_the_token_file_s` | `radius.*` against `--r-*` |
| `the_elevations_are_the_token_file_s_declarations` | every shadow in every elevation against its `--e-*` declaration |
| `a_blur_radius_is_half_the_gaussian_the_renderer_is_given` | the CSS blur radius to standard deviation conversion |
| `the_component_measurements_are_the_page_stylesheet_s` | component padding, gaps and sizes against `image-creator.css` |
| `the_type_is_the_page_stylesheet_s`, `the_named_type_scale_is_the_token_file_s` | `Style::*` against `.t-*` |
| `the_dark_theme_changes_no_recipe_at_all` | both themes have identically shaped elevations and gradient angles |

So changing a token means changing the reference, or the test fails. The stylesheet is the source of
truth, and the library is a copy of it.

## Every component drawn, in a debug build

`tests/draw.rs` draws every component, every instrument, every chat part and every board part, in both
themes, on a desktop screen and on a phone screen, with no window.

It exists because a release build hid a crash. `epaint::Mesh::colored_vertex` has a `debug_assert` that
the mesh has no texture, and both places in `rux::icon` that draw a turned mark called it. With
assertions off the picture was right. With assertions on the process stopped. Every program anybody ran
was a release build, so the Android application's first frame was the first time that line ran with
assertions on. A component that gains a drawing path goes in `tests/draw.rs`.

## On a phone

`tests/touch.rs` checks two things:

1. On a touch screen nothing a finger presses measures under 44 points. The test loops over the size
   enums, so a size added later is covered.
2. On a desktop every measurement is exactly what it was before rux knew about phones. That is a table
   of the old numbers.

It also checks that a touch pointer never hovers, that the nav strip fits a phone, that a `Select` menu
is a sheet on a phone and a list under its trigger on a desktop, and that a segment or a dialog with no
icon still draws its words.

## Two regressions from Unluminous

| File | What it checks |
|---|---|
| `tests/hint.rs` | A field's placeholder is set in the field's own font, whatever the `Ui`'s body size is. `egui` 0.36 lays a hint out at the `Ui`'s body size, which in Unluminous was 24 points inside a 13 point field. |
| `tests/transform.rs` | A layer drawn through a `TSTransform` opens its popups in the right place, and rasterises its decoration and icons at the size it is seen at. Unluminous zooms a dialog this way. |

## The same pixels on every machine

`Canvas::deterministic()` and `IconAtlas::deterministic()` pin `vello_cpu`'s SIMD level to
`Level::baseline()`, and `RuxState::deterministic(theme)` uses both. The widest SIMD level a processor
has is not promised to give the same bits as another processor's, so a picture accepted on one machine
could otherwise fail on another for a reason that is not a fault in rux.

## Photographing the storybook

The capture scripts send no keystroke and no click and never take the foreground, so they can run while
somebody is working on the same machine. On Windows, activating a window switches the virtual desktop
with it and takes the keyboard out of whatever the person was typing into.

| Switch | What it does |
|---|---|
| `--background` | opens the window without taking the foreground |
| `--story <name>` | opens straight onto a story |
| `--open` | opens the dropdowns and the dialog |
| `--filter <text>` | starts with the sidebar filtered |
| `--size <W>x<H>` | opens the window at this size in points |
| `--scroll <points>` | opens the story scrolled down this far |

`tools/capture-window.ps1` uses `PrintWindow` with `PW_RENDERFULLCONTENT | PW_CLIENTONLY`, so it
photographs a window that is behind another one or has never been activated.

`tools/capture-docs.ps1` takes each story in slices, because Windows will not open a window taller than
the screen. It opens the storybook once for each slice with a larger `--scroll`, cuts away the top bar,
the sidebar and the scrollbar, joins the slices, and trims the empty ground under the last state.

Pictures from `capture-stories.ps1` go under `_agent_output/`, which git ignores. Pictures from
`capture-docs.ps1` go in `docs/images/` and are committed.

## Where to go next

- [Repository and building](repository.md): the crates and the commands.
- [How rux draws](architecture.md): the rules these tests check.
