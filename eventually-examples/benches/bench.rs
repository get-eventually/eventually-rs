#![allow(warnings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use futures::stream::StreamExt;

use rand::prelude::*;

use eventually::{
    aggregate::optional::AsAggregate,
    command::{
        dispatcher::Dispatcher,
        r#static::{AsHandler, StaticHandler},
    },
};
use eventually_memory::MemoryStore;

type DispatcherType =
    Dispatcher<MemoryStore<std::string::String, point::Event>, AsHandler<point::CommandHandler>>;

fn dispatch(dispatcher: &mut DispatcherType, id: &'static str) {
    let mut rng = rand::thread_rng();
    let random: i32 = rng.gen::<i32>() % 100;
    let choice: u8 = rng.gen::<u8>() % 4;

    tokio_test::block_on(dispatcher.dispatch(match choice {
        0 => point::Command::GoUp {
            id: id.to_string(),
            v: random,
        },
        1 => point::Command::GoDown {
            id: id.to_string(),
            v: random,
        },
        2 => point::Command::GoLeft {
            id: id.to_string(),
            v: random,
        },
        3 => point::Command::GoRight {
            id: id.to_string(),
            v: random,
        },
        _ => panic!("should not be entered"),
    }));
}

fn benchmark(c: &mut Criterion) {
    let store = eventually_memory::MemoryStore::<String, point::Event>::default();
    let handler = point::CommandHandler::as_handler();

    let mut dispatcher = Dispatcher::new(store, handler);

    tokio_test::block_on(dispatcher.dispatch(point::Command::Register {
        id: "benchmark".to_string(),
    }));

    c.bench_function("dispatch", |b| {
        b.iter(|| dispatch(black_box(&mut dispatcher), black_box("benchmark")))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
