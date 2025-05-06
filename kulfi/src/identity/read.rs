impl kulfi::Identity {
    pub async fn read(
        identities_dir: &std::path::Path,
        id: String,
        client_pools: kulfi_utils::HttpConnectionPools,
    ) -> eyre::Result<Self> {
        Self::from_id52(identities_dir, id.as_str(), client_pools)
    }
}
