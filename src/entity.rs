use std::convert::TryFrom;

use crate::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct RawLatest {
    pub version: Version,
    pub players: Players,
    pub description: RawDescription,
    pub favicon: Option<String>,
    #[serde(rename = "modinfo")]
    pub mod_info: Option<ModInfo>,
    #[serde(rename = "forgeData")]
    pub forge_data: Option<ForgeData>,
}

#[derive(Debug)]
/// A ping response returned from server.
pub struct Response {
    /// The version name of the server.
    pub version: String,
    /// The protocol number of the server.
    /// See also [the minecraft protocol wiki](https://wiki.vg/Protocol_version_numbers) for the actual values.
    pub protocol: i32,
    /// The maximum number of the connected players.
    pub max_players: usize,
    /// The number of the players currently connected.
    pub online_players: usize,
    /// The sample of the connected players.
    /// Note that it can be `None` even if some players are connected.
    pub sample: Option<Vec<Player>>,
    /// The description (aka MOTD) of the server.
    /// See also [the minecraft protocol wiki](https://wiki.vg/Chat#Current_system_.28JSON_Chat.29) for the [`Chat`](Chat) format.
    pub description: Chat,
    /// The favicon of the server in PNG format.
    pub favicon: Option<Vec<u8>>,
    /// The mod information object used in FML protocol (version 1.7 - 1.12).
    /// See also [the minecraft protocol wiki](https://wiki.vg/Minecraft_Forge_Handshake#FML_protocol_.281.7_-_1.12.29)
    /// for the [`ModInfo`](ModInfo) format.
    pub mod_info: Option<ModInfo>,
    /// The forge information object used in FML2 protocol (version 1.13 - current).
    /// See also [the minecraft protocol wiki](https://wiki.vg/Minecraft_Forge_Handshake#FML2_protocol_.281.13_-_Current.29)
    /// for the [`ForgeData`](ForgeData) format.
    pub forge_data: Option<ForgeData>,
}

impl TryFrom<RawLatest> for Response {
    type Error = Error;

    fn try_from(raw: RawLatest) -> Result<Self, Self::Error> {
        let favicon = if let Some(favicon) = raw.favicon {
            // normal server favicon should start with "data:image/png;base64,"
            Some(base64::decode(&favicon[22..]).map_err(|_| Error::UnsupportedProtocol)?)
        } else {
            None
        };
        Ok(Self {
            version: raw.version.name,
            protocol: raw.version.protocol,
            max_players: raw.players.max,
            online_players: raw.players.online,
            sample: raw.players.sample,
            description: raw.description.into(),
            favicon,
            mod_info: raw.mod_info,
            forge_data: raw.forge_data,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Players {
    pub max: usize,
    pub online: usize,
    pub sample: Option<Vec<Player>>,
}

#[derive(Debug, Deserialize)]
/// The sample players' information.
pub struct Player {
    /// The name of the player.
    pub name: String,
    /// The uuid of the player.
    /// Normally used to identify a player.
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawDescription {
    Raw(String),
    Chat(Chat),
}

#[derive(Debug, Deserialize)]
/// The mod information object used in FML protocol (version 1.7 - 1.12).
pub struct ModInfo {
    #[serde(rename = "type")]
    /// The field `type` of `modinfo`. It should be FML if forge is installed.
    pub mod_type: String,
    #[serde(rename = "modList")]
    /// The list of the mod installed on the server.
    /// See also [`ModInfoItem`](ModInfoItem)
    pub mod_list: Vec<ModInfoItem>,
}

#[derive(Debug, Deserialize)]
/// The information of an installed mod.
pub struct ModInfoItem {
    #[serde(rename = "modid")]
    /// The id of the mod.
    pub mod_id: String,
    /// The version of the mod.
    pub version: String,
}

#[derive(Debug, Deserialize)]
/// The forge information object used in FML2 protocol (version 1.13 - current).
pub struct ForgeData {
    /// The list of the channels used by the mods.
    /// See [the minecraft protocol wiki](https://wiki.vg/Plugin_channels) for more information.
    pub channels: Vec<ForgeChannel>,
    /// The list of the mods installed on the server.
    pub mods: Vec<ForgeMod>,
    #[serde(rename = "fmlNetworkVersion")]
    pub fml_network_version: String,
}

#[derive(Debug, Deserialize)]
/// The information of the channels used by the mods.
///
/// See [the minecraft protocol wiki](https://wiki.vg/Plugin_channels) for more information.
/// Unfortunately, the exact semantics of its field is currently not found.
/// We do not guarantee the document is right, and you should re-check the values you've received.
pub struct ForgeChannel {
    /// The namespaced key of the channel
    pub res: String,
    /// The version of the channel
    pub version: String,
    /// `true` if it is required
    pub required: bool,
}

#[derive(Debug, Deserialize)]
/// The information of an installed mod.
pub struct ForgeMod {
    #[serde(rename = "modId")]
    /// The id of the mod.
    pub mod_id: String,
    #[serde(rename = "modmarker")]
    /// The version of the mod.
    pub mod_marker: String,
}

#[derive(Debug, Deserialize, Default)]
/// The chat component used in the server description.
///
/// See also [the minecraft protocol wiki](https://wiki.vg/Chat#Current_system_.28JSON_Chat.29).
pub struct Chat {
    /// The text which this `Chat` object holds.
    pub text: String,
    #[serde(default)]
    /// `true` if the text *and* the extras should be __bold__.
    pub bold: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should be *italic*.
    pub italic: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should be <u>underlined</u>.
    pub underlined: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should have a <strike>strikethrough</strike>.
    pub strikethrough: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should look obfuscated.
    pub obfuscated: bool,
    /// The color which the text and the extras should have.
    /// `None` to use default color.
    pub color: Option<String>,
    #[serde(default)]
    /// The extra text components following this text.
    /// They should inherit this chat component's properties (bold, italic, etc.) but can also override the properties.
    pub extra: Vec<Chat>,
}

impl From<RawDescription> for Chat {
    fn from(description: RawDescription) -> Self {
        match description {
            RawDescription::Chat(chat) => chat,
            RawDescription::Raw(text) => Chat {
                text,
                ..Default::default()
            },
        }
    }
}
