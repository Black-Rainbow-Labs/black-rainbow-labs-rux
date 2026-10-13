# Example pages

`examples/pages/image_creator.rs` rebuilds the AI Studio Image Creator page out of rux components. It is
the test that the components can make a real page: if a page needs something a component cannot do,
this is where it shows.

Storybook: **Pages → Image Creator**. It fills the storybook's content area and is fully interactive.

![The Image Creator page in the storybook, dark theme](images/image-creator/dark-neumorphic.png)

<details><summary>The Image Creator page in the light theme</summary>

![The Image Creator page in the storybook, light theme](images/image-creator/light-neumorphic.png)

</details>

## What is on it

| Area | Components |
|---|---|
| The top bar | `ProjectMenu`, `TopNav`, and a `Chip` saying where results are saved |
| The prompt panel | a `TextArea` with a count and a corner button, two `Button`s, a size `Select`, a `Stepper`, a workflow `Select`, and the primary `Button` |
| The controls panel | an accent `Chip` with a dismiss button, four `SubGroup`s, and labelled `Select`s, each with a `DiceToggle` |
| The results canvas | a `Grid` of `Tile`s with `Spinner`s and elapsed `Chip`s, and a `Button` to clear them |
| The Prompt Manager | a `Modal` with a list of `ListItem`s on the left and an editor on the right, from `split_body` |

The panels collapse, the sub groups fold, the selects open, the stepper counts, the dice toggle, the
prompt text is editable with its count following it, and the Prompt Manager opens from **Prompts** and
closes with Escape.

## How it is built

```rust
pub struct ImageCreator { /* every value the page shows */ }

impl ImageCreator {
    pub fn new() -> Self { /* ... */ }
    pub fn show(&mut self, rux: &mut Rux<'_>, area: Rect) { /* ... */ }
}
```

The page keeps its own state in `ImageCreator` and takes it by `&mut`, so the storybook can show it
without the library owning any of the page's data. This is the shape an application's own pages take.

Read it alongside the reference: `reference/image-creator-page.jsx.html` is the React page and
`reference/image-creator.css` is its stylesheet. The rule names in the comments in
`examples/pages/image_creator.rs` are the rules in that stylesheet.

## The mobile application

`mobile/app` is the second example: the same kind of page on a phone, as three screens. See
[Mobile](mobile.md).
