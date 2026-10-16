# rux documentation

This folder holds the documentation for rux, the Black Rainbow Labs component library for Rust. Each
page can be read on its own.

The tables below list the pages in reading order for a Rust programmer who has used `egui` and has never
looked inside rux. Start with what rux is, then build a first window, then read about the components you
need.

## Start here

| | Page | What it covers |
|---|---|---|
| 1 | [Product overview](product-overview.md) | What rux is, who it is for, and how it compares with `egui` on its own |
| 2 | [Getting started](getting-started.md) | Adding the dependency, the three calls every frame makes, and a whole first window |
| 3 | [Glossary](glossary.md) | Each design and drawing term the pages use, in a sentence or two |

## Components

| | Page | What it covers |
|---|---|---|
| 4 | [Components](components/README.md) | Every component, with a picture of each group and a table of where each is documented |
| 5 | [Buttons](components/buttons.md) | `Button` and `IconButton` |
| 6 | [Fields and selects](components/fields.md) | `TextInput`, `TextArea`, `Stepper` and `Select` |
| 7 | [Toggles](components/toggles.md) | `Segmented`, `DiceToggle` and `Switch` |
| 8 | [Navigation](components/navigation.md) | `TopNav` and `ProjectMenu` |
| 9 | [Surfaces, rows and chips](components/surfaces.md) | `Panel`, `Well`, `SubGroup`, `Divider`, `ListItem`, `Chip`, `Avatar`, `Tile` and `Scrollbar` |
| 10 | [Loaders](components/loaders.md) | `Spinner`, `Progress` and `Skeleton` |
| 11 | [Dialogs](components/modal.md) | `Modal`, as a centred dialog and as a sheet from the bottom edge |
| 12 | [Instruments](components/instruments.md) | Key caps, checkboxes, faders, meters, charts, tables and timelines, drawn as hardware |
| 13 | [Agent-Chat parts](components/agent-chat.md) | The charts, readouts, buttons, status wells and cards Unluminous's chat pane is drawn with |
| 14 | [Agent-Tasks parts](components/agent-tasks.md) | The view switch, sprint ring, lanes, task cards and ticket fields of Unluminous's task board |
| 15 | [Icons](icons.md) | All 63 marks, how to draw one, and how to add one |
| 16 | [Example pages](examples.md) | The Image Creator page, rebuilt from the components |

## Foundations

| | Page | What it covers |
|---|---|---|
| 17 | [Themes](themes.md) | Every token, the two themes, the Agent-Chat values, and a theme made from another application's colours |
| 18 | [Glass](glass.md) | A glazed theme, whose raised surfaces bend the picture behind the window, and how to give it a picture |
| 19 | [Layout](layout.md) | `Flex`, `Grid`, `Pen` and `Pad`, and how components are measured and placed |
| 20 | [How rux draws](architecture.md) | `egui` and `vello_cpu` together, layers, the rule that keeps a still frame free, and the measurements |
| 21 | [Mobile](mobile.md) | What a component does differently on a phone, the mobile preview, and the iOS and Android builds |

## Working on it

| | Page | What it covers |
|---|---|---|
| 22 | [Repository and building](repository.md) | The crates, the commands, the capture tools, the conventions and the versions that have to match |
| 23 | [Testing](testing.md) | The four layers of tests, the reference check, and how the pictures are taken |
| 24 | [Writing style](writing-style.md) | How to write and keep a page in this folder correct |
| | [`CLAUDE.md`](../CLAUDE.md) | The conventions a change has to follow, with the reason for each |
| | [`tasks/`](../tasks) | The design documents, with the options that were weighed and refused |

## Pictures

Every picture in `images/` is taken by `pwsh tools/capture-docs.ps1` from the storybook, and by
`pwsh tools/capture-mobile.ps1 -Out docs/images/mobile` from the mobile preview. A folder is named after
its storybook story, and each holds `dark-neumorphic.png` and `light-neumorphic.png`, plus
`--open` versions for the stories with a menu or a dialog. Take a picture again after changing a
component. Never edit one by hand.
