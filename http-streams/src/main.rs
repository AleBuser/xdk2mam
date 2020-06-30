use actix_web::{guard, web, App, HttpResponse, HttpServer};
use local::api::handlers;

use local::security::keystore::KeyManager;

use local::iota_channels_lite::channels_lite::channel_author::Channel;
use local::iota_channels_lite::channels_lite::Network;

use local::ChannelState;
use local::TagLists;

use std::sync::{Arc, Mutex};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    dotenv::dotenv().ok();

    let mut c = Channel::new(Network::Main, None);
    let (x, y) = c.open().unwrap();
    println!("Opened Streams channel at address:\n {}", x);
    let channel_state = Arc::new(Mutex::new(ChannelState {
        channel: c,
        channel_address: x,
        announcement_tag: y,
    }));

    println!("Started server at: 127.0.0.1:8081");

    let tag_store = Arc::new(Mutex::new(TagLists {
        signed_public: vec![],
        signed_masked: vec![],
        tagged: vec![],
    }));

    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(4096))
            .data(KeyManager::restore())
            .data(channel_state.clone())
            .data(tag_store.clone())
            .service(web::resource("/status").route(web::get().to(handlers::status)))
            .service(
                web::resource("/get_announcement").route(web::get().to(handlers::get_announcement)),
            )
            .service(
                web::resource("/sensor_data")
                    .route(web::post().to(handlers::sensor_data))
                    .guard(guard::fn_guard(|req| {
                        req.headers().contains_key("x-api-key")
                    }))
                    .to(|| HttpResponse::MethodNotAllowed()),
            )
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
