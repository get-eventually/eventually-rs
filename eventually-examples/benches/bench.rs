#![allow(warnings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use futures::stream::StreamExt;

use rand::prelude::*;

use eventually::command::dispatcher::{DirectDispatcher, Error};
use eventually::command::Dispatcher;
use eventually::optional::{AsAggregate, AsHandler, CommandHandler};
use eventually_memory::Store as MemoryStore;

fn dispatch<D>(dispatcher: &mut D, id: &'static str)
where
    D: Dispatcher<CommandHandler = AsHandler<point::Handler>>,
{
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
    let store = MemoryStore::<String, point::Event>::default();
    let handler = point::Handler.as_handler();

    let mut dispatcher = DirectDispatcher::new(store, handler);

    tokio_test::block_on(dispatcher.dispatch(point::Command::Register {
        id: "benchmark".to_string(),
    }));

    c.bench_function("dispatch", |b| {
        b.iter(|| dispatch(black_box(&mut dispatcher), black_box("benchmark")))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
