// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), feature = "ui"),
    windows_subsystem = "windows"
)]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="malai=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let graceful = kulfi_utils::Graceful::default();

    // TODO: each subcommand should handle their error and return ()
    match cli.command {
        Some(Command::Http {
            port,
            host,
            bridge,
            public,
            secret_key_path,
            // secure,
            // what_to_do,
        }) => {
            if !malai::public_check(
                public,
                "HTTP service",
                &format!("malai http {port} --public"),
            ) {
                std::process::exit(1);
            }

            let secret_key_path = match resolve_secret_key_path(secret_key_path) {
                Ok(v) => v,
                Err(_) => std::process::exit(1), // error msg is printed
            };

            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing HTTP service on kulfi.");
            let g = graceful.clone();
            graceful.spawn(async move {
                malai::expose_http(host, port, bridge, &secret_key_path, g).await
            });
        }
        Some(Command::HttpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting HTTP bridge.");
            let g = graceful.clone();
            graceful
                .spawn(async move { malai::http_bridge(port, proxy_target, g, |_| Ok(())).await });
        }
        Some(Command::Tcp {
            port,
            host,
            public,
            secret_key_path,
        }) => {
            if !malai::public_check(
                public,
                "HTTP service",
                &format!("malai http {port} --public"),
            ) {
                std::process::exit(1);
            }

            let secret_key_path = match resolve_secret_key_path(secret_key_path) {
                Ok(v) => v,
                Err(_) => std::process::exit(1), // error msg is printed
            };

            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing TCP service on kulfi.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::expose_tcp(host, port, &secret_key_path, g).await });
        }
        Some(Command::TcpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting TCP bridge.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::tcp_bridge(port, proxy_target, g).await });
        }
        Some(Command::Browse { url }) => {
            tracing::info!(url, verbose = ?cli.verbose, "Opening browser.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::browse(url, g).await });
        }
        Some(Command::Folder {
            path,
            bridge,
            public,
            secret_key_path,
        }) => {
            if !malai::public_check(public, "folder", &format!("malai folder --public {path}")) {
                std::process::exit(1);
            }

            let secret_key_path = match resolve_secret_key_path(secret_key_path) {
                Ok(v) => v,
                Err(_) => std::process::exit(1), // error msg is printed
            };

            tracing::info!(path, verbose = ?cli.verbose, "Exposing folder to kulfi network.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::folder(path, bridge, secret_key_path, g).await });
        }
        Some(Command::Keygen { path }) => {
            let secret_key_path = match path {
                Some(v) => std::path::PathBuf::from(v),
                None => directories::BaseDirs::new()
                    .expect("Failed to get valid HOME dir")
                    .data_dir()
                    .join("malai")
                    .join("default.key"),
            };

            if secret_key_path.exists() {
                eprintln!(
                    "Path {secret_key_path:?} already exists. Please provide a different path or delete the existing file."
                );
                std::process::exit(1);
            }

            if secret_key_path.is_dir() {
                eprintln!("Secret key path is a directory. Please provide a valid file path.");
                std::process::exit(1);
            }

            tracing::info!(path = ?secret_key_path, verbose = ?cli.verbose, "Generating new secret key.");

            std::fs::create_dir_all(
                secret_key_path
                    .parent()
                    .expect("Failed to get parent directory"),
            )?;

            match kulfi_utils::generate_secret_key(&secret_key_path) {
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to generate secret key");
                    std::process::exit(1);
                }
                Ok(_) => {
                    tracing::info!(path = ?secret_key_path, "Secret key generated successfully.");
                    println!("Secret key generated successfully at {secret_key_path:?}");
                    std::process::exit(0);
                }
            }
        }
        #[cfg(feature = "ui")]
        None => {
            tracing::info!(verbose = ?cli.verbose, "Starting UI.");
            let _ = malai::ui();
        }
        #[cfg(not(feature = "ui"))]
        None => {
            use clap::CommandFactory;

            Cli::command().print_help()?;
            return Ok(());
        }
    };

    graceful.shutdown().await
}

fn resolve_secret_key_path(path_arg: Option<String>) -> Result<std::path::PathBuf, ()> {
    let secret_key_path = match path_arg {
        Some(v) => std::path::PathBuf::from(v),
        None => directories::BaseDirs::new()
            .expect("Failed to get valid HOME dir")
            .data_dir()
            .join("malai")
            .join("default.key"),
    };

    if !secret_key_path.exists() {
        eprintln!(
            "Secret key file does not exist at {secret_key_path:?}. Please create it using `malai keygen {secret_key_path:?}` or provide a valid path."
        );
        return Err(());
    }

    if secret_key_path.is_dir() {
        eprintln!("Secret key path is a directory. Please provide a valid file path.");
    }

    Ok(secret_key_path)
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    pub command: Option<Command>,

    // adding these two because when we run `cargo tauri dev,` it automatically passes these
    // arguments. need to figure out why and how to disable that, till then this is a workaround
    #[arg(default_value = "true", long, hide = true)]
    no_default_features: bool,
    #[arg(default_value = "auto", long, hide = true)]
    color: String,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    // TODO: add this to the docs when we have ACL
    // By default it allows any peer to connect to the HTTP(s) service. You can pass --what-to-do
    // argument to specify a What To Do service that can be used to add access control."
    #[clap(about = "Expose HTTP Service on kulfi, connect using kulfi or browser")]
    Http {
        port: u16,
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Host serving the http service."
        )]
        host: String,
        #[arg(
            long,
            default_value = "kulfi.site",
            help = "Use this for the HTTP bridge. To run an HTTP bridge, use `malai http-bridge`",
            env = "MALAI_HTTP_BRIDGE"
        )]
        bridge: String,
        #[arg(
            long,
            help = "Make the exposed service public. Anyone will be able to access."
        )]
        public: bool,
        #[arg(
            long,
            short('k'),
            help = "The path to the secret key file. If not provided, it'll read it from the default location. Use `malai keygen` to create new secret keys."
        )]
        secret_key_path: Option<String>,
        // #[arg(
        //     long,
        //     default_value_t = false,
        //     help = "Use this if the service is HTTPS"
        // )]
        // secure: bool,
        // #[arg(
        //     long,
        //     help = "The What To Do Service that can be used to add access control."
        // )]
        // this will be the id52 of the identity server that should be consulted
        // what_to_do: Option<String>,
    },
    #[clap(about = "Browse a kulfi site.")]
    Browse {
        #[arg(help = "The Kulfi URL to browse. Should look like kulfi://<id52>/<path>")]
        url: String,
    },
    #[clap(about = "Expose TCP Service on kulfi.")]
    Tcp {
        port: u16,
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Host serving the TCP service."
        )]
        host: String,
        #[arg(
            long,
            help = "Make the exposed service public. Anyone will be able to access."
        )]
        public: bool,
        #[arg(
            long,
            short('k'),
            help = "The path to the secret key file. If not provided, it'll read it from the default location. Use `malai keygen` to create new secret keys."
        )]
        secret_key_path: Option<String>,
    },
    #[clap(
        about = "Run an http server that forwards requests to the given id52 taken from the HOST header"
    )]
    HttpBridge {
        #[arg(
            long,
            short('t'),
            help = "The id52 to which this bridge will forward incoming HTTP request. By default it forwards to every id52."
        )]
        proxy_target: Option<String>,
        #[arg(
            long,
            short('p'),
            help = "The port on which this bridge will listen for incoming HTTP requests.",
            default_value = "8080"
        )]
        port: u16,
    },
    #[clap(about = "Run a TCP server that forwards incoming requests to the given id52.")]
    TcpBridge {
        #[arg(help = "The id52 to which this bridge will forward incoming TCP request.")]
        proxy_target: String,
        #[arg(
            help = "The port on which this bridge will listen for incoming TCP requests.",
            default_value = "8081"
        )]
        port: u16,
    },
    #[clap(about = "Expose a folder to kulfi network")]
    Folder {
        #[arg(help = "The folder to expose.")]
        path: String,
        #[arg(
            long,
            default_value = "kulfi.site",
            help = "Use this for the HTTP bridge. To run an HTTP bridge, use `malai http-bridge`",
            env = "MALAI_HTTP_BRIDGE"
        )]
        bridge: String,
        #[arg(long, help = "Make the folder public. Anyone will be able to access.")]
        public: bool,
        #[arg(
            long,
            short('k'),
            help = "The path to the secret key file. If not provided, it'll read it from the default location. Use `malai keygen` to create new secret keys."
        )]
        secret_key_path: Option<String>,
    },
    #[clap(about = "Generate a new secret key")]
    Keygen {
        #[arg(
            long,
            short('p'),
            help = "The path to store the secret key. Leave empty to use the default location."
        )]
        path: Option<String>,
    },
}
