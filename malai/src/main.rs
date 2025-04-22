// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;
    use eyre::WrapErr;

    // run with RUST_LOG="malai=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let (graceful_shutdown_tx, graceful_shutdown_rx) = tokio::sync::watch::channel(false);
    let (show_info_tx, show_info_rx) = tokio::sync::watch::channel(false);

    // TODO: each subcommand should handle their error and return ()
    let help_shown = cli.command.is_none();
    match cli.command {
        Some(Command::Http {
            port,
            host,
            // secure,
            // what_to_do,
        }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing HTTP service on kulfi.");
            let rx = graceful_shutdown_rx.clone();
            let show_info_rx = show_info_rx.clone();
            tokio::spawn(async move { malai::expose_http(host, port, rx, show_info_rx).await });
        }
        Some(Command::HttpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting HTTP bridge.");
            let rx = graceful_shutdown_rx.clone();
            tokio::spawn(async move { malai::http_bridge(port, proxy_target, rx).await });
        }
        Some(Command::Tcp { port, host }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing TCP service on kulfi.");
            let rx = graceful_shutdown_rx.clone();
            tokio::spawn(async move { malai::expose_tcp(host, port, rx).await });
        }
        Some(Command::TcpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting TCP bridge.");
            let rx = graceful_shutdown_rx.clone();
            tokio::spawn(async move { malai::tcp_bridge(port, proxy_target, rx).await });
        }
        #[cfg(feature = "ui")]
        None => {
            tracing::info!(verbose = ?cli.verbose, "Starting UI.");
            let _ = malai::ui();
        }
        #[cfg(not(feature = "ui"))]
        None => {
            use clap::CommandFactory;
            // TODO: handle error here
            Cli::command().print_help().unwrap();
        }
    };

    if !help_shown {
        loop {
            tokio::signal::ctrl_c()
                .await
                .wrap_err_with(|| "failed to get ctrl-c signal handler")?;

            tracing::info!("Received ctrl-c signal, showing info.");

            show_info_tx
                .send(true)
                .inspect_err(|e| tracing::error!("failed to send show info signal: {e:?}"))?;

            tokio::pin! {
                let second_ctrl_c = tokio::signal::ctrl_c();
                let timeout = tokio::time::sleep(std::time::Duration::from_secs(3));
            };

            tokio::select! {
                _ = &mut second_ctrl_c => {
                    tracing::info!("Received second ctrl-c signal, shutting down.");

                    graceful_shutdown_tx
                        .send(true)
                        .wrap_err_with(|| "failed to send graceful shutdown signal")?;

                    // TODO: wait for the running task to finish with a timeout. Setup global counters.
                    break;
                }
                _ = &mut timeout => {
                    tracing::info!("Timeout expired. Continuing...");
                    println!("Did not receive ctrl+c within 3 secs. Press ctrl+c in quick succession to exit.");
                }
            }
        }
    }

    Ok(())
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
