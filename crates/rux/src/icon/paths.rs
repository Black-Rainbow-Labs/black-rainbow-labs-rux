//! The icon set, transcribed from `reference/shared-components.jsx`.
//!
//! **The path data is verbatim.** An icon redrawn from a recollection of what it looked like is a
//! drawing of something else wearing the right name, so every `d` string below is the string in
//! `SbIcon`, character for character, and the `<circle>` and `<rect>` children keep their own typed
//! shapes rather than being converted by hand into path data.
//!
//! The parent `<svg>` sets `fill="none" stroke="currentColor" stroke-width="{stroke}"
//! stroke-linecap="round" stroke-linejoin="round"` on a `0 0 24 24` viewBox. A child that sets
//! `fill="currentColor"` **also inherits the stroke**, which is why [`Part::filled`] leaves stroking
//! on and only `play` and `pause` — which say `stroke="none"` — turn it off.
//!
//! Each arm of [`Icon::parts`] is an inline `const` block, so every drawing is one static allocation
//! made at compile time rather than a slice built on the stack and handed out by reference.

/// One child of an icon's `<svg>`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub fill: bool,
    pub stroke: bool,
}

/// What that child is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// `<path d="…"/>`, in the viewBox's own units.
    Path(&'static str),
    /// `<circle cx cy r/>`.
    Circle { cx: f32, cy: f32, r: f32 },
    /// `<rect x y width height rx/>`.
    Rect { x: f32, y: f32, w: f32, h: f32, rx: f32 },
}

impl Part {
    /// The ordinary case: stroked, not filled.
    pub const fn stroked(shape: Shape) -> Self {
        Self { shape, fill: false, stroke: true }
    }

    /// `fill="currentColor"` with the stroke still inherited, which is what `dots` and `grip` are.
    pub const fn filled(shape: Shape) -> Self {
        Self { shape, fill: true, stroke: true }
    }

    /// `fill="currentColor" stroke="none"` — `play` and `pause`.
    pub const fn solid(shape: Shape) -> Self {
        Self { shape, fill: true, stroke: false }
    }
}

const fn p(d: &'static str) -> Part {
    Part::stroked(Shape::Path(d))
}

const fn dot(cx: f32, cy: f32, r: f32) -> Part {
    Part::filled(Shape::Circle { cx, cy, r })
}

const fn circle(cx: f32, cy: f32, r: f32) -> Part {
    Part::stroked(Shape::Circle { cx, cy, r })
}

const fn rect(x: f32, y: f32, w: f32, h: f32, rx: f32) -> Part {
    Part::stroked(Shape::Rect { x, y, w, h, rx })
}

const fn solid_rect(x: f32, y: f32, w: f32, h: f32, rx: f32) -> Part {
    Part::solid(Shape::Rect { x, y, w, h, rx })
}

/// Every icon in the set, in `SbIcon`'s own order.
///
/// The name is the JSX key, so a rule in `reference/image-creator-page.jsx.html` that asks for
/// `<SbIcon name="chevDown"/>` maps onto [`Icon::ChevDown`] without a translation table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Icon {
    Search,
    Plus,
    X,
    Check,
    ChevDown,
    ChevLeft,
    ChevRight,
    ArrowRight,
    Home,
    Docs,
    Spark,
    Dots,
    Film,
    Storyboard,
    Layers,
    Folder,
    Settings,
    Bell,
    User,
    Trash,
    Download,
    Heart,
    Star,
    Chat,
    Cube,
    Image,
    Play,
    Pause,
    Music,
    ImagePlus,
    Edit,
    Video,
    Upload,
    Grip,
    Expand,
    Copy,
    Wand,
}

impl Icon {
    /// Every icon, which is what the storybook's icon sheet walks.
    pub const ALL: &'static [Icon] = &[
        Icon::Search,
        Icon::Plus,
        Icon::X,
        Icon::Check,
        Icon::ChevDown,
        Icon::ChevLeft,
        Icon::ChevRight,
        Icon::ArrowRight,
        Icon::Home,
        Icon::Docs,
        Icon::Spark,
        Icon::Dots,
        Icon::Film,
        Icon::Storyboard,
        Icon::Layers,
        Icon::Folder,
        Icon::Settings,
        Icon::Bell,
        Icon::User,
        Icon::Trash,
        Icon::Download,
        Icon::Heart,
        Icon::Star,
        Icon::Chat,
        Icon::Cube,
        Icon::Image,
        Icon::Play,
        Icon::Pause,
        Icon::Music,
        Icon::ImagePlus,
        Icon::Edit,
        Icon::Video,
        Icon::Upload,
        Icon::Grip,
        Icon::Expand,
        Icon::Copy,
        Icon::Wand,
    ];

    /// The name `SbIcon` knows this mark by.
    pub fn name(self) -> &'static str {
        match self {
            Icon::Search => "search",
            Icon::Plus => "plus",
            Icon::X => "x",
            Icon::Check => "check",
            Icon::ChevDown => "chevDown",
            Icon::ChevLeft => "chevLeft",
            Icon::ChevRight => "chevRight",
            Icon::ArrowRight => "arrowRight",
            Icon::Home => "home",
            Icon::Docs => "docs",
            Icon::Spark => "spark",
            Icon::Dots => "dots",
            Icon::Film => "film",
            Icon::Storyboard => "storyboard",
            Icon::Layers => "layers",
            Icon::Folder => "folder",
            Icon::Settings => "settings",
            Icon::Bell => "bell",
            Icon::User => "user",
            Icon::Trash => "trash",
            Icon::Download => "download",
            Icon::Heart => "heart",
            Icon::Star => "star",
            Icon::Chat => "chat",
            Icon::Cube => "cube",
            Icon::Image => "image",
            Icon::Play => "play",
            Icon::Pause => "pause",
            Icon::Music => "music",
            Icon::ImagePlus => "imagePlus",
            Icon::Edit => "edit",
            Icon::Video => "video",
            Icon::Upload => "upload",
            Icon::Grip => "grip",
            Icon::Expand => "expand",
            Icon::Copy => "copy",
            Icon::Wand => "wand",
        }
    }

    /// The icon with this `SbIcon` name, or `None`.
    pub fn named(name: &str) -> Option<Icon> {
        Icon::ALL.iter().copied().find(|icon| icon.name() == name)
    }

    /// The children of this icon's `<svg>`, in document order.
    pub fn parts(self) -> &'static [Part] {
        match self {
            Icon::Search => const { &[circle(11.0, 11.0, 7.0), p("m20 20-3.5-3.5")] },
            Icon::Plus => const { &[p("M12 5v14M5 12h14")] },
            Icon::X => const { &[p("M6 6l12 12M18 6L6 18")] },
            Icon::Check => const { &[p("M5 12l5 5L20 7")] },
            Icon::ChevDown => const { &[p("m6 9 6 6 6-6")] },
            Icon::ChevLeft => const { &[p("m15 6-6 6 6 6")] },
            Icon::ChevRight => const { &[p("m9 6 6 6-6 6")] },
            Icon::ArrowRight => const { &[p("M5 12h14M13 6l6 6-6 6")] },
            Icon::Home => const { &[p("M3 12 12 4l9 8"), p("M5 10v10h14V10")] },
            Icon::Docs => const {
                &[p("M8 3h8l4 4v14H4V3h4"), p("M9 3v4h6V3"), p("M8 12h8M8 16h5")]
            },
            Icon::Spark => const {
                &[p("M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5 18 18M6 18l2.5-2.5M15.5 8.5 18 6")]
            },
            Icon::Dots => const {
                &[dot(5.0, 12.0, 1.2), dot(12.0, 12.0, 1.2), dot(19.0, 12.0, 1.2)]
            },
            Icon::Film => const {
                &[rect(3.0, 4.0, 18.0, 16.0, 2.0), p("M3 9h18M3 15h18M8 4v16M16 4v16")]
            },
            Icon::Storyboard => const {
                &[
                    rect(3.0, 4.0, 8.0, 7.0, 1.5),
                    rect(13.0, 4.0, 8.0, 7.0, 1.5),
                    rect(3.0, 13.0, 8.0, 7.0, 1.5),
                    rect(13.0, 13.0, 8.0, 7.0, 1.5),
                ]
            },
            Icon::Layers => const {
                &[p("m12 3 9 5-9 5-9-5 9-5Z"), p("m3 13 9 5 9-5M3 18l9 5 9-5")]
            },
            Icon::Folder => const {
                &[p("M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z")]
            },
            Icon::Settings => const {
                &[
                    circle(12.0, 12.0, 3.0),
                    p("M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 0 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z"),
                ]
            },
            Icon::Bell => const {
                &[p("M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"), p("M10 21a2 2 0 0 0 4 0")]
            },
            Icon::User => const { &[circle(12.0, 8.0, 4.0), p("M4 21a8 8 0 0 1 16 0")] },
            Icon::Trash => const {
                &[p("M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6")]
            },
            Icon::Download => const { &[p("M12 3v12M7 10l5 5 5-5M4 21h16")] },
            Icon::Heart => const {
                &[p("M12 21s-8-5-8-11a5 5 0 0 1 9-3 5 5 0 0 1 9 3c0 6-8 11-8 11Z")]
            },
            Icon::Star => const {
                &[p("m12 3 2.8 6 6.2.8-4.5 4.4 1 6.1-5.5-3-5.5 3 1-6.1L3 9.8 9.2 9 12 3Z")]
            },
            Icon::Chat => const { &[p("M21 12a8 8 0 0 1-12 7l-5 1 1-5A8 8 0 1 1 21 12Z")] },
            Icon::Cube => const {
                &[p("m12 3 9 5v8l-9 5-9-5V8l9-5Z"), p("M3 8l9 5 9-5M12 13v10")]
            },
            Icon::Image => const {
                &[
                    rect(3.0, 3.0, 18.0, 18.0, 3.0),
                    circle(9.0, 9.0, 2.0),
                    p("m3 17 5-5 5 5 3-3 5 5"),
                ]
            },
            Icon::Play => const { &[Part::solid(Shape::Path("M7 4v16l13-8L7 4Z"))] },
            Icon::Pause => const {
                &[solid_rect(6.0, 4.0, 4.0, 16.0, 1.0), solid_rect(14.0, 4.0, 4.0, 16.0, 1.0)]
            },
            Icon::Music => const {
                &[p("M9 18V6l12-2v12"), circle(6.0, 18.0, 3.0), circle(18.0, 16.0, 3.0)]
            },
            Icon::ImagePlus => const {
                &[
                    rect(3.0, 3.0, 14.0, 14.0, 2.0),
                    p("M17 17h4M19 15v4"),
                    circle(8.0, 8.0, 1.5),
                    p("m3 13 4-4 5 5"),
                ]
            },
            Icon::Edit => const { &[p("M16 4l4 4-11 11H5v-4L16 4Z")] },
            Icon::Video => const { &[rect(3.0, 6.0, 13.0, 12.0, 2.0), p("m22 8-6 4 6 4V8Z")] },
            Icon::Upload => const { &[p("M12 21V9M7 14l5-5 5 5M4 3h16")] },
            Icon::Grip => const {
                &[
                    dot(9.0, 6.0, 1.0),
                    dot(15.0, 6.0, 1.0),
                    dot(9.0, 12.0, 1.0),
                    dot(15.0, 12.0, 1.0),
                    dot(9.0, 18.0, 1.0),
                    dot(15.0, 18.0, 1.0),
                ]
            },
            Icon::Expand => const { &[p("M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5")] },
            Icon::Copy => const {
                &[rect(8.0, 8.0, 13.0, 13.0, 2.0), p("M4 16V5a2 2 0 0 1 2-2h11")]
            },
            Icon::Wand => const {
                &[p("m15 4 2 2M19 8l2 2M9 15l-6 6M15 4l-6 6 4 4 6-6-4-4Z")]
            },
        }
    }
}

/// The default `stroke-width` of an `SbIcon`, in viewBox units.
pub const DEFAULT_STROKE: f32 = 1.8;

/// The icon viewBox: `0 0 24 24`.
pub const VIEW_BOX: f32 = 24.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_a_name_and_a_drawing() {
        for icon in Icon::ALL {
            assert!(!icon.name().is_empty());
            assert!(!icon.parts().is_empty(), "{} has no parts", icon.name());
            assert_eq!(Icon::named(icon.name()), Some(*icon));
        }
    }

    #[test]
    fn the_set_is_the_reference_s_own() {
        // `SbIcon` has thirty-seven marks. One added here belongs in the storybook's icon sheet too,
        // which walks `ALL`.
        assert_eq!(Icon::ALL.len(), 37);
        assert!(Icon::named("cheveron").is_none());
    }

    #[test]
    fn only_play_and_pause_turn_the_stroke_off() {
        // The parent `<svg>` sets the stroke, so a child that only overrides `fill` is both filled and
        // stroked. Getting this wrong makes `dots` and `grip` a third of the size they should be.
        for icon in Icon::ALL {
            for part in icon.parts() {
                let expected = !matches!(icon, Icon::Play | Icon::Pause);
                assert_eq!(part.stroke, expected, "{}", icon.name());
            }
        }
    }
}
