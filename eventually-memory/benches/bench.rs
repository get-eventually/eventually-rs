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

fn insert_new_events_concurrently_4_threads() {
    let store = std::sync::Arc::new(std::sync::Mutex::new(
        MemoryStore::<&'static str, Event>::default(),
    ));

    (0..4)
        .map(|_id| {
            std::thread::spawn({
                let clone = store.clone();
                move || {
                    let mut store = clone.lock().unwrap();
                    tokio_test::block_on(store.append(
                        "benchtest",
                        0,
                        vec![Event::A, Event::B, Event::C],
                    ))
                    .unwrap();
                }
            })
        })
        .for_each(|handle| handle.join().unwrap());
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

    insert_elements(&mut store, "benchtest3", 3);
    insert_elements(&mut store, "benchtest10", 10);
    insert_elements(&mut store, "benchtest100", 100);
    insert_elements(&mut store, "benchtest1000", 1000);

    c.bench_function("insert new events", |b| {
        b.iter(|| insert_new_events(black_box(&mut store)))
    });

    c.bench_function("insert new events (4 threads)", |b| {
        b.iter(insert_new_events_concurrently_4_threads)
    });

    c.bench_function("read stream events 3", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest3")))
    });

    c.bench_function("read stream events 10", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest10")))
    });

    c.bench_function("read stream events 100", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest100")))
    });

    c.bench_function("read stream events 1000", |b| {
        b.iter(|| read_event_stream(black_box(&store), black_box("benchtest1000")))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
