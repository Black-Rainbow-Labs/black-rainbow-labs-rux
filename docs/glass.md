# Glass

![Unluminous's chat pane in its glass theme, over a picture of spheres on pinstripes](images/glass/unluminous-chat.png)

A glazed theme draws every raised surface as glass that **bends** the picture behind the window, the
way a lens does, instead of blurring it or hiding it. The edges pull the picture in, the middle magnifies
it, and red, green and blue are bent by slightly different amounts, so an edge carries a faint colour
split. Over the bent picture each surface keeps a light tint, a lit top edge, a hairline rim and a long
soft shadow. Carved wells are a dark tint with no bend.

It came from a Claude Design canvas ("Unluminous", page "Agent-Chat 2"), which builds it in a browser
with an SVG backdrop filter per element. rux draws the same arithmetic with `egui` meshes, so it costs a
texture fetch per pixel on the graphics card and works anywhere egui draws, offscreen tests included.

## Making a theme glass

```rust
let dark = Theme::named("dark-neumorphic").unwrap();
let glass: &'static Theme = Box::leak(Box::new(dark.glazed("my-glass", my_pane_colour)));
state.set_theme(glass);
```

`Theme::glazed` keeps the theme's ink and accents and replaces the surfaces, the shadows and
`theme.chat` with the glass design's: the card is a white sheen over a smoke, the well is
`rgba(6, 8, 14, 0.42)`, and the raised elevations are the design's long fall and lit edges. It sets
`theme.glass` to `Glass::DESIGN`, the numbers the bend is made of. The two shipped themes have
`glass: None`, so nothing changes for a program that does not ask for it.

This is the one place a theme changes a recipe rather than a value. Glass is a material, and the design
draws depth with a fall and lit edges where neumorphism draws it with a light shadow and a dark one.

## Giving glass something to bend

Glass bends a picture the application hands over. egui draws every layer into one render pass and cannot
read back what it has already drawn, so glass cannot sample whatever happens to be under it.

```rust
rux::render::set_backdrop(ctx, Plane::Page, Some(Backdrop {
    bands: picture_strips,          // the picture's textures, top to bottom
    rect: window_rect,              // where the picture is drawn, in the window's points
    uv: shown_part,                 // which part of the picture is drawn there
    opacity: 1.0,                   // how much of what is behind the picture covers
}));
```

Set it once a frame, before anything draws. `Plane::Page` is what layers in egui's background and middle
orders bend. `Plane::Above` is what popups and modals bend, and falls back to the page's picture when it
is not set. Unluminous sets `Above` to the blurred photograph of the window a modal takes, which gives a
frosted modal whose glass bends the window's own contents. `Backdrop::single` is a picture held as one
texture. A picture held as several strips, which a large picture should be so a graphics driver never
keeps one large upload's staging memory, is a list of `Band`s.

With no backdrop set, glass is its tint and its edges and nothing is bent.

## Which surfaces are glass

`Chrome::surface` decides, so no component asks. A surface is glass when the theme is glazed, the
surface has a shadow outside it (it is raised) and its fill is translucent. An opaque fill hides the
picture anyway, and a carved well has no outer shadow. A layer with glass in it is cut at each piece of
glass: the decoration before it is one texture, the bent picture is drawn, and the decoration after it is
another texture. That is what puts a carved track over the card it is carved into and under the pill
raised out of it.

`render::glass::shapes` is public, for an application that paints a surface with egui directly and wants
it to be glass too. Unluminous uses it for egui's own popup frames and for its canvas nodes.

## No line along any edge

Glass draws no light line along its outline. Two things used to draw one, and both were measured with a
script that reads a screenshot's brightness across each round control's rim (Unluminous's `tools/glass-rims.py`): the title bar's chips on Unluminous 0.91.0 had rims 27 levels brighter than
both sides all round and 44 along the top.

- **The cover fades on the passes' own mesh.** The dark cover under the three colour passes was an `egui`
  rounded rectangle, whose one pixel feather is centred on the edge, while the passes stay whole up to the
  edge and fade only outside it. Along the outline the picture was added over a cover that was already
  partly gone. `glass::shapes` now builds the cover from `geometry`'s own points and triangles, so the cover
  and the passes leave the same share of what is behind at every point, and a test checks it.
- **The glass elevations have no hairline rim and no lit edge line.** The light from the top is a glow six
  points down and eighteen across (four and ten on a chip), so it lights the upper part of a surface without
  drawing a line. `glass_out`, `glass_out_sm`, `carved`, `carved_sm`, `action_raised` and `floating` in
  `theme/tokens.rs`.

After both, the same chips measure +2 all round and +4 along the top, the +4 being the soft light.

## What it costs

A piece of glass is about four hundred vertices, drawn three times, one for each colour channel. The
shapes are kept per layer and built again only when the surface, the picture or the layer's transform
moves, so a still frame builds nothing. Measured in Unluminous while scrolling its chat pane, egui's own
time per frame went up by about 0.4 ms against the same pane in the dark theme.

The decoration of a layer with glass is rasterised on the processor, one canvas for each run between two
pieces of glass, and on the frame a large surface first appears that was most of the frame: Unluminous's
Settings window rasterised 6.6 million pixels in 104 ms. Under glass the outline is the glass mesh, drawn by
the graphics card, and a canvas holds only a tint and soft shadows, so two limits apply to a canvas in a
glass layer and to no other:

- **At most one pixel a point** (`GLASS_MAX_SCALE`), where other canvases go up to `MAX_SCALE`.
- **At most 250,000 pixels** (`GLASS_CANVAS_PIXELS`); a larger run is rasterised at a lower scale and drawn
  back at full size with linear filtering.
- **A run of nothing but outer shadows at two pixels to the smallest deviation**, never below a quarter of a
  pixel a point (`SHADOW_LEAST_SCALE`): a Gaussian has no finer detail, and the surface's glass covers where
  the shadow meets its edge.
- **A surface's tint and inner glows at half a pixel a point** (`TINT_SCALE`): the run after a piece of glass
  holds only a faint fill whose outline the glass mesh draws, and glows as soft as a shadow.

The same limits apply to every layer under a glazed theme, including one with no glass in it. Measured on
Unluminous's Settings window, the first opening went from 172 ms to 23 ms.

Zoomed crops before and after cannot be told apart. `render::cost::take` says what was rasterised and built
on this thread since it was last asked, for an application's frame trace.

**A canvas not drawn is kept for twenty seconds** (`KEEP_UNUSED`): it lets go of its processor copy at once
and keeps its texture, so a modal closed and opened again, or a pane hidden and shown again, is drawn from
what it drew last time instead of rasterising again.

## What it does not do

- Glass inside glass bends the picture, not the glass under it. A browser bends the parent's already bent
  picture; doing that needs the frame read back.
- The desktop behind a transparent window is not a picture rux can read, so glass over it is tint only.
- There is no light glass. The design is dark.
