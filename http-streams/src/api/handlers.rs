use crate::security::keystore::calculate_hash;
use crate::security::keystore::KeyManager;
use crate::types::sensor_data::SensorData;
use crate::ChannelState;
use crate::TagLists;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::iota_channels_lite::utils::payload::json::PayloadBuilder;

pub async fn status() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(format!("OK")))
}

pub async fn get_announcement(
    channel: web::Data<Arc<Mutex<ChannelState>>>,
) -> Result<HttpResponse, Error> {
    println!(
        "GET /get_announcement -- {:?} -- authorized request",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let channel = channel.lock().unwrap();
    let channel_address = channel.channel_address.clone();
    let announcement_tag = channel.announcement_tag.clone();
    Ok(HttpResponse::Ok().json(format!(
        "{{ channel_address:'{}', announcement_tag:'{}' }}",
        channel_address, announcement_tag
    )))
}

pub async fn sensor_data(
    data: Option<String>,
    req: HttpRequest,
    store: web::Data<KeyManager>,
    channel: web::Data<Arc<Mutex<ChannelState>>>,
    list: web::Data<Arc<Mutex<TagLists>>>,
) -> Result<HttpResponse, Error> {
    let hash = store.keystore.api_key_author.clone();
    if is_valid(
        req.headers().get("x-api-key").unwrap().to_str().unwrap(),
        hash.clone(),
    ) {
        println!(
            "POST /sensor_data -- {:?} -- authorized request",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        match data {
            Some(data) => {
                let json_data: serde_json::Result<SensorData> = serde_json::from_str(&data);
                match json_data {
                    Ok(data) => {
                        let mut channel = channel.lock().unwrap();
                        match channel.channel.write_signed(
                            false,
                            PayloadBuilder::new().public(&data).unwrap().build(),
                        ) {
                            Ok(public_message_tag) => {
                                let response = serde_json::to_string(&public_message_tag).unwrap();
                                list.lock()
                                    .expect("lock list data")
                                    .signed_public
                                    .push(response.clone());
                                println!("Streams response: {:?}", response.clone());

                                Ok(HttpResponse::Ok()
                                    .json(format!("Data Successfully sent to Tangle!")))
                            }
                            Err(_e) => Ok(HttpResponse::Ok().json(format!("ERROR"))),
                        }
                    }
                    Err(e) => Ok(HttpResponse::Ok().json(format!("ERROR"))),
                }
            }
            None => Ok(HttpResponse::Ok().json(format!("No thing!"))),
        }
    } else {
        Ok(HttpResponse::Ok().json(format!("No thing!")))
    }
}

fn is_valid(key: &str, hash: String) -> bool {
    calculate_hash(key.to_string()) == hash
}

/*

pub async fn get_last_trades(amount: web::Path<i32>) -> Result<HttpResponse, Error> {
    Ok(web::block(move || last_trades(amount.into_inner(), db))
        .await
        .map(|t| HttpResponse::Ok().json(t))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

pub async fn get_trader_profile(trader_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(
        web::block(move || trader_profile(&trader_id.into_inner(), db))
            .await
            .map(|user| HttpResponse::Ok().json(user))
            .map_err(|_| HttpResponse::InternalServerError())?,
    )
}
*/
