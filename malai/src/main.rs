// prevents an additional console window on Windows in release, DO NOT REMOVE!
#![cfg_attr(
    all(not(debug_assertions), feature = "ui"),
    windows_subsystem = "windows"
)]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="malai=trace,kulfi_utils=trace" to see logs
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let graceful = kulfi_utils::Graceful::default();

    match cli.command {
        Some(Command::Http {
            port,
            host,
            bridge,
            public,
            // secure,
            // what_to_do,
        }) => {
            if !malai::public_check(
                public,
                "HTTP service",
                &format!("malai http {port} --public"),
            ) {
                return Ok(());
            }

            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing HTTP service on kulfi.");
            let graceful_for_export_http = graceful.clone();
            graceful.spawn(async move {
                malai::expose_http(host, port, bridge, graceful_for_export_http).await
            });
        }
        Some(Command::HttpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting HTTP bridge.");
            let graceful_for_http_bridge = graceful.clone();
            graceful.spawn(async move {
                malai::http_bridge(port, proxy_target, graceful_for_http_bridge, |_| Ok(())).await
            });
        }
        Some(Command::Tcp { port, host, public }) => {
            if !malai::public_check(
                public,
                "HTTP service",
                &format!("malai http {port} --public"),
            ) {
                return Ok(());
            }

            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing TCP service on kulfi.");
            let graceful_for_expose_tcp = graceful.clone();
            graceful
                .spawn(async move { malai::expose_tcp(host, port, graceful_for_expose_tcp).await });
        }
        Some(Command::TcpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting TCP bridge.");
            let graceful_for_tcp_bridge = graceful.clone();
            graceful.spawn(async move {
                malai::tcp_bridge(port, proxy_target, graceful_for_tcp_bridge).await
            });
        }
        Some(Command::Browse { url }) => {
            tracing::info!(url, verbose = ?cli.verbose, "Opening browser.");
            let graceful_for_browse = graceful.clone();
            graceful.spawn(async move { malai::browse(url, graceful_for_browse).await });
        }
        Some(Command::Folder {
            path,
            bridge,
            public,
        }) => {
            if !malai::public_check(public, "folder", &format!("malai folder --public {path}")) {
                return Ok(());
            }

            tracing::info!(path, verbose = ?cli.verbose, "Exposing folder to kulfi network.");
            let graceful_for_folder = graceful.clone();
            graceful.spawn(async move { malai::folder(path, bridge, graceful_for_folder).await });
        }
        Some(Command::Run { home }) => {
            tracing::info!(verbose = ?cli.verbose, "Running all services.");
            let graceful_for_run = graceful.clone();
            graceful.spawn(async move { malai::run(home, graceful_for_run).await });
        }
        Some(Command::HttpProxyRemote { public }) => {
            if !malai::public_check(
                public,
                "http-proxy-remote",
                "malai http-proxy-remote --public",
            ) {
                return Ok(());
            }
            tracing::info!(verbose = ?cli.verbose, "Running HTTP Proxy Remote.");
            let graceful_for_run = graceful.clone();
            graceful.spawn(async move { malai::http_proxy_remote(graceful_for_run).await });
        }
        Some(Command::HttpProxy { remote, port }) => {
            tracing::info!(port, remote, verbose = ?cli.verbose, "Starting HTTP Proxy.");
            let graceful_for_tcp_bridge = graceful.clone();
            graceful.spawn(async move {
                malai::http_proxy(port, remote, graceful_for_tcp_bridge, |_| Ok(())).await
            });
        }
        Some(Command::Generate { file }) => {
            tracing::info!(verbose = ?cli.verbose, "Generating new identity.");
            malai::generate(file).await?;
            return Ok(());
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
            help = "The port on which this bridge will listen for incoming HTTP requests. If you pass 0, it will bind to a random port.",
            default_value = "0"
        )]
        port: u16,
    },
    #[clap(about = "Run a TCP server that forwards incoming requests to the given id52.")]
    TcpBridge {
        #[arg(help = "The id52 to which this bridge will forward incoming TCP request.")]
        proxy_target: String,
        #[arg(
            help = "The port on which this bridge will listen for incoming TCP requests. If you pass 0, it will bind to a random port.",
            default_value = "0"
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
    },
    #[clap(about = "Run all the services")]
    Run {
        #[arg(long, help = "Malai Home", env = "MALAI_HOME")]
        home: Option<String>,
    },
    #[clap(about = "Run an iroh remote server that handles requests from http-proxy.")]
    HttpProxyRemote {
        #[arg(long, help = "Make the proxy public. Anyone will be able to access.")]
        public: bool,
    },
    #[clap(about = "Run a http proxy server that forwards incoming requests to http-proxy-remote.")]
    HttpProxy {
        #[arg(help = "The id52 of remote to which this http proxy will forward request to.")]
        remote: String,
        #[arg(
            help = "The port on which this proxy will listen for incoming TCP requests. If you pass 0, it will bind to a random port.",
            default_value = "0"
        )]
        port: u16,
    },
    #[clap(about = "Generate a new identity.")]
    Generate {
        #[arg(
            long,
            short,
            num_args=0..=1,
            default_missing_value=kulfi_utils::SECRET_KEY_FILE,
            help = "The file where the generated key will be live, by default we do not create any file."
        )]
        file: Option<String>,
    },
}
