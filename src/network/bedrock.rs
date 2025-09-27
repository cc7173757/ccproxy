use crate::error::{CCProxyError, CCProxyResult};
use serde::{Deserialize, Serialize};

fn default_guid() -> u64 {
    0
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BedrockMotd {
    pub edition: BedrockEdition,

    pub server_name: String,

    pub protocol_version: i32,

    pub version: String,

    pub num_players: i32,

    pub max_players: i32,

    #[serde(default = "default_guid")]
    pub guid: u64,

    pub server_sub_name: String,

    pub gametype: BedrockGametype,

    pub nintendo_limited: bool,

    pub ipv4_port: Option<u16>,

    pub ipv6_port: Option<u16>,
}

impl Default for BedrockMotd {
    fn default() -> Self {
        Self {
            edition: Default::default(),
            server_name: "CCProxy".to_owned(),
            protocol_version: 827,
            version: "1.21.101".to_owned(),
            num_players: 0,
            max_players: 100,
            guid: default_guid(),
            server_sub_name: "CCProxy".to_owned(),
            gametype: Default::default(),
            nintendo_limited: false,
            ipv4_port: Some(19132),
            ipv6_port: None,
        }
    }
}

impl BedrockMotd {
    /// Encode the [`BedrockMotd`] to the [`String`].
    ///
    /// You can pass optional `guid` to override the GUID during encoding.
    pub fn encode(&self, guid: Option<u64>) -> String {
        let mut motd = vec![
            self.edition.encode(),
            self.server_name.clone(),
            self.protocol_version.to_string(),
            self.version.clone(),
            self.num_players.to_string(),
            self.max_players.to_string(),
            guid.map(|g| g.to_string()).unwrap_or(self.guid.to_string()),
            self.server_sub_name.clone(),
            self.gametype.encode(),
            if self.nintendo_limited {
                "0".to_owned()
            } else {
                "1".to_owned()
            },
        ];

        match (self.ipv4_port, self.ipv6_port) {
            (Some(ipv4_port), Some(ipv6_port)) => {
                motd.append(&mut vec![ipv4_port.to_string(), ipv6_port.to_string()])
            }
            (Some(ipv4_port), None) => motd.push(ipv4_port.to_string()),
            _ => (),
        };

        format!("{};", motd.join(";"))
    }

    /// Decode the [`String`] to the [`BedrockMotd`].
    ///
    /// You can pass optional parameters to override fields during decode.
    pub fn decode(
        buf: String,
        guid: Option<u64>,
        ipv4_port: Option<u16>,
        ipv6_port: Option<u16>,
    ) -> CCProxyResult<Self> {
        let buf = buf.split(";").map(|b| b.to_owned()).collect::<Vec<_>>();

        if !(11..=14).contains(&buf.len()) {
            return Err(CCProxyError::MotdInvalid);
        }

        let mut motd = Self {
            edition: BedrockEdition::decode(&buf[0])?,
            server_name: buf[1].clone(),
            protocol_version: buf[2].parse().map_err(|_| CCProxyError::MotdInvalid)?,
            version: buf[3].clone(),
            num_players: buf[4].parse().map_err(|_| CCProxyError::MotdInvalid)?,
            max_players: buf[5].parse().map_err(|_| CCProxyError::MotdInvalid)?,
            guid: guid.unwrap_or(buf[6].parse().map_err(|_| CCProxyError::MotdInvalid)?),
            server_sub_name: buf[7].clone(),
            gametype: BedrockGametype::decode(&buf[8])?,
            nintendo_limited: buf[9] == "0",
            ipv4_port: None,
            ipv6_port: None,
        };

        match (ipv4_port, ipv6_port) {
            (Some(ipv4_port), Some(ipv6_port)) => {
                motd.ipv4_port = Some(ipv4_port);
                motd.ipv6_port = Some(ipv6_port);
            }
            (Some(ipv4_port), None) => {
                motd.ipv4_port = Some(ipv4_port);
            }
            _ => (),
        };

        Ok(motd)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum BedrockEdition {
    #[default]
    MCPE,

    MCEE,
}

impl BedrockEdition {
    pub fn encode(&self) -> String {
        use BedrockEdition::*;
        match self {
            MCPE => "MCPE".to_owned(),
            MCEE => "MCEE".to_owned(),
        }
    }

    pub fn decode(buf: &str) -> CCProxyResult<Self> {
        use BedrockEdition::*;
        Ok(match buf {
            "MCPE" => MCPE,
            "MCEE" => MCEE,
            _ => Err(CCProxyError::MotdInvalid)?,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum BedrockGametype {
    #[default]
    Survival,

    Creative,
}

impl BedrockGametype {
    pub fn encode(&self) -> String {
        use BedrockGametype::*;
        match self {
            Survival => "Survival".to_owned(),
            Creative => "Creative".to_owned(),
        }
    }

    pub fn decode(buf: &str) -> CCProxyResult<Self> {
        use BedrockGametype::*;
        Ok(match buf {
            "Survival" => Survival,
            "Creative" => Creative,
            _ => Err(CCProxyError::MotdInvalid)?,
        })
    }
}
