# Bevy Alternative UI Navigation (Lite)

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)

A generic UI navigation algorithm for the
[Bevy](https://github.com/bevyengine/bevy) engine default UI library.

Based on [`bevy-ui-navigation`](https://github.com/nicopap) but stripped down to remove support for `cuicui_layout` and `bevy_mod_picking`.

Offered with limited support -- I plan to keep this updated with the latest Bevy release and fix bugs, but I am not interested in adding new features.

```toml
[dependencies]
bevy-alt-ui-navigation-lite = "0.1"
```

## Changelog

See the changelog at <CHANGELOG.md>

## Version matrix

| `bevy` | `bevy-alt-ui-navigation-lite` |
|------|--------|
| 0.13 | 0.1.0  |

## License

This project is a derivative of [`bevy-ui-navigation`](https://github.com/nicopap/ui-navigation).

Copyright © 2022 Nicola Papale.

`bevy-ui-navigation` is licensed under either MIT or Apache 2.0. See
[`licenses-bevy-ui-navigation`](licenses-bevy-ui-navigation) directory for details.

This project is licensed under either MIT or Apache 2.0. See
[`licenses`](licenses) directory for details.
