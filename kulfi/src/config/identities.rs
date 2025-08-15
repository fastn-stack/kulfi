impl kulfi::Config {
    pub async fn identities(
        &self,
        client_pools: fastn_net::HttpConnectionPools,
    ) -> eyre::Result<Vec<kulfi::Identity>> {
        use eyre::WrapErr;

        let mut identities = Vec::new();
        let identities_dir = self.dir.join("identities");
        for entry in std::fs::read_dir(&identities_dir)
            .wrap_err_with(|| format!("failed to run identities folder: {identities_dir:?}"))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.starts_with("temp-") {
                // this might be a leftover folder, we should ideally delete it if is older than
                // say 5 minutes, but for now, we just skip it.
                continue;
            }

            // `.file_name()` is wrongly named, it returns the last component of the path, and
            // not really the "file name".
            let id = match path.file_name().and_then(|v| v.to_str()) {
                Some(id) => id.to_string(),
                None => {
                    return Err(eyre::anyhow!("failed to get file name from path: {path:?}"));
                }
            };

            let identity = kulfi::Identity::read(&identities_dir, id, client_pools.clone())
                .await
                .wrap_err_with(|| format!("failed to read {path:?} as an identity folder"))?;

            identities.push(identity);
        }

        // std::fs::read_dir does not guarantee the order of the entries, so we sort them
        identities.sort_by(|a, b| a.id52.cmp(&b.id52));

        Ok(identities)
    }
}
