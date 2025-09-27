use crate::error::{CCProxyError, CCProxyResult};
use crate::network::bedrock::BedrockMotd;
use figment::Figment;
use figment::providers::{Env, Format, Yaml};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};

pub const CCPROXY_ENV_PREFIX: &str = "CCPROXY__";

pub fn ccproxy_env(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(format!("{CCPROXY_ENV_PREFIX}{key}"))
}

/// Try to get the data path from environment variable. If it is not available,
/// get current + data/ directory.
///
/// The reason why this environment variable is not imported from [`CCProxyConfig`]
/// is that the values must be retrieved first before loading the config file.
pub static DATA_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| match ccproxy_env("DATA_PATH").ok().map(PathBuf::from) {
        Some(path) => {
            std::fs::create_dir_all(&path).expect("Cannot create the data directory");
            path.is_dir()
                .then_some(path)
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "The data directory is not found.",
                ))
                .unwrap()
        }
        None => std::env::current_dir()
            .expect("Cannot get or access current directory for data path")
            .join("data/"),
    });

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct CCProxyConfig {
    #[serde(default)]
    pub log: LogConfig,

    pub proxy: ProxyConfig,

    pub upstream: UpstreamConfig,
}

impl CCProxyConfig {
    pub fn init() -> CCProxyResult<Self> {
        // Create the config path
        let config_path = DATA_PATH.join("config");
        std::fs::create_dir_all(&config_path)?;

        let config = config_path.join("config.yaml");

        // Init the default config if it doesn't exist.
        if !config.exists() {
            std::fs::write(
                &config,
                serde_yaml::to_string(&CCProxyConfig::default()).unwrap(),
            )?;
        }

        // Load the config
        Ok(Figment::new()
            .merge(Env::prefixed(CCPROXY_ENV_PREFIX).split("__"))
            .merge(Yaml::file(config))
            .extract()
            .map_err(Box::new)?)
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct LogConfig {
    #[serde(default)]
    pub stdout: LogBaseConfig,

    #[serde(default)]
    pub file: LogBaseConfig,
}

impl LogConfig {
    pub fn tracing_subscriber(&self) -> CCProxyResult<(impl tracing::Subscriber, WorkerGuard)> {
        let stdout_filter = EnvFilter::builder().parse(self.stdout.filter.clone())?;
        let file_filter = EnvFilter::builder().parse(self.file.filter.clone())?;

        // stdout
        let stdout_log = match self.stdout.format {
            LogFormat::Plain => tracing_subscriber::fmt::layer()
                .with_filter(stdout_filter)
                .boxed(),
            LogFormat::Json => tracing_subscriber::fmt::layer()
                .json()
                .with_filter(stdout_filter)
                .boxed(),
        };

        // file
        let file_appender = RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_suffix("log")
            .build(DATA_PATH.join("logs"))?;
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
        let file_log = match self.file.format {
            LogFormat::Plain => tracing_subscriber::fmt::layer()
                // No colors in text file.
                // TODO: Find why this not work in other crates.
                .with_ansi(false)
                .with_writer(file_writer)
                .with_filter(file_filter)
                .boxed(),
            LogFormat::Json => tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_filter(file_filter)
                .boxed(),
        };

        let subscriber = tracing_subscriber::registry()
            .with(stdout_log)
            .with(file_log);

        Ok((subscriber, guard))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LogBaseConfig {
    pub filter: String,

    #[serde(default)]
    pub format: LogFormat,
}

impl Default for LogBaseConfig {
    fn default() -> Self {
        Self {
            filter: "info".to_owned(),
            format: Default::default(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Plain,

    Json,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub address: SocketAddr,

    pub fallback_motd: BedrockMotd,

    pub fallback_query: ProxyQueryConfig,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:19132".parse().unwrap(),
            fallback_motd: Default::default(),
            fallback_query: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProxyQueryConfig {
    pub motd: String,

    pub game_type: String,

    pub map: String,

    pub num_players: u64,

    pub max_players: u64,

    pub host_port: u16,

    pub host_ip: IpAddr,

    pub version: String,

    #[serde(default)]
    pub plugins: Option<String>,

    #[serde(default)]
    pub players: Vec<String>,
}

impl Default for ProxyQueryConfig {
    fn default() -> Self {
        Self {
            motd: "CCProxy".to_owned(),
            game_type: "SMP".to_owned(),
            map: "CCProxy".to_owned(),
            num_players: 0,
            max_players: 100,
            host_port: 19132,
            host_ip: "0.0.0.0".parse().unwrap(),
            version: "1.21.101".to_owned(),
            plugins: Default::default(),
            players: Default::default(),
        }
    }
}

impl ProxyQueryConfig {
    pub fn from_kv_and_players(
        k_v_section: HashMap<String, String>,
        players: Vec<String>,
    ) -> CCProxyResult<Self> {
        Ok(Self {
            motd: k_v_section
                .get("hostname")
                .ok_or(CCProxyError::QueryInvalid)?
                .to_owned(),
            game_type: k_v_section
                .get("gametype")
                .ok_or(CCProxyError::QueryInvalid)?
                .to_owned(),
            map: k_v_section
                .get("map")
                .ok_or(CCProxyError::QueryInvalid)?
                .to_owned(),
            num_players: k_v_section
                .get("numplayers")
                .ok_or(CCProxyError::QueryInvalid)?
                .parse()
                .map_err(|_| CCProxyError::QueryInvalid)?,
            max_players: k_v_section
                .get("maxplayers")
                .ok_or(CCProxyError::QueryInvalid)?
                .parse()
                .map_err(|_| CCProxyError::QueryInvalid)?,
            host_port: k_v_section
                .get("hostport")
                .ok_or(CCProxyError::QueryInvalid)?
                .parse()
                .map_err(|_| CCProxyError::QueryInvalid)?,
            host_ip: k_v_section
                .get("hostip")
                .ok_or(CCProxyError::QueryInvalid)?
                .to_owned()
                .parse()
                .map_err(|_| CCProxyError::QueryInvalid)?,
            version: k_v_section
                .get("version")
                .ok_or(CCProxyError::QueryInvalid)?
                .to_owned(),
            plugins: k_v_section.get("plugins").cloned(),
            players,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UpstreamConfig {
    pub address: SocketAddr,

    pub query_address: Option<SocketAddr>,

    #[serde(default)]
    pub proxy_protocol: bool,
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:19133".parse().unwrap(),
            query_address: Some("127.0.0.1:19133".parse().unwrap()),
            proxy_protocol: false,
        }
    }
}
