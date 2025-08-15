// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="kulfi=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();
    // configure_tracing_subscriber();

    let cli = Cli::parse();

    let graceful = fastn_net::Graceful::default();

    if let Err(e) = match cli.command {
        Some(Command::Start {
            foreground,
            data_dir,
            control_port,
        }) => {
            let data_dir = match data_dir {
                Some(dir) => dir.into(),
                // https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html#method.data_dir
                None => match directories::ProjectDirs::from("com", "FifthTry", "kulfi") {
                    Some(dir) => dir.data_dir().to_path_buf(),
                    None => {
                        return Err(eyre::anyhow!(
                            "dot_kulfi init failed: can not find data dir when dir is not provided"
                        ));
                    }
                },
            };

            kulfi::start(foreground, data_dir, control_port, graceful.clone()).await
        }
        Some(Command::Browse { url }) => {
            tracing::info!(url, verbose = ?cli.verbose, "Opening browser.");
            kulfi::browse(url, graceful.clone()).await
        }
        None => {
            tracing::info!(verbose = ?cli.verbose, "Starting UI.");
            kulfi::ui()
        }
    } {
        tracing::error!("Error: {e:?}");
    }

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
    #[clap(about = "Start the kulfi service.")]
    Start {
        #[arg(default_value_t = false, short = 'f')]
        foreground: bool,
        #[arg(long, short = 'd')]
        data_dir: Option<String>,
        #[arg(default_value_t = 80, long, short = 'p')]
        control_port: u16,
    },
    #[clap(about = "Browse a kulfi site.")]
    Browse { url: String },
}
