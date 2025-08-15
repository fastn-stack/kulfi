impl kulfi::Identity {
    pub async fn read(
        _path: &std::path::Path,
        id: String,
        client_pools: fastn_net::HttpConnectionPools,
    ) -> eyre::Result<Self> {
        Self::from_id52(id.as_str(), client_pools)
    }
}
