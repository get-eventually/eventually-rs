use std::sync::atomic::AtomicBool;
use std::sync::Once;
use std::thread;

use chrono::Utc;

use envconfig::Envconfig;

use lazy_static::lazy_static;

use eventually_test::config::Config;
use eventually_test::order::Order;

static START: Once = Once::new();

lazy_static! {
    static ref SERVER_STARTED: AtomicBool = AtomicBool::default();
}

fn setup() {
    START.call_once(|| {
        thread::spawn(move || {
            let config = Config::init().unwrap();
            SERVER_STARTED.store(true, std::sync::atomic::Ordering::SeqCst);

            smol::run(eventually_test::run(config)).expect("don't fail :(");
        });
    });

    // Busy loading :(
    while !SERVER_STARTED.load(std::sync::atomic::Ordering::SeqCst) {}
}

#[test]
fn it_creates_an_order_successfully() {
    setup();

    smol::run(async {
        let url = format!("http://localhost:8080/orders/test/create");
        let client = reqwest::Client::new();

        let start = Utc::now();

        let root: Order = client
            .post(&url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert!(root.created_at() >= start);
        assert!(root.is_editable());
        assert!(root.items().is_empty());
    });
}
