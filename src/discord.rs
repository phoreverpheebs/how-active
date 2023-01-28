use std::fmt;
use reqwest::{
    self,
    blocking::Client,
    header, 
};
use serde::{Deserialize, Serialize};

pub const AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.21 Chrome/94.0.4606.81 Electron/15.5.7 Safari/537.36";

macro_rules! make_object {
    (
        $name:ident,
        $func_name:ident,
        $empty:ident,
        $endpoint:literal,
        ($format:literal, $($value:ident),+),
        $($vis:vis $attribute:ident: $type:ty = $default:expr),+,
    ) => {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct $name {
            $($vis $attribute: $type),*
        }

        pub fn $func_name(id: &str, token: &str) -> reqwest::Result<$name> {
            Client::new().get(format!("https://discord.com/api/v9/{}/{}", $endpoint, id))
                .header(header::AUTHORIZATION, token)
                .header(header::USER_AGENT, AGENT)
                .send()?
                .error_for_status()?
                .json::<$name>()
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $format, $(self.$value),+)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    $($attribute: $default),*
                }
            }
        }

        impl $name {
            pub fn is_empty(&self) -> bool {
                self.$empty.is_empty()
            }
        }
    }
}

make_object! {
    User, get_user, id, "users", 
    ("{}#{} ({})", username, discriminator, id),
    pub id: String = String::new(),
    username: String = String::new(),
    discriminator: String = String::new(),
}

make_object! {
    Channel, get_channel, id, "channels",
    ("{} ({})", name, id),
    pub id: String = String::new(),
    name: String = String::new(),
    pub guild_id: String = String::new(),
}

make_object! {
    Guild, get_guild, id, "guilds",
    ("{} ({})", name, id),
    pub id: String = String::new(),
    name: String = String::new(),
}

/// struct that implements iterator to incrementally
/// grab messages
pub struct Messenger {
    user: String,
    guild: String,
    channel: String,
    pub offset: usize,
    pub total_results: u32,
    token: String,
    client: Client,
}

impl Messenger {
    pub fn new(token: String, user: String, guild: String, channel: Option<String>) -> Self {
        Messenger {
            token,
            user,
            guild,
            channel: channel.unwrap_or_else(|| String::new()),
            offset: 0,
            total_results: 0,
            client: Client::new(),
        }
    }
}

impl Iterator for Messenger {
    type Item = Vec<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(!self.guild.is_empty());
        assert!(!self.user.is_empty());
        if let Ok(resp) = self.client.get(format!("https://discord.com/api/v9/guilds/{}/messages/search?author_id={}{}{}",
                self.guild, self.user, 
                if !self.channel.is_empty() {
                    format!("&channel_id={}", self.channel)
                } else { String::from("") }, 
                if self.offset != 0 {
                    format!("&offset={}", self.offset)
                } else { String::from("") }))
            .header(header::AUTHORIZATION, &self.token)
            .header(header::USER_AGENT, AGENT)
            .send() {
                if let Ok(resp) = resp.error_for_status() {
                    match resp.json::<MessageSearch>() {
                        Ok(v) => {
                            self.total_results = v.total_results;
                            let messages = v.messages.into_iter().flatten().collect::<Vec<Message>>();
                            self.offset += messages.len();
                            Some(messages)
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageSearch {
    total_results: u32,
    messages: Vec<Vec<Message>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    id: String,
    author: User,
    pub timestamp: String,
    edited_timestamp: Option<String>,
    content: String,
}

impl fmt::Display for Message { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}#{}: {}", self.author.username, self.author.discriminator, self.content)
    }
}
