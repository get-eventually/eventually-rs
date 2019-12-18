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
    <!-- Crates.io -->
    <a href="https://crates.io/crates/eventually">
        <img alt="Crates.io"
        src="https://img.shields.io/crates/v/eventually?style=flat-square">
    </a>
    <!-- Docs.rs -->
    <a href="https://docs.rs/eventually">
        <img alt="docs.rs docs"
        src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" />
    </a>
    <!-- License -->
    <a href="https://github.com/ar3s3ru/eventually-rs/blob/master/LICENSE">
        <img alt="GitHub license"
        src="https://img.shields.io/github/license/ar3s3ru/eventually-rs?style=flat-square">
    </a>
</div>

<br />

Collection of traits and other utilities to help you build your Event-sourced applications in Rust.

## Versioning

This library is **actively being developed**, and prior to `v1` release the following [Semantic versioning]()
is being adopted:

* Breaking changes are tagged with a new `MINOR` release
* New features, patches and documentation are tagged with a new `PATCH` release

## Project Layout

Eventually is a workspace containing different sub-crates, as follows:

* [`eventually`](eventually): contains foundation traits and types for domain modeling
and event store.

* [`eventually-memory`](eventually-memory): contains an in-memory event store implementation.

* [`eventually-examples`](eventually-examples): contains different examples on how to use
the library.

## Examples

### A simple Order aggregate

```rust
use chrono::{DateTime, Utc};

use eventually::Aggregate;

enum OrderEvent {
    Created { id: String, on: DateTime<Utc> },
    ItemAdded { item: String, quantity: u32 },
    Purchased { on: DateTime<Utc> },
}

struct OrderState {
    id: String,
    created_at: DateTime<Utc>,
    items: Vec<(String, u32)>,
    purchased_on: Option<DateTime<Utc>>,
}

enum OrderError {
    AlreadyCreated,
    NotYetCreated,
    AlreadyPurchased,
}

struct OrderAggregate;
impl Aggregate for OrderAggregate {
    type State = Option<OrderState>;
    type Event = OrderEvent;
    type Error = OrderError;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        use OrderEvent::*;

        match state {
            None => match event {
                Created { id, on } => Ok(Some(OrderState {
                    id,
                    created_at: on,
                    items: Vec::new(),
                    purchased_on: None,
                })),
                _ => Err(OrderError::NotYetCreated),
            },
            
            Some(mut state) => match event {
                Created { .. } => Err(OrderError::AlreadyCreated),
                ItemAdded { item, quantity } => {
                    if state.purchased_on.is_none() {
                        state.items.push((item, quantity));
                        Ok(state)
                    } else {
                        Err(OrderError::AlreadyPurchased)
                    }
                },
                Purchased { on } => {
                    state.purchased_on = Some(on);
                    Ok(state)
                },
            }
        }
    }
}
```

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `eventually-rs` by you, shall be licensed as MIT, without any additional terms or conditions.
