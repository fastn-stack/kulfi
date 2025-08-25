use eyre::Context;
use eyre::eyre;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_malai_conf")]
    malai: MalaiConf,
    http: Option<HttpServices>,
    tcp: Option<TcpServices>,
}

#[derive(Deserialize, Debug)]
struct MalaiConf {
    log: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HttpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, HttpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpServiceConf {
    identity: Option<String>, // Leave None to read from env, .malai.secret-key file or .malai.id52 file and system keyring
    port: u16,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_bridge")]
    bridge: String,
}

#[derive(Deserialize, Debug)]
struct TcpServices {
    #[allow(dead_code)]
    #[serde(flatten)]
    services: HashMap<String, TcpServiceConf>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TcpServiceConf {
    identity: Option<String>, // Leave None to read from env, .malai.secret-key file or .malai.id52 file and system keyring
    port: u16,
    public: bool,
    active: bool,
    #[serde(default = "default_host")]
    host: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_bridge() -> String {
    match env::var("MALAI_HTTP_BRIDGE") {
        Ok(value) => value,
        Err(_) => "kulfi.site".to_string(),
    }
}

fn default_malai_conf() -> MalaiConf {
    MalaiConf { log: None }
}

fn parse_config(path: &Path) -> eyre::Result<Config> {
    let conf_str = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;

    let conf = toml::from_str(&conf_str).context("Failed to parse config file")?;
    Ok(conf)
}

fn set_up_logging(conf: &Config) -> eyre::Result<Option<WorkerGuard>> {
    match &conf.malai.log {
        Some(log_dir) => {
            let log_dir = Path::new(&log_dir);
            let file_appender = rolling::daily(
                log_dir.parent().unwrap_or(Path::new("./")),
                log_dir
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("malai.log")),
            );
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            // tracing_subscriber::fmt()
            //     .with_writer(non_blocking)
            //     .with_ansi(false)
            //     .init();
            let subscriber = fmt::Subscriber::builder().finish().with(
                fmt::Layer::new()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true),
            );

            tracing::subscriber::set_global_default(subscriber)
                .expect("Failed to set tracing subscriber");
            return Ok(Some(guard));
        }
        None => {
            tracing_subscriber::fmt::init();
        }
    }
    Ok(None)
}

async fn load_identity(identity: &Option<String>) -> eyre::Result<(String, kulfi_id52::SecretKey)> {
    match identity {
        Some(id52) => match kulfi_utils::secret::handle_identity(id52.to_string()) {
            Ok(v) => Ok(v),
            Err(_e) => Err(eyre!(
                "Failed to load identity {} from system keyring.",
                id52
            )),
        },
        None => match kulfi_utils::read_or_create_key().await {
            Ok(v) => Ok(v),
            Err(e) => {
                malai::identity_read_err_msg(e);
                Err(eyre!("Failed to load/create identity from/to file."))
            }
        },
    }
}

async fn set_up_http_services(conf: &Config, graceful: kulfi_utils::Graceful) {
    if let Some(http_conf) = &conf.http {
        for (name, service_conf) in &http_conf.services {
            info!("Starting HTTP services: {}", name);
            // Check
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }
            let host = service_conf.host.clone();
            let port = service_conf.port;
            let bridge = service_conf.bridge.clone();
            let graceful_clone = graceful.clone();

            let (id52, secret_key) = match load_identity(&service_conf.identity).await {
                Ok(v) => v,
                Err(e) => {
                    // The error message has been printed by tracing::error!
                    error!(
                        "Failed to load identity for service {}: {} Skipping.",
                        name, e
                    );
                    continue;
                }
            };

            graceful.spawn(async move {
                malai::expose_http(host, port, bridge, id52, secret_key, graceful_clone).await
            });
        }
    }
}

async fn set_up_tcp_services(conf: &Config, graceful: kulfi_utils::Graceful) {
    if let Some(tcp_conf) = &conf.tcp {
        for (name, service_conf) in &tcp_conf.services {
            info!("Starting TCP services: {}", name);
            // Check
            if !service_conf.active {
                continue;
            }
            if !service_conf.public {
                tracing::warn!(
                    "You have to set public to true for service {}. Skipping.",
                    name
                );
                continue;
            }
            let host = service_conf.host.clone();
            let port = service_conf.port;
            let graceful_clone = graceful.clone();

            let (id52, secret_key) = match load_identity(&service_conf.identity).await {
                Ok(v) => v,
                Err(e) => {
                    // The error message has been printed by tracing::error!
                    error!(
                        "Failed to load identity for service {}: {} Skipping.",
                        name, e
                    );
                    continue;
                }
            };

            graceful.spawn(async move {
                malai::expose_tcp(host, port, id52, secret_key, graceful_clone).await
            });
        }
    }
}

pub async fn run(conf_path: &Path, graceful: kulfi_utils::Graceful) -> Option<WorkerGuard> {
    let conf = match parse_config(conf_path) {
        Ok(conf) => conf,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return None;
        }
    };

    let guard = match set_up_logging(&conf) {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to set up logging: {}. Skipping.", e);
            None
        }
    };

    set_up_http_services(&conf, graceful.clone()).await;
    set_up_tcp_services(&conf, graceful.clone()).await;
    guard
}

#[test]
fn parse_config_test() {
    let conf = parse_config(Path::new("tests/http_example_conf.toml")).unwrap();
    println!("{:?}", conf);
    assert!(conf.http.is_some());
    let http = conf.http.as_ref().expect("HTTP services should be present");
    assert!(http.services.get("service1").is_some());
    assert!(http.services.get("service2").is_some());

    assert!(conf.tcp.is_some());
    let tcp = conf.tcp.as_ref().expect("TCP services should be present");
    assert!(tcp.services.get("service3").is_some());
}
