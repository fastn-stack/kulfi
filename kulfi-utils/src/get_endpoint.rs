pub async fn get_endpoint(secret_key: kulfi_id52::SecretKey) -> eyre::Result<iroh::Endpoint> {
    // Convert kulfi_id52::SecretKey to iroh::SecretKey
    let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key.to_bytes());

    match iroh::Endpoint::builder()
        .discovery(iroh::discovery::pkarr::PkarrPublisher::n0_dns())
        .discovery(iroh::discovery::dns::DnsDiscovery::n0_dns())
        .discovery(iroh::discovery::mdns::MdnsDiscovery::builder())
        .alpns(vec![crate::APNS_IDENTITY.into()])
        .secret_key(iroh_secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network2: {e:?}"))
        }
    }
}
