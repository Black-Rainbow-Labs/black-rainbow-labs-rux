//! What `rux`'s drawing cost on this thread since it was last asked, for an application's frame trace.
//!
//! A layer's decoration is rasterised on the processor and uploaded as a texture, and a piece of glass is
//! a mesh built from a few hundred points. Both are kept while nothing they were made from moves, so on
//! most frames they cost nothing, and on the frame something moves they can be most of the frame. An
//! application that measures its frames could not see inside `rux::layer` to say which; this is what it
//! reads. Counting is two clock reads and two additions per canvas or piece of glass actually made.

use std::cell::Cell;
use std::time::Instant;

/// The drawing done since the last [`take`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cost {
    /// Milliseconds spent rasterising canvases.
    pub raster_ms: f64,
    /// How many canvases were rasterised.
    pub canvases: u32,
    /// How many pixels those canvases had between them, which is what is uploaded.
    pub pixels: u64,
    /// Milliseconds spent building pieces of glass.
    pub glass_ms: f64,
    /// How many pieces of glass were built rather than reused.
    pub glasses: u32,
}

thread_local! {
    static SO_FAR: Cell<Cost> = const { Cell::new(Cost {
        raster_ms: 0.0, canvases: 0, pixels: 0, glass_ms: 0.0, glasses: 0,
    }) };
}

/// What was drawn since the last call, and start counting again.
pub fn take() -> Cost {
    SO_FAR.with(|cost| cost.replace(Cost::default()))
}

/// Note a canvas of `pixels` rasterised from `started` until now.
pub(crate) fn rasterised(started: Instant, pixels: u64) {
    let took = started.elapsed().as_secs_f64() * 1000.0;
    SO_FAR.with(|cost| {
        let mut now = cost.get();
        now.raster_ms += took;
        now.canvases += 1;
        now.pixels += pixels;
        cost.set(now);
    });
}

/// Note a piece of glass built from `started` until now.
pub(crate) fn glazed(started: Instant) {
    let took = started.elapsed().as_secs_f64() * 1000.0;
    SO_FAR.with(|cost| {
        let mut now = cost.get();
        now.glass_ms += took;
        now.glasses += 1;
        cost.set(now);
    });
}
