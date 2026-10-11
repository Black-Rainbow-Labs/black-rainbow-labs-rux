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
    /// `fill-opacity`, which only the paper airplane's body uses: a quarter-tinted wing under a full
    /// outline.
    pub fill_opacity: f32,
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
        Self { shape, fill: false, stroke: true, fill_opacity: 1.0 }
    }

    /// `fill="currentColor"` with the stroke still inherited, which is what `dots` and `grip` are.
    pub const fn filled(shape: Shape) -> Self {
        Self { shape, fill: true, stroke: true, fill_opacity: 1.0 }
    }

    /// `fill="currentColor" fill-opacity="…"` with the stroke inherited — the paper airplane's body.
    pub const fn veiled(shape: Shape, fill_opacity: f32) -> Self {
        Self { shape, fill: true, stroke: true, fill_opacity }
    }

    /// `fill="currentColor" stroke="none"` — `play` and `pause`.
    pub const fn solid(shape: Shape) -> Self {
        Self { shape, fill: true, stroke: false, fill_opacity: 1.0 }
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

const fn solid(shape: Shape) -> Part {
    Part::solid(shape)
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
    // The marks below are not in `SbIcon`. They are the Agent-Chat design's (`task-2235`): the first
    // seven are Unluminous's own window marks, transcribed from the SVG that
    // `cargo run -p unluminous-app --example export_icons_svg` writes for the material set, and `Send`
    // is the paper airplane the design canvas draws inline.
    Clock,
    Key,
    Tick,
    Cross,
    File,
    Rerun,
    Photo,
    Send,
    // The Editing code design's marks (`task-2241`): the rail's line icons, the text options mark, the three
    // view modes and the run button's triangle, transcribed from the canvas's `EditorWindow` artboard.
    Project,
    Document,
    Git,
    Realm,
    Bubble,
    Board,
    Database,
    Bug,
    RunTile,
    Terminal,
    Letters,
    Lines,
    Halves,
    Eye,
    Start,
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

    /// The marks the Agent-Chat design added, which are not `SbIcon`'s and so are not in [`Icon::ALL`].
    pub const CHAT: &'static [Icon] = &[
        Icon::Clock,
        Icon::Key,
        Icon::Tick,
        Icon::Cross,
        Icon::File,
        Icon::Rerun,
        Icon::Photo,
        Icon::Send,
    ];

    /// The marks the Editing code design added (`task-2241`), which are not `SbIcon`'s either.
    pub const WINDOW: &'static [Icon] = &[
        Icon::Project,
        Icon::Document,
        Icon::Git,
        Icon::Realm,
        Icon::Bubble,
        Icon::Board,
        Icon::Database,
        Icon::Bug,
        Icon::RunTile,
        Icon::Terminal,
        Icon::Letters,
        Icon::Lines,
        Icon::Halves,
        Icon::Eye,
        Icon::Start,
    ];

    /// Every mark the library can draw: the reference's, the Agent-Chat design's and the Editing code design's.
    pub fn every() -> impl Iterator<Item = Icon> {
        Icon::ALL.iter().chain(Icon::CHAT).chain(Icon::WINDOW).copied()
    }

    /// The `stroke-width` the mark's own drawing sets, in viewBox units.
    ///
    /// Every `SbIcon` is drawn at [`DEFAULT_STROKE`]. The Agent-Chat marks are smaller drawings inside
    /// the same viewBox, each with the stroke its source sets, so a well that draws one at eighteen
    /// points gets the weight the design shows.
    pub fn stroke(self) -> f32 {
        match self {
            Icon::Clock | Icon::Key | Icon::File => 1.3,
            Icon::Tick => 1.8,
            Icon::Cross => 1.5,
            Icon::Rerun => 1.4,
            Icon::Photo => 1.2,
            Icon::Send => 1.9,
            Icon::Letters | Icon::Document | Icon::Bug | Icon::Terminal | Icon::Git | Icon::Realm | Icon::Bubble
            | Icon::Board | Icon::Database | Icon::RunTile | Icon::Project => 1.7,
            _ => DEFAULT_STROKE,
        }
    }

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
            Icon::Clock => "clock",
            Icon::Key => "key",
            Icon::Tick => "tick",
            Icon::Cross => "cross",
            Icon::File => "file",
            Icon::Rerun => "rerun",
            Icon::Photo => "photo",
            Icon::Send => "send",
            Icon::Project => "project",
            Icon::Document => "document",
            Icon::Git => "git",
            Icon::Realm => "realm",
            Icon::Bubble => "bubble",
            Icon::Board => "board",
            Icon::Database => "database",
            Icon::Bug => "bug",
            Icon::RunTile => "runTile",
            Icon::Terminal => "terminal",
            Icon::Letters => "letters",
            Icon::Lines => "lines",
            Icon::Halves => "halves",
            Icon::Eye => "eye",
            Icon::Start => "start",
        }
    }

    /// The icon with this `SbIcon` name, or `None`.
    pub fn named(name: &str) -> Option<Icon> {
        Icon::every().find(|icon| icon.name() == name)
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
            // Unluminous's `<line>`, `<polyline>` and `<polygon>` children are written as path data with
            // the same points, which is the same drawing; `<circle>` and `<rect>` keep their typed shapes.
            Icon::Clock => const {
                &[circle(12.0, 12.0, 5.0), p("M12 12L12 8.8"), p("M12 12L14.6 12.6")]
            },
            Icon::Key => const {
                &[
                    circle(8.5, 10.5, 2.6),
                    p("M10.3 9.1L16.5 8"),
                    p("M13.09 8.605L14.69 11.005"),
                    p("M14.95 8.275L16.55 10.675"),
                ]
            },
            Icon::Tick => const { &[p("M8 12L10.8 15"), p("M10.8 15L16 8.8")] },
            Icon::Cross => const { &[p("M8 8L16 16"), p("M8 16L16 8")] },
            Icon::File => const {
                &[
                    p("M8.9 6.5L12.8 6.5L16.2 9.9L16.2 16.4L15.1 17.5L8.9 17.5L7.8 16.4L7.8 7.6Z"),
                    p("M12.8 6.5L12.8 9.3L13.4 9.9L16.2 9.9"),
                ]
            },
            Icon::Rerun => const {
                &[
                    p("M13.783 7.115L15.097 7.823L16.156 8.874L16.872 10.183L17.188 11.642L17.076 13.13L16.546 14.525L15.642 15.712L14.438 16.593L13.033 17.096L11.543 17.18L10.091 16.837L8.796 16.096L7.765 15.017L7.082 13.69L6.805 12.224L6.955 10.739L7.521 9.358L8.455 8.195"),
                    solid(Shape::Path("M13.685 6.32L18.285 4.32L17.885 9.32Z")),
                ]
            },
            Icon::Photo => const {
                &[
                    rect(5.5, 6.5, 13.0, 11.0, 1.0),
                    solid(Shape::Circle { cx: 15.1, cy: 9.7, r: 1.2 }),
                    solid(Shape::Path("M7 16L10 11.5L13 16Z")),
                    solid(Shape::Path("M11 16L14 13L17 16Z")),
                ]
            },
            Icon::Send => const {
                &[
                    Part::veiled(Shape::Path("M21.5 2.5 14.6 21.4l-4-8-8-4z"), 0.22),
                    p("M21.5 2.5 10.6 13.4"),
                ]
            },
            Icon::Project => const { &[p("M3.5 7.5a2 2 0 0 1 2-2h3.6l2 2h7.4a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z")] },
            Icon::Document => const { &[p("M7 3.5h7l4 4v11a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2v-13a2 2 0 0 1 2-2zM14 3.5v4h4M8.5 12h7M8.5 15.5h4.5")] },
            Icon::Git => const { &[p("M7 4a2 2 0 1 1 0 4a2 2 0 1 1 0-4zM7 16a2 2 0 1 1 0 4a2 2 0 1 1 0-4zM17 6a2 2 0 1 1 0 4a2 2 0 1 1 0-4zM7 8v8M17 10c0 3.5-3 4.5-8.6 6.4")] },
            Icon::Realm => const { &[p("M6 4.8a2.2 2.2 0 1 1 0 4.4a2.2 2.2 0 1 1 0-4.4zM18 4.8a2.2 2.2 0 1 1 0 4.4a2.2 2.2 0 1 1 0-4.4zM12 14.8a2.2 2.2 0 1 1 0 4.4a2.2 2.2 0 1 1 0-4.4zM8.2 7h7.6M7.2 8.9l3.6 6.2M16.8 8.9l-3.6 6.2")] },
            Icon::Bubble => const { &[p("M5 7a2.5 2.5 0 0 1 2.5-2.5h9A2.5 2.5 0 0 1 19 7v5.5a2.5 2.5 0 0 1-2.5 2.5H11l-4 3.5V15h0.5A2.5 2.5 0 0 1 5 12.5zM9 9.8h6")] },
            Icon::Board => const { &[p("M4.5 6a1.5 1.5 0 0 1 1.5-1.5h12A1.5 1.5 0 0 1 19.5 6v12a1.5 1.5 0 0 1-1.5 1.5H6A1.5 1.5 0 0 1 4.5 18zM9.5 4.5v15M14.5 4.5v15")] },
            Icon::Database => const { &[p("M5 6.5C5 5.1 8.1 4 12 4s7 1.1 7 2.5S15.9 9 12 9 5 7.9 5 6.5zM5 6.5v11c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5v-11M5 12c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5")] },
            Icon::Bug => const { &[p("M8.5 10a3.5 3.5 0 0 1 7 0v3.5a3.5 3.5 0 0 1-7 0zM12 10v7M8.5 12.5H5M15.5 12.5H19M8.8 16l-2.6 2M15.2 16l2.6 2M8.8 9l-2.4-2M15.2 9l2.4-2M10 6.8 9 5M14 6.8l1-1.8")] },
            Icon::RunTile => const { &[p("M12 3.5a8.5 8.5 0 1 1 0 17a8.5 8.5 0 1 1 0-17zM10.3 8.9v6.2l4.9-3.1z")] },
            Icon::Terminal => const { &[p("M4 6.5a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM7.8 9.5l2.8 2.5-2.8 2.5M12.6 15h3.8")] },
            Icon::Letters => const { &[p("M3.5 18L8 6.5h1L13.5 18M5.2 14h6.6M20 12.2v5.8M20 15a2.2 2.2 0 1 1 0-0.01")] },
            Icon::Lines => const { &[p("M5 7h14M5 11h14M5 15h14M5 19h8")] },
            Icon::Halves => const { &[p("M4 6.5a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM12 4.5v15")] },
            Icon::Eye => const { &[p("M2.8 12s3.4-6 9.2-6 9.2 6 9.2 6-3.4 6-9.2 6-9.2-6-9.2-6zM12 9.3a2.7 2.7 0 1 1 0 5.4a2.7 2.7 0 1 1 0-5.4z")] },
            // `M8 5.5v13l10.5-6.5z` with `fill-opacity: 0.18`: the run button's triangle, tinted inside.
            Icon::Start => const { &[Part::veiled(Shape::Path("M8 5.5v13l10.5-6.5z"), 0.18)] },
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
        for icon in Icon::every() {
            assert!(!icon.name().is_empty());
            assert!(!icon.parts().is_empty(), "{} has no parts", icon.name());
            assert_eq!(Icon::named(icon.name()), Some(icon));
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
