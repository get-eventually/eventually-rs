use std::sync::atomic::AtomicBool;
use std::sync::Once;
use std::thread;

use chrono::Utc;

use envconfig::Envconfig;

use lazy_static::lazy_static;

use reqwest::{Client, Response, StatusCode};

use eventually_test::config::Config;
use eventually_test::order::{Order, OrderItem};

static START: Once = Once::new();

lazy_static! {
    static ref SERVER_STARTED: AtomicBool = AtomicBool::default();
}

fn setup() {
    START.call_once(|| {
        thread::spawn(move || {
            let config = Config::init().unwrap();
            SERVER_STARTED.store(true, std::sync::atomic::Ordering::SeqCst);

            smol::run(eventually_test::run(config));
        });
    });

    while !SERVER_STARTED.load(std::sync::atomic::Ordering::SeqCst) {}
}

#[tokio::test]
async fn it_creates_an_order_successfully() {
    setup();

    let url = format!("http://localhost:8080/orders/test/create");
    let client = Client::new();

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

    // Sending a Create command again will cause the endpoint to return '409 Conflict'.
    let root: Response = client.post(&url).send().await.unwrap();
    assert_eq!(StatusCode::CONFLICT, root.status());
}

#[tokio::test]
async fn it_adds_items_to_an_order_successfully() {
    setup();

    let id = "test-add-items";
    let url = format!("http://localhost:8080/orders/{}/create", id);
    let client = Client::new();

    let response: Response = client.post(&url).send().await.unwrap();

    assert_eq!(StatusCode::CREATED, response.status());

    let url = format!("http://localhost:8080/orders/{}/add-item", id);

    let item = OrderItem {
        item_sku: "ITEM-SKU".to_owned(),
        quantity: 10,
        price: 5.99,
    };

    let order: Order = client
        .post(&url)
        .json(&item)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(order.is_editable());
    assert_eq!(1, order.items().len());
    assert_eq!(&item, order.items().first().unwrap());
}
