use criterion::{black_box, criterion_group, criterion_main, Criterion};

use futures::stream::TryStreamExt;

use eventually_core::store::{EventStore, Expected, Select};
use eventually_util::inmemory::EventStore as InMemoryStore;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Event {
    A,
    B,
    C,
}

fn read_event_stream(store: InMemoryStore<&'static str, Event>, source_id: &'static str) {
    tokio_test::block_on(async move {
        store
            .stream(source_id, Select::All)
            .await
            .unwrap()
            .try_fold(0u32, |acc, _x| async move { Ok(acc + 1) })
            .await
    })
    .unwrap();
}

fn insert_elements(mut store: InMemoryStore<&'static str, Event>, name: &'static str, num: usize) {
    tokio_test::block_on(
        store.append(
            name,
            Expected::Any,
            (0..=num)
                .map(|idx| match idx % 3 {
                    0 => Event::A,
                    1 => Event::B,
                    2 => Event::C,
                    _ => panic!("unreachable code"),
                })
                .collect::<Vec<Event>>(),
        ),
    )
    .unwrap();
}

fn benchmark(c: &mut Criterion) {
    let store = InMemoryStore::<&'static str, Event>::default();

    insert_elements(store.clone(), "benchtest10", 10);
    insert_elements(store.clone(), "benchtest100", 100);
    insert_elements(store.clone(), "benchtest1_000", 1_000);
    insert_elements(store.clone(), "benchtest10_000", 10_000);
    insert_elements(store.clone(), "benchtest100_000", 100_000);
    insert_elements(store.clone(), "benchtest1_000_000", 1_000_000);

    c.bench_function("insert events 10", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest10"),
                10,
            )
        })
    });

    c.bench_function("insert events 100", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest100"),
                100,
            )
        })
    });

    c.bench_function("insert events 1_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest1_000"),
                1_000,
            )
        })
    });

    c.bench_function("insert events 10_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest10_000"),
                10_000,
            )
        })
    });

    c.bench_function("insert events 100_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest100_000"),
                100_000,
            )
        })
    });

    c.bench_function("insert events 1_000_000", |b| {
        b.iter(|| {
            insert_elements(
                black_box(store.clone()),
                black_box("insert_benchtest1_000_000"),
                1_000_000,
            )
        })
    });

    c.bench_function("read stream events 10", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest10")))
    });

    c.bench_function("read stream events 100", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest100")))
    });

    c.bench_function("read stream events 1_000", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest1_000")))
    });

    c.bench_function("read stream events 10_000", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest10_000")))
    });

    c.bench_function("read stream events 100_000", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest100_000")))
    });

    c.bench_function("read stream events 1_000_000", |b| {
        b.iter(|| read_event_stream(black_box(store.clone()), black_box("benchtest1_000_000")))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
