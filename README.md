<h1 align="center">Eventually</h1>
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

## Usage

Add `eventually` into your project dependencies:

```toml
[dependencies]
eventually = "0.3"
```

Check out [`eventually-test`](eventually-test) crate to see how to use the different
components offered by the crate.

## Project Layout

Eventually is a workspace containing different sub-crates, as follows:

* [`eventually`](eventually): crate containing the public API -- users should only depend on this crate,
* [`eventually-core`](eventually-core): contains foundation traits and types to use Event Sourcing,
* [`eventually-util`](eventually-util): contains set of extensions built on top
of the foundation traits contained in the core crate,
* [`eventually-postgres`](eventually-postgres): contains a PostgreSQL-backed Event Store implementation, to permanently store Domain Events,
* [`eventually-test`](eventually-test): contains an _HTTP service_ example application build using `eventually` and [`tide`](https://github.com/http-rs/tide).


## Versioning

This library is **actively being developed**, and prior to `v1` release the following [Semantic versioning]()
is being adopted:

* Breaking changes are tagged with a new `MINOR` release
* New features, patches and documentation are tagged with a new `PATCH` release

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `eventually-rs` by you, shall be licensed as MIT, without any additional terms or conditions.
