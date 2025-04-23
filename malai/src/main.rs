// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="malai=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let (show_info_tx, show_info_rx) = tokio::sync::watch::channel(false);
    let graceful = kulfi_utils::Graceful::default();

    // TODO: each subcommand should handle their error and return ()
    match cli.command {
        Some(Command::Http {
            port,
            host,
            bridge,
            // secure,
            // what_to_do,
        }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing HTTP service on kulfi.");
            let g = graceful.clone();
            let show_info_rx = show_info_rx.clone();
            graceful.spawn(
                async move { malai::expose_http(host, port, bridge, g, show_info_rx).await },
            );
        }
        Some(Command::HttpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting HTTP bridge.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::http_bridge(port, proxy_target, g).await });
        }
        Some(Command::Tcp { port, host }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing TCP service on kulfi.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::expose_tcp(host, port, g).await });
        }
        Some(Command::TcpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting TCP bridge.");
            let g = graceful.clone();
            graceful.spawn(async move { malai::tcp_bridge(port, proxy_target, g).await });
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

    graceful.shutdown(show_info_tx).await
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
    #[clap(about = "Expose TCP Service on kulfi", hide = true)]
    Tcp {
        port: u16,
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Host serving the TCP service."
        )]
        host: String,
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
    #[clap(hide = true)]
    TcpBridge {
        #[arg(help = "The id52 to which this bridge will forward incoming TCP request.")]
        proxy_target: String,
        #[arg(
            help = "The port on which this bridge will listen for incoming TCP requests.",
            default_value = "8081"
        )]
        port: u16,
    },
}
