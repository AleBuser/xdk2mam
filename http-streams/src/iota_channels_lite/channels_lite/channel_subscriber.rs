//!
//! Channel Subscriber
//!
use super::Network;
use crate::iota_channels_lite::utils::{payload::json::Payload, random_seed};
use failure::Fallible;
use iota_lib_rs::prelude::iota_client;
use iota_streams::app::transport::tangle::client::SendTrytesOptions;
use iota_streams::app::transport::Transport;
use iota_streams::app_channels::{
    api::tangle::{Address, Subscriber},
    message,
};
use serde::de::DeserializeOwned;

///
/// Channel subscriber
///
pub struct Channel {
    subscriber: Subscriber,
    is_connected: bool,
    send_opt: SendTrytesOptions,
    client: iota_client::Client<'static>,
    announcement_link: Address,
    subscription_link: Address,
    channel_address: String,
}

impl Channel {
    ///
    /// Initialize the subscriber
    ///
    pub fn new(
        node: Network,
        channel_address: String,
        announcement_tag: String,
        seed_option: Option<String>,
    ) -> Channel {
        let seed = match seed_option {
            Some(seed) => seed,
            None => random_seed::new(),
        };
        let subscriber = Subscriber::new(&seed, true);

        Self {
            subscriber: subscriber,
            is_connected: false,
            send_opt: node.send_options(),
            client: iota_client::Client::new(node.as_string()),
            announcement_link: Address::from_str(&channel_address, &announcement_tag).unwrap(),
            subscription_link: Address::default(),
            channel_address: channel_address,
        }
    }

    ///
    /// Connect
    ///
    pub fn connect(&mut self) -> Fallible<String> {
        let message_list = self
            .client
            .recv_messages_with_options(&self.announcement_link, ())?;

        let mut found_valid_msg = false;

        for tx in message_list.iter() {
            let header = tx.parse_header()?;
            if header.check_content_type(message::announce::TYPE) {
                self.subscriber.unwrap_announcement(header.clone())?;
                found_valid_msg = true;
                break;
            }
        }
        if found_valid_msg {
            let subscribe_link = {
                let msg = self.subscriber.subscribe(&self.announcement_link)?;
                self.client.send_message_with_options(&msg, self.send_opt)?;
                msg.link.clone()
            };

            self.subscription_link = subscribe_link;
            self.is_connected = true;
        } else {
            println!("No valid announce message found");
        }
        Ok(self.subscription_link.msgid.to_string())
    }

    ///
    /// Disconnect
    ///
    pub fn disconnect(&mut self) -> Fallible<String> {
        let unsubscribe_link = {
            let msg = self.subscriber.unsubscribe(&self.subscription_link)?;
            self.client.send_message_with_options(&msg, self.send_opt)?;
            msg.link.msgid
        };
        Ok(unsubscribe_link.to_string())
    }

    ///
    /// Read signed packet
    ///
    pub fn read_signed<T>(
        &mut self,
        signed_packet_tag: String,
        change_key_tag: Option<String>,
    ) -> Fallible<Vec<(Option<T>, Option<T>)>>
    where
        T: DeserializeOwned,
    {
        if change_key_tag.is_some() {
            self.update_change_key(change_key_tag.unwrap())?;
        }

        let mut response: Vec<(Option<T>, Option<T>)> = Vec::new();

        if self.is_connected {
            let link = Address::from_str(&self.channel_address, &signed_packet_tag).unwrap();
            let message_list = self.client.recv_messages_with_options(&link, ())?;

            for tx in message_list.iter() {
                let header = tx.parse_header()?;
                if header.check_content_type(message::signed_packet::TYPE) {
                    match self.subscriber.unwrap_signed_packet(header.clone()) {
                        Ok((unwrapped_public, unwrapped_masked)) => {
                            response.push((
                                Payload::unwrap_data(&unwrapped_public)?,
                                Payload::unwrap_data(&unwrapped_masked)?,
                            ));
                        }
                        Err(e) => println!("Signed Packet Error: {}", e),
                    }
                }
            }
        } else {
            println!("Channel not connected");
        }

        Ok(response)
    }

    ///
    /// Read tagged packet
    ///
    pub fn read_tagged<T>(
        &mut self,
        tagged_packet_tag: String,
    ) -> Fallible<Vec<(Option<T>, Option<T>)>>
    where
        T: DeserializeOwned,
    {
        let mut response: Vec<(Option<T>, Option<T>)> = Vec::new();

        if self.is_connected {
            let link = Address::from_str(&self.channel_address, &tagged_packet_tag).unwrap();

            let message_list = self.client.recv_messages_with_options(&link, ())?;

            for tx in message_list.iter() {
                let header = tx.parse_header()?;
                if header.check_content_type(message::tagged_packet::TYPE) {
                    match self.subscriber.unwrap_tagged_packet(header.clone()) {
                        Ok((unwrapped_public, unwrapped_masked)) => {
                            response.push((
                                Payload::unwrap_data(&unwrapped_public)?,
                                Payload::unwrap_data(&unwrapped_masked)?,
                            ));
                        }
                        Err(e) => println!("Tagged Packet Error: {}", e),
                    }
                }
            }
        } else {
            println!("Channel not connected");
        }

        Ok(response)
    }

    ///
    /// Update keyload
    ///
    pub fn update_keyload(&mut self, keyload_tag: String) -> Fallible<()> {
        let keyload_link = Address::from_str(&self.channel_address, &keyload_tag).unwrap();

        if self.is_connected {
            let message_list = self.client.recv_messages_with_options(&keyload_link, ())?;

            for tx in message_list.iter() {
                let header = tx.parse_header()?;
                if header.check_content_type(message::keyload::TYPE) {
                    match self.subscriber.unwrap_keyload(header.clone()) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => println!("Keyload Packet Error: {}", e),
                    }
                } else if header.check_content_type(message::change_key::TYPE) {
                    match self.subscriber.unwrap_change_key(header.clone()) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => println!("Change Key Packet Error: {}", e),
                    }
                } else {
                    println!(
                        "Expected a keyload message, found {}",
                        header.content_type()
                    );
                }
            }
        }

        Ok(())
    }

    ///
    /// Update the change key
    ///
    pub fn update_change_key(&mut self, change_key_tag: String) -> Fallible<()> {
        let keyload_link = Address::from_str(&self.channel_address, &change_key_tag).unwrap();

        if self.is_connected {
            let message_list = self.client.recv_messages_with_options(&keyload_link, ())?;
            for tx in message_list.iter() {
                let header = tx.parse_header()?;
                if header.check_content_type(message::change_key::TYPE) {
                    match self.subscriber.unwrap_change_key(header.clone()) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => println!("Keyload Packet Error: {}", e),
                    }
                } else {
                    println!(
                        "Expected a keyload message, found {}",
                        header.content_type()
                    );
                }
            }
        } else {
            println!("Channel not connected");
        }

        Ok(())
    }
}
