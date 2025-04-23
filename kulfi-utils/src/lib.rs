extern crate self as kulfi_utils;

pub mod get_endpoint;
mod get_stream;
pub mod http;
mod http_connection_manager;
mod http_to_peer;
mod peer_to_http;
mod peer_to_tcp;
mod ping;
pub mod protocol;
mod secret;
mod utils;

use eyre::Context;
#[cfg(feature = "keyring")]
pub use secret::KeyringSecretStore;

pub use get_endpoint::get_endpoint;
pub use get_stream::{get_stream, PeerStreamSenders};
pub use http::ProxyResult;
pub use http_connection_manager::{HttpConnectionManager, HttpConnectionPool, HttpConnectionPools};
pub use http_to_peer::http_to_peer;
pub use peer_to_http::peer_to_http;
pub use peer_to_tcp::peer_to_tcp;
pub use ping::{ping, PONG};
pub use protocol::{Protocol, APNS_IDENTITY};
pub use secret::{read_or_create_key, SecretStore};
pub use utils::{
    accept_bi, frame_reader, get_remote_id52, id52_to_public_key, public_key_to_id52, FrameReader,
};

/// IDMap stores the fastn port and the endpoint for every identity
///
/// why is it a Vec and not a HasMap? the incoming requests contain the first few characters of id
/// and not the full id. the reason for this is we want to use <id>.localhost.direct as well, and
/// subdomain can be max 63 char long, and our ids are 64 chars. if we use <id>.kulfi, then this
/// will not be a problem. we still do prefix match instead of exact match just to be sure.
///
/// since the number of identities will be small, a prefix match is probably going to be the same
/// speed as the hash map exact lookup.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, (u16, iroh::endpoint::Endpoint))>>>;

const ACK: &str = "ack";

#[derive(Clone, Default)]
pub struct Graceful {
    pub cancel: tokio_util::sync::CancellationToken,
    pub tracker: tokio_util::task::TaskTracker,
}

impl Graceful {
    pub async fn shutdown(
        &self,
        show_info_tx: tokio::sync::watch::Sender<bool>,
    ) -> eyre::Result<()> {
        loop {
            tokio::signal::ctrl_c()
                .await
                .wrap_err_with(|| "failed to get ctrl-c signal handler")?;

            tracing::info!("Received ctrl-c signal, showing info.");

            show_info_tx
                .send(true)
                .inspect_err(|e| tracing::error!("failed to send show info signal: {e:?}"))?;

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received second ctrl-c signal, shutting down.");
                    self.cancel.cancel();
                    self.tracker.close();

                    self.tracker.wait().await;
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    tracing::info!("Timeout expired. Continuing...");
                    println!("Did not receive ctrl+c within 3 secs. Press ctrl+c in quick succession to exit.");
                }
            }
        }

        Ok(())
    }

    pub fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture<'_> {
        self.cancel.cancelled()
    }
}
