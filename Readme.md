# Bevy Alternative UI Navigation (Lite)

[![crates.io](https://img.shields.io/crates/v/bevy-alt-ui-navigation-lite.svg)](https://crates.io/crates/bevy-alt-ui-navigation-lite)
[![docs](https://docs.rs/bevy-alt-ui-navigation-lite/badge.svg)](https://docs.rs/bevy-alt-ui-navigation-lite)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking)

A generic UI navigation algorithm for the
[Bevy](https://github.com/bevyengine/bevy) engine default UI library.

Based on [`bevy-ui-navigation`](https://github.com/nicopap) but stripped down to remove support for `cuicui_layout` and `bevy_mod_picking`.

Maintenance of this project is becoming burdensome for me and I am considering archiving it soon.

Many simpler use-cases can now be handled by Bevy's [directional navigation](https://bevy.org/examples/ui-user-interface/directional-navigation/). Consider using it instead, if possible.

```toml
[dependencies]
bevy-alt-ui-navigation-lite = "0.4"
```

## Changelog

See the changelog at [`CHANGELOG.md`](./CHANGELOG.md)

## Version matrix

| `bevy` | `bevy-alt-ui-navigation-lite` |
|------|------|
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
