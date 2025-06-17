extern crate self as kulfi_utils;

pub mod dot_kulfi;
pub mod get_endpoint;
mod get_stream;
mod graceful;
pub mod http;
mod http_connection_manager;
mod http_to_peer;
mod peer_to_http;
mod ping;
pub mod protocol;
mod secret;
mod tcp;
mod utils;

pub use get_endpoint::get_endpoint;
pub use get_stream::{PeerStreamSenders, get_stream};
pub use graceful::Graceful;
pub use http::ProxyResult;
pub use http_connection_manager::{HttpConnectionManager, HttpConnectionPool, HttpConnectionPools};
pub use http_to_peer::{http_to_peer, http_to_peer_non_streaming};
pub use peer_to_http::peer_to_http;
pub use ping::{PONG, ping};
pub use protocol::{APNS_IDENTITY, Protocol, ProtocolHeader};
pub use secret::{
    SECRET_KEY_FILE, generate_and_save_key, generate_secret_key, get_secret_key, read_or_create_key,
};
pub use tcp::{peer_to_tcp, pipe_tcp_stream_over_iroh, tcp_to_peer};
pub use utils::{
    accept_bi, accept_bi_with, get_remote_id52, global_iroh_endpoint, id52_to_public_key, mkdir,
    next_json, next_string, public_key_to_id52,
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
