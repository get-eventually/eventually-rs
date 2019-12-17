use eventually::{
    command::dispatcher::{Dispatcher, Error},
    optional::Handler,
    versioned::HandlerExt,
};

use rand::prelude::*;

type DispatchError = Error<point::EventError, point::CommandError, std::convert::Infallible>;

fn main() {
    let store = eventually_memory::MemoryStore::<String, point::Event>::default();
    let handler = point::CommandHandler.as_handler().versioned();

    let mut dispatcher = Dispatcher::new(store, handler);

    let result: Result<(), DispatchError> = tokio_test::block_on(async {
        println!(
            "Register: {:?}",
            dispatcher
                .dispatch(point::Command::Register {
                    id: "MyPoint".to_string(),
                })
                .await?
        );

        for _i in 0..1000 - 1 {
            let mut rng = rand::thread_rng();
            let random: i32 = rng.gen::<i32>() % 100;
            let choice: u8 = rng.gen::<u8>() % 4;

            println!(
                "State: {:?}",
                dispatcher
                    .dispatch(match choice {
                        0 => point::Command::GoUp {
                            id: "MyPoint".to_string(),
                            v: random
                        },
                        1 => point::Command::GoDown {
                            id: "MyPoint".to_string(),
                            v: random
                        },
                        2 => point::Command::GoLeft {
                            id: "MyPoint".to_string(),
                            v: random
                        },
                        3 => point::Command::GoRight {
                            id: "MyPoint".to_string(),
                            v: random
                        },
                        _ => panic!("should not be entered"),
                    })
                    .await?
            );
        }

        dispatcher
            .dispatch(point::Command::Register {
                id: "MyPoint".to_string(),
            })
            .await?;

        Ok(())
    });

    assert_eq!(
        result,
        Err(Error::CommandFailed(
            point::CommandError::AlreadyRegistered("MyPoint".to_string())
        ))
    );

    println!("Result: {}", result.unwrap_err());
}
