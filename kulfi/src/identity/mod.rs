// mod bb8;
mod create;
mod read;
mod run;

// pub use bb8::{PeerIdentity, get_endpoint};

#[derive(Debug)]
pub struct Identity {
    pub id52: String,
    pub public_key: fastn_id52::PublicKey,
    pub client_pools: fastn_net::HttpConnectionPools,
}

impl Identity {
    pub fn from_id52(id: &str, client_pools: fastn_net::HttpConnectionPools) -> eyre::Result<Self> {
        use std::str::FromStr;
        let public_key = fastn_id52::PublicKey::from_str(id)?;
        Ok(Self {
            id52: public_key.to_string(),
            public_key,
            client_pools,
        })
    }
}
