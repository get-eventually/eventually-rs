#![allow(warnings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use futures::stream::StreamExt;

use eventually::store::{ReadStore, WriteStore};
use eventually_memory::MemoryStore;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Event {
    A,
    B,
    C,
}

fn insert_new_events(store: &mut MemoryStore<&'static str, Event>) {
    tokio_test::block_on(store.append("benchtest", 0, vec![Event::A, Event::B, Event::C])).unwrap()
}

fn read_event_stream(store: &MemoryStore<&'static str, Event>, source_id: &'static str) {
    tokio_test::block_on(store.stream(source_id, 0).collect::<Vec<Event>>());
}

fn insert_elements(store: &mut MemoryStore<&'static str, Event>, name: &'static str, num: usize) {
    tokio_test::block_on(store.append(
        name,
        0,
        (0..=num).map(|_idx| Event::A).collect::<Vec<Event>>(),
    ))
    .unwrap()
}

fn benchmark(c: &mut Criterion) {
    let mut store = MemoryStore::<&'static str, Event>::default();

    insert_elements(&mut store, "benchtest10", 10);
    insert_elements(&mut store, "benchtest100", 100);
    insert_elements(&mut store, "benchtest1_000", 1000);
    insert_elements(&mut store, "benchtest10_000", 10000);
    insert_elements(&mut store, "benchtest100_000", 100000);
    insert_elements(&mut store, "benchtest1_000_000", 1000000);

    c.bench_function("insert events 10", |b| {
        b.iter(|| insert_elements(black_box(&mut store), black_box("insert_benchtest10"), 10))
    });

    c.bench_function("insert events 100", |b| {
        b.iter(|| insert_elements(black_box(&mut store), black_box("insert_benchtest100"), 100))
    });

    c.bench_function("insert events 1_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(&mut store),
                black_box("insert_benchtest1_000"),
                1000,
            )
        })
    });

    c.bench_function("insert events 10_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(&mut store),
                black_box("insert_benchtest10_000"),
                10000,
            )
        })
    });

    c.bench_function("insert events 100_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(&mut store),
                black_box("insert_benchtest100_000"),
                100000,
            )
        })
    });

    c.bench_function("insert events 1_000_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(&mut store),
                black_box("insert_benchtest1_000_000"),
                1000000,
            )
        })
    });

    c.bench_function("read stream events 10", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest10")))
    });

    c.bench_function("read stream events 100", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest100")))
    });

    c.bench_function("read stream events 1_000", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest1_000")))
    });

    c.bench_function("read stream events 10_000", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest10_000")))
    });

    c.bench_function("read stream events 100_000", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest100_000")))
    });

    c.bench_function("read stream events 1_000_000", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest1_000_000")))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
