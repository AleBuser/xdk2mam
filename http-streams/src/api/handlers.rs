use crate::is_valid;
use crate::security::keystore::KeyManager;
use crate::types::sensor_data::SensorData;
use crate::AnnouncementInfo;
use crate::ChannelState;
use crate::SignedTags;
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
    Ok(HttpResponse::Ok().json(AnnouncementInfo {
        channel_address: channel_address,
        announcement_tag: announcement_tag,
    }))
}

pub async fn get_tags(tag_lists: web::Data<Arc<Mutex<TagLists>>>) -> Result<HttpResponse, Error> {
    println!(
        "GET /tag_lists -- {:?} -- authorized request",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let tag_lists = tag_lists.lock().unwrap();
    Ok(HttpResponse::Ok().json(tag_lists.clone()))
}

pub async fn sensor_data_public(
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
            "POST /sensor_data_public -- {:?} -- authorized request",
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
                                let response = SignedTags {
                                    signed_message_tag: public_message_tag.signed_message_tag,
                                    change_key_tag: public_message_tag.change_key_tag,
                                };
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
        Ok(HttpResponse::Unauthorized().json("Unauthorized"))
    }
}

pub async fn sensor_data_masked(
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
            "POST /sensor_data_masked -- {:?} -- authorized request",
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

                        if channel.channel.can_send_masked() {
                            match channel.channel.write_signed(
                                true,
                                PayloadBuilder::new().masked(&data).unwrap().build(),
                            ) {
                                Ok(masked_message_tag) => {
                                    let response = SignedTags {
                                        signed_message_tag: masked_message_tag.signed_message_tag,
                                        change_key_tag: masked_message_tag.change_key_tag,
                                    };
                                    list.lock()
                                        .expect("lock list data")
                                        .signed_masked
                                        .push(response.clone());
                                    println!("Streams response: {:?}", response.clone());

                                    Ok(HttpResponse::Ok()
                                        .json(format!("Data Successfully sent to Tangle!")))
                                }
                                Err(_e) => Ok(HttpResponse::Ok().json(format!("ERROR"))),
                            }
                        } else {
                            Ok(HttpResponse::Ok().json(format!("NO SUBSCRIBERS")))
                        }
                    }
                    Err(e) => Ok(HttpResponse::Ok().json(format!("ERROR"))),
                }
            }
            None => Ok(HttpResponse::Ok().json(format!("No thing!"))),
        }
    } else {
        Ok(HttpResponse::Unauthorized().json("Unauthorized"))
    }
}
