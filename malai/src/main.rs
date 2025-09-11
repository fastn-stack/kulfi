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
        Some(Command::Cluster { cluster_command }) => {
            match cluster_command {
                ClusterCommand::Init { cluster_name } => {
                    malai::init_cluster(cluster_name.clone()).await?;
                    return Ok(());
                }
            }
        }
        Some(Command::Machine { machine_command }) => {
            match machine_command {
                MachineCommand::Init { cluster_manager, cluster_alias } => {
                    if let Err(e) = malai::init_machine_for_cluster(cluster_manager.clone(), cluster_alias.clone()).await {
                        println!("❌ Machine initialization failed: {}", e);
                    }
                    return Ok(());
                }
            }
        }
        Some(Command::Daemon { environment, foreground }) => {
            if environment {
                // Print environment variables  
                let malai_home = if let Ok(home) = std::env::var("MALAI_HOME") {
                    std::path::PathBuf::from(home)
                } else {
                    dirs::data_dir().unwrap_or_default().join("malai")
                };
                println!("MALAI_HOME={}", malai_home.display());
                println!("MALAI_DAEMON_SOCK={}", malai_home.join("malai.sock").display());
                return Ok(());
            }
            
            // Start real daemon
            malai::start_real_daemon(foreground).await?;
            return Ok(());
        }
        Some(Command::Info) => {
            malai::show_cluster_info().await?;
            return Ok(());
        }
        Some(Command::Status) => {
            malai::show_detailed_status().await?;
            return Ok(());
        }
        Some(Command::TestSimple) => {
            malai::test_simple_server().await?;
            return Ok(());
        }
        Some(Command::StartServer) => {
            let identity = fastn_id52::SecretKey::generate();
            println!("🔥 Starting real malai server with identity: {}", identity.id52());
            malai::run_malai_server(identity).await?;
            return Ok(());
        }
        Some(Command::TestReal) => {
            println!("🧪 Testing complete malai infrastructure...");
            
            // Generate cluster manager and machine identities
            let cluster_manager_key = fastn_id52::SecretKey::generate();  
            let machine_key = fastn_id52::SecretKey::generate();
            
            let cm_id52 = cluster_manager_key.id52();
            let machine_id52 = machine_key.id52();
            
            println!("🔑 Cluster Manager: {}", cm_id52);
            println!("🔑 Machine: {}", machine_id52);
            
            // Start machine server (waits for config)
            fastn_p2p::spawn(async move {
                if let Err(e) = malai::run_malai_server(machine_key).await {
                    println!("❌ Machine server failed: {}", e);
                }
            });
            
            // Wait for machine to start
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            
            // Test 1: Config distribution
            println!("📤 Step 1: Testing config distribution...");
            let sample_config = format!(r#"[cluster_manager]
id52 = "{}"
cluster_name = "test"

[machine.server1]
id52 = "{}"  
allow_from = "*"
"#, cm_id52, machine_id52);
            
            if let Err(e) = malai::send_config(cluster_manager_key.clone(), &machine_id52, &sample_config).await {
                println!("❌ Config distribution failed: {}", e);
                return Ok(());
            }
            
            println!("✅ Config distribution successful");
            
            // Wait for config to be processed
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            // Test 2: Command execution (should work after config)
            println!("📤 Step 2: Testing command execution...");
            if let Err(e) = malai::send_command(cluster_manager_key, &machine_id52, "echo", vec!["Complete malai infrastructure working!".to_string()]).await {
                println!("❌ Command execution failed: {}", e);
                return Ok(());
            }
            
            println!("🎉 Complete malai infrastructure test successful!");
            return Ok(());
        }
        Some(Command::ScanRoles) => {
            println!("🔍 Scanning cluster roles...");
            let roles = malai::scan_cluster_roles().await?;
            
            if roles.is_empty() {
                println!("❌ No clusters found");
            } else {
                println!("\n📊 Summary:");
                for (alias, identity, role) in roles {
                    println!("   {} ({}): {:?}", alias, &identity.id52()[..8], role);
                }
            }
            return Ok(());
        }
        Some(Command::Rescan { check, cluster_name }) => {
            if check {
                match cluster_name {
                    Some(cluster) => {
                        println!("🔍 Checking configuration validity for cluster: {}", cluster);
                        malai::check_cluster_config(&cluster).await?;
                    }
                    None => {
                        println!("🔍 Checking configuration validity...");
                        malai::check_all_configs().await?;
                    }
                }
                return Ok(());
            } else {
                match cluster_name {
                    Some(cluster) => {
                        println!("🔄 Rescanning cluster: {}", cluster);
                        malai::check_cluster_config(&cluster).await?;
                        malai::reload_daemon_config_selective(cluster).await?;
                    }
                    None => {
                        println!("🔄 Rescanning and applying configuration changes...");
                        malai::check_all_configs().await?;
                        malai::reload_daemon_config().await?;
                    }
                }
                return Ok(());
            }
        }
        Some(Command::Service { service_command }) => {
            match service_command {
                ServiceCommand::Add { service_type, name, target } => {
                    println!("Adding {} service: {} → {}", service_type, name, target);
                    todo!("Implement service add command");
                }
                ServiceCommand::Remove { name } => {
                    println!("Removing service: {}", name);
                    todo!("Implement service remove command");
                }
                ServiceCommand::List => {
                    println!("Listing services...");
                    todo!("Implement service list command");
                }
            }
        }
        Some(Command::Identity { identity_command }) => {
            match identity_command {
                IdentityCommand::Create { name } => {
                    println!("Creating identity: {:?}", name);
                    todo!("Implement identity create command (replaces keygen)");
                }
                IdentityCommand::List => {
                    println!("Listing identities...");
                    todo!("Implement identity list command");
                }
                IdentityCommand::Export { name } => {
                    println!("Exporting identity: {}", name);
                    todo!("Implement identity export command");
                }
                IdentityCommand::Import { file } => {
                    println!("Importing identity from: {}", file);
                    todo!("Implement identity import command");
                }
                IdentityCommand::Delete { name } => {
                    println!("Deleting identity: {}", name);
                    todo!("Implement identity delete command");
                }
            }
        }
        Some(Command::Config { config_command }) => {
            match config_command {
                ConfigCommand::Download { cluster } => {
                    println!("📥 Downloading config for cluster: {}", cluster);
                    todo!("Implement config download with version hash");
                }
                ConfigCommand::Upload { file, force } => {
                    if force {
                        println!("⚠️  Force uploading config: {}", file);
                        todo!("Implement force config upload (bypass hash check)");
                    } else {
                        println!("📤 Uploading config: {}", file);
                        todo!("Implement config upload with hash validation");
                    }
                }
                ConfigCommand::Edit { cluster } => {
                    println!("✏️  Editing config for cluster: {}", cluster);
                    todo!("Implement atomic config edit with $EDITOR");
                }
                ConfigCommand::Show { cluster } => {
                    println!("📋 Showing config for cluster: {}", cluster);
                    todo!("Implement config show command");
                }
                ConfigCommand::Validate { file } => {
                    println!("✅ Validating config: {}", file);
                    todo!("Implement config validation command");
                }
            }
        }
        Some(Command::External(args)) => {
            // Handle direct SSH syntax: malai <machine> <command>
            if args.len() >= 1 {
                let machine = &args[0];
                if args.len() >= 2 {
                    let command = &args[1];
                    let cmd_args: Vec<String> = args[2..].iter().map(|s| s.to_string()).collect();
                    // Direct CLI mode - works without daemon (MVP primary mode)
                    if let Err(e) = malai::execute_direct_command(machine, command, cmd_args).await {
                        println!("❌ Command failed: {}", e);
                    }
                } else {
                    // Interactive shell
                    println!("Starting shell on machine '{}'", machine);
                    todo!("Implement interactive shell");
                }
            } else {
                println!("❌ Usage: malai <machine> [command] [args...]");
            }
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
    // Core malai commands (promoted from SSH):
    #[clap(about = "Cluster manager commands")]
    Cluster {
        #[command(subcommand)]
        cluster_command: ClusterCommand,
    },
    #[clap(about = "Machine commands")]
    Machine {
        #[command(subcommand)]
        machine_command: MachineCommand,
    },
    #[clap(about = "Start malai daemon (cluster managers + SSH daemon + service proxy)", name = "daemon", alias = "d")]
    Daemon {
        #[arg(
            long,
            short = 'e',
            help = "Print environment variables for shell integration"
        )]
        environment: bool,
        #[arg(
            long,
            help = "Run in foreground (don't daemonize - for systemd/supervisor)"
        )]
        foreground: bool,
    },
    #[clap(about = "Show cluster information for this machine")]
    Info,
    #[clap(about = "Show detailed daemon and cluster status")]
    Status,
    #[clap(about = "Test simple P2P server")]
    TestSimple,
    #[clap(about = "Start real malai server")]
    StartServer,
    #[clap(about = "Test real malai P2P")]
    TestReal,
    #[clap(about = "Scan and show cluster roles")]
    ScanRoles,
    #[clap(about = "Reload configuration changes")]
    Rescan {
        #[arg(long, help = "Check config validity without applying changes")]
        check: bool,
        #[arg(help = "Rescan only specific cluster (optional)")]
        cluster_name: Option<String>,
    },
    #[clap(about = "Service management commands")]
    Service {
        #[command(subcommand)]
        service_command: ServiceCommand,
    },
    #[clap(about = "Remote cluster config management commands")]
    Config {
        #[command(subcommand)]
        config_command: ConfigCommand,
    },
    #[clap(about = "Identity management commands")]
    Identity {
        #[command(subcommand)]
        identity_command: IdentityCommand,
    },
    #[clap(external_subcommand)]
    External(Vec<String>),
}

#[derive(clap::Subcommand, Debug)]
pub enum ClusterCommand {
    #[clap(about = "Initialize a new cluster")]
    Init {
        #[arg(help = "Cluster name")]
        cluster_name: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum MachineCommand {
    #[clap(about = "Initialize machine for cluster")]
    Init {
        #[arg(help = "Cluster manager ID52 or domain name")]
        cluster_manager: String,
        #[arg(help = "Local alias for cluster")]
        cluster_alias: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ServiceCommand {
    #[clap(about = "Add service configuration")]
    Add {
        #[arg(help = "Service type: ssh, tcp, or http")]
        service_type: String,
        #[arg(help = "Service name")]
        name: String,
        #[arg(help = "Service target")]
        target: String,
    },
    #[clap(about = "Remove service configuration")]
    Remove {
        #[arg(help = "Service name")]
        name: String,
    },
    #[clap(about = "List all configured services")]
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum IdentityCommand {
    #[clap(about = "Create new identity")]
    Create {
        #[arg(help = "Identity name (optional)")]
        name: Option<String>,
    },
    #[clap(about = "List all identities")]
    List,
    #[clap(about = "Export identity")]
    Export {
        #[arg(help = "Identity name")]
        name: String,
    },
    #[clap(about = "Import identity")]
    Import {
        #[arg(help = "Identity file path")]
        file: String,
    },
    #[clap(about = "Delete identity")]
    Delete {
        #[arg(help = "Identity name")]
        name: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommand {
    #[clap(about = "Download cluster config for editing")]
    Download {
        #[arg(help = "Cluster alias")]
        cluster: String,
    },
    #[clap(about = "Upload edited cluster config")]  
    Upload {
        #[arg(help = "Config file path")]
        file: String,
        #[arg(long, help = "Force upload (bypass hash check)")]
        force: bool,
    },
    #[clap(about = "Edit cluster config with $EDITOR")]
    Edit {
        #[arg(help = "Cluster alias")]
        cluster: String,
    },
    #[clap(about = "Show current cluster config")]
    Show {
        #[arg(help = "Cluster alias")]
        cluster: String,
    },
    #[clap(about = "Validate config file syntax")]
    Validate {
        #[arg(help = "Config file path")]
        file: String,
    },
}



