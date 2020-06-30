pub mod api;
pub mod iota_channels_lite;
pub mod security;
pub mod types;

use iota_channels_lite::channels_lite::channel_author::Channel;

pub struct ChannelState {
    pub channel: Channel,
    pub channel_address: String,
    pub announcement_tag: String,
}

use std::sync::Mutex;
pub struct TagLists {
    pub signed_public: Vec<String>,
    pub signed_masked: Vec<String>,
    pub tagged: Vec<String>,
}
