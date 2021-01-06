use std::convert::TryFrom;

use crate::Error;
use serde::Deserialize;

#[derive(Debug)]
pub enum Response {
    Latest(Latest),
    Legacy(Legacy),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawLatest {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    pub favicon: Option<String>,
    #[serde(rename = "modinfo")]
    pub mod_info: Option<ModInfo>,
    #[serde(rename = "forgeData")]
    pub forge_data: Option<ForgeData>,
}

#[derive(Debug)]
pub struct Latest {
    pub version: String,
    pub protocol: i32,
    pub max_players: usize,
    pub online_players: usize,
    pub sample: Option<Vec<Player>>,
    pub description: Description,
    pub favicon: Option<Vec<u8>>,
    pub mod_info: Option<ModInfo>,
    pub forge_data: Option<ForgeData>,
}

impl TryFrom<RawLatest> for Latest {
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
            description: raw.description,
            favicon,
            mod_info: raw.mod_info,
            forge_data: raw.forge_data,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Deserialize)]
pub struct Players {
    pub max: usize,
    pub online: usize,
    pub sample: Option<Vec<Player>>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Description {
    Raw(String),
    // TODO: properly handle ChatComponent
    Chat { text: String },
}

#[derive(Debug, Deserialize)]
pub struct ModInfo {
    #[serde(rename = "type")]
    pub mod_type: String,
    #[serde(rename = "modList")]
    pub mod_list: Vec<ModInfoItem>,
}

#[derive(Debug, Deserialize)]
pub struct ModInfoItem {
    #[serde(rename = "modid")]
    pub mod_id: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgeData {
    pub channels: Vec<ForgeChannel>,
    pub mods: Vec<ForgeMod>,
    #[serde(rename = "fmlNetworkVersion")]
    pub fml_network_version: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgeChannel {
    pub res: String,
    pub version: String,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct ForgeMod {
    #[serde(rename = "modId")]
    pub mod_id: String,
    #[serde(rename = "modmarker")]
    pub mod_marker: String,
}

#[derive(Debug)]
pub struct Legacy {
    pub protocol: i32,
    pub version: String,
    pub motd: String,
    pub players: usize,
    pub max_players: usize,
}
