# `libsurfacesam`

![CI](https://github.com/linux-surface/libsurfacesam/workflows/CI/badge.svg)

Library for Linux Surface System Aggregator Module (SSAM) kernel driver user-space API (cdev).
Provides an interface for the `surface_aggregator_cdev` kernel module.

The following crates are provided:
- `ssam`: Main API wrapper.
- `ssam-tokio`: [`tokio`][tokio] compatibility layer for asynchronous event handling.
