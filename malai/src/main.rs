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
        Some(Command::Keygen { file }) => {
            tracing::info!(verbose = ?cli.verbose, "Generating new identity.");
            malai::keygen(file);
            return Ok(());
        }
        Some(Command::Ssh { ssh_command }) => {
            match ssh_command {
                SshCommand::InitCluster { alias } => {
                    malai::create_cluster(alias.clone()).await?;
                    return Ok(());
                }
                SshCommand::Init => {
                    malai::init_machine().await?;
                    return Ok(());
                }
                SshCommand::Agent { environment, lockdown, http } => {
                    malai::start_ssh_agent(environment, lockdown, http).await?;
                    return Ok(());
                }
                SshCommand::ClusterInfo => {
                    malai::show_cluster_info().await?;
                    return Ok(());
                }
                SshCommand::Execute { machine, command, args } => {
                    malai::execute_ssh_command(&machine, &command, args).await?;
                    return Ok(());
                }
                SshCommand::External(args) => {
                    // Handle "malai ssh <machine> <command>" syntax  
                    if args.len() >= 1 {
                        let machine = &args[0];
                        if args.len() >= 2 {
                            // Command with args: malai ssh web01 "echo hello"
                            let full_command = args[1..].join(" ");
                            // Split command from args
                            let command_parts: Vec<&str> = full_command.split_whitespace().collect();
                            if !command_parts.is_empty() {
                                let command = command_parts[0];
                                let cmd_args: Vec<String> = command_parts[1..].iter().map(|s| s.to_string()).collect();
                                malai::execute_ssh_command(machine, command, cmd_args).await?;
                            }
                        } else {
                            // Just machine, start interactive shell
                            println!("Starting shell on machine '{}'", machine);
                            println!("❌ SSH shell not yet implemented");
                        }
                    } else {
                        println!("❌ Usage: malai ssh <machine> [command] [args...]");
                    }
                    return Ok(());
                }
                SshCommand::Shell { machine } => {
                    println!("Starting shell on machine '{}'", machine);
                    println!("❌ SSH shell not yet implemented");
                    return Ok(());
                }
                SshCommand::Curl { url, curl_args: _ } => {
                    println!("Curl to '{}'", url);
                    println!("❌ SSH curl not yet implemented");
                    return Ok(());
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
    Keygen {
        #[arg(
            long,
            short,
            num_args=0..=1,
            default_missing_value=kulfi_utils::SECRET_KEY_FILE,
            help = "The file where the private key of the identity will be stored. If not provided, the private key will be printed to stdout."
        )]
        file: Option<String>,
    },
    #[clap(about = "SSH functionality for secure P2P remote access")]
    Ssh {
        #[command(subcommand)]
        ssh_command: SshCommand,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum SshCommand {
    #[clap(about = "Initialize a new cluster (generates cluster manager identity)")]
    InitCluster {
        #[arg(long, help = "Optional cluster alias")]
        alias: Option<String>,
    },
    #[clap(about = "Initialize machine for SSH (generates identity)")]
    Init,
    #[clap(about = "Start SSH agent for connection management and HTTP proxy")]
    Agent {
        #[arg(
            long,
            short = 'e',
            help = "Print environment variables for shell integration"
        )]
        environment: bool,
        #[arg(long, help = "Enable lockdown mode (keys only accessible to agent)")]
        lockdown: bool,
        #[arg(long, help = "Enable HTTP proxy functionality", default_value = "true")]
        http: bool,
    },
    #[clap(about = "Show cluster information for this machine")]
    ClusterInfo,
    #[clap(about = "Execute command on remote machine", name = "exec")]
    Execute {
        #[arg(help = "Machine address (e.g., web01.company.com, web01.cluster-id52)")]
        machine: String,
        #[arg(help = "Command to execute")]
        command: String,
        #[arg(help = "Command arguments")]
        args: Vec<String>,
    },
    #[clap(external_subcommand)]
    External(Vec<String>),
    #[clap(about = "Start interactive shell session on remote machine")]
    Shell {
        #[arg(help = "Machine address (e.g., web01.company.com, web01.cluster-id52)")]
        machine: String,
    },
    #[clap(about = "Access HTTP service through malai network")]
    Curl {
        #[arg(help = "Service URL (e.g., admin.web01.company.com/api)")]
        url: String,
        #[arg(help = "Additional curl arguments")]
        curl_args: Vec<String>,
    },
}
