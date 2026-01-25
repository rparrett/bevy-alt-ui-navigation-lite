# Bevy Alternative UI Navigation (Lite)

[![crates.io](https://img.shields.io/crates/v/bevy-alt-ui-navigation-lite.svg)](https://crates.io/crates/bevy-alt-ui-navigation-lite)
[![docs](https://docs.rs/bevy-alt-ui-navigation-lite/badge.svg)](https://docs.rs/bevy-alt-ui-navigation-lite)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking)

> [!NOTE]
> This project has been archived now that Bevy 0.18 has built-in [automatic directional navigation](https://bevy.org/news/bevy-0-18/#automatic-directional-navigation).

A generic UI navigation algorithm for the
[Bevy](https://github.com/bevyengine/bevy) engine default UI library.

Based on [`bevy-ui-navigation`](https://github.com/nicopap) but stripped down to remove support for `cuicui_layout` and `bevy_mod_picking`.

```toml
[dependencies]
bevy-alt-ui-navigation-lite = "0.5"
```

## Changelog

See the changelog at [`CHANGELOG.md`](./CHANGELOG.md)

## Version matrix

| `bevy` | `bevy-alt-ui-navigation-lite` |
|------|------|
| 0.17 | 0.5  |
| 0.16 | 0.4  |
| 0.15 | 0.3  |
| 0.14 | 0.2  |
| 0.13 | 0.1  |

## License

This project is a derivative of [`bevy-ui-navigation`](https://github.com/nicopap/ui-navigation).

Copyright © 2022 Nicola Papale.

`bevy-ui-navigation` is licensed under either MIT or Apache 2.0. See
[`licenses-bevy-ui-navigation`](licenses-bevy-ui-navigation) directory for details.

This project is licensed under either MIT or Apache 2.0. See
[`licenses`](./licenses) directory for details.
