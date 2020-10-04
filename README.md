<br />

<div align="center">
    <img alt="Eventually" src = "./resources/logo.png" width = 300>
</div>

<br />

<div align="center">
    <strong>
        Event Sourcing for Rust
    </strong>
</div>

<br />

<div align="center">
    <!-- Testing pipeline -->
    <a href="https://github.com/ar3s3ru/eventually-rs/actions?query=workflow%3A%22Rust+%28stable%29%22">
        <img alt="GitHub Workflow Status"
        src="https://img.shields.io/github/workflow/status/ar3s3ru/eventually-rs/Rust%20(stable)?style=flat-square">
    </a>
    <!-- Codecov -->
    <a href="https://codecov.io/gh/ar3s3ru/eventually-rs">
            <img alt="Codecov"
            src="https://img.shields.io/codecov/c/github/ar3s3ru/eventually-rs?style=flat-square">
    </a>
    <!-- Crates.io -->
    <a href="https://crates.io/crates/eventually">
        <img alt="Crates.io"
        src="https://img.shields.io/crates/v/eventually?style=flat-square">
    </a>
    <!-- Github pages docs -->
    <a href="https://ar3s3ru.github.io/eventually-rs/eventually/">
        <img alt="latest master docs"
        src="https://img.shields.io/badge/docs-master-important?style=flat-square" />
    </a>
    <!-- Docs.rs -->
    <a href="https://docs.rs/eventually">
        <img alt="docs.rs docs"
        src="https://img.shields.io/badge/dynamic/json?style=flat-square&color=blue&label=docs.rs&query=%24.crate.newest_version&url=https%3A%2F%2Fcrates.io%2Fapi%2Fv1%2Fcrates%2Feventually" />
    </a>
    <!-- License -->
    <a href="https://github.com/ar3s3ru/eventually-rs/blob/master/LICENSE">
        <img alt="GitHub license"
        src="https://img.shields.io/github/license/ar3s3ru/eventually-rs?style=flat-square">
    </a>
    <!-- Gitter -->
    <a href="https://gitter.im/eventually-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge">
        <img alt="Gitter"
         src="https://img.shields.io/gitter/room/eventually-rs/community?style=flat-square">
    </a>
</div>

<br />


Collection of traits and other utilities to help you build your Event-sourced applications in Rust.

## What is Event Sourcing?

Before diving into the crate's internals,you might be wondering what
Event Sourcing is.

From eventstore.com introduction:

>Event Sourcing is an architectural pattern that is gaining popularity as a method for building modern systems. Unlike traditional databases which only store and update the current state of data, event-sourced systems store all changes as an immutable series of events in the order that they occurred and current state is derived from that event log.

## How does `eventually` support Event Sourcing?

`eventually` exposes all the necessary abstraction to model your
Doman Entities (in lingo, _Aggregates_) using Domain Events, and
to save these Events using an _Event Store_ (the append-only event log).

For more information, [check out the crate documentation](https://docs.rs/eventually).

You can also take a look at [`eventually-app-example`](./eventually-app-example),
showcasing an example event-sourced microservice using HTTP transport layer.

All other questions are more than welcome on our [Gitter chat](https://gitter.im/eventually-rs/community).

## Installation

Add `eventually` into your project dependencies:

```toml
[dependencies]
eventually = { version = "0.4.0-alpha.3", features = ["full"] }
```

### Note on semantic versioning

This library is **actively being developed**, and prior to `v1` release the following [Semantic versioning]()
is being adopted:

* Breaking changes are tagged with a new `MINOR` release
* New features, patches and documentation are tagged with a new `PATCH` release

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `eventually-rs` by you, shall be licensed as MIT, without any additional terms or conditions.
