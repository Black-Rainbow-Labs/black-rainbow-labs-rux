//! Example pages built out of the `rux` components.
//!
//! `pages/` is the crate root rather than `src/`, because that is the folder the work was asked for
//! in; `Cargo.toml` says `[lib] path = "pages/mod.rs"` and nothing has been moved to satisfy a
//! convention.
//!
//! A page here is a **page, not a picture**: its selects open, its panels collapse, its groups fold,
//! its stepper counts, its modal opens and closes, and its text is editable. That is what makes it a
//! test of the components rather than a screenshot of them, and it is the thing that finds the
//! component that cannot do what a real page needs.
//!
//! Each page holds its own state and takes it by `&mut`, so the storybook can show one without the
//! library owning a page's model.

pub mod image_creator;

pub use image_creator::ImageCreator;

/// Every page here, for a host that wants to list them.
pub const PAGES: &[(&str, &str)] = &[(
    "image-creator",
    "Image Creator — the AI Studio reference page, rebuilt from the components",
)];
