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

#[cfg(feature = "keyring")]
pub use secret::KeyringSecretStore;

pub use get_endpoint::get_endpoint;
pub use get_stream::{PeerStreamSenders, get_stream};
pub use http::ProxyResult;
pub use http_connection_manager::{HttpConnectionManager, HttpConnectionPool, HttpConnectionPools};
pub use http_to_peer::http_to_peer;
pub use peer_to_http::peer_to_http;
pub use peer_to_tcp::peer_to_tcp;
pub use ping::{PONG, ping};
pub use protocol::{APNS_IDENTITY, Protocol};
pub use secret::{SecretStore, read_or_create_key};
pub use utils::{
    FrameReader, accept_bi, frame_reader, get_remote_id52, id52_to_public_key, public_key_to_id52,
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
