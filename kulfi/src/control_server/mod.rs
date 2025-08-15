//! what is the control server?
//! ===========================
//!
//! the control server is the main entry point for kulfi.
//!
//! one way to set things up is that kulfi runs on port 80 and 443 on say 127.0.0.23[1], and there
//! is a wildcard domain entry for say `<id>.kulfi` and ensure that on the OS `*.kulfi` resolves to
//! 127.0.0.23. further, kulfi installs a trusted root certificate in the OS, and `*.kulfi` domains
//! are accessible via HTTPS on the current machine.
//!
//! [1]: why not 127.0.0.1? so if you have any other running on 127.0.0.1:{80, 443} things do not
//!      conflict. TODO: test this assumption
//!
//! this requires us install root certificate, and to manage DNS resolution process somehow.
//!
//! the other way would be for kulfi to include a tauri based browser, so you can not access kulfi
//! sites via normal browser/curl/scripts etc, but can access via kulfi Browser only.
//!
//! the kulfiBrowser approach has other advantages like we can ensure media streaming works without
//! relying WebRTC, which will not work with our access control mechanisms. we can even ditch the
//! entire web rendering and do our own custom rendering as kulfi largely only needs to render
//! fastn frontend, not general purpose HTML/CSS/JS "nightmare". if fastn can be made to compile
//! frontend to wasm, and we can do native rendering, we can get rid of CSS engine, the JS engine
//! and all the myriad of half baked, out dated technologies, and recreate the internet from scratch
//! based on lessons learnt so far.
//!
//! <id>.kulfi sites
//! =================
//!
//! The job of kulfi is mainly make sure <id>.kulfi sites work. All traffic for <id>.kulfi will
//! arrive at the "control server".
//!
//! if the <id> is one of the home identities, meaning managed by this instance of kulfi running on
//! this machine, there must be a fastn server running on this machine too, ensured by
//! `identity/run.rs`. so the control server simply forwards / "proxy passes" the traffic to the
//! corresponding fastn server.
//!
//! if the <id> is anything else, control server opens an iroh connection with the <id>, and proxy
//! passes the request to that connection.
//!
//! what about http devices?
//! ========================
//!
//! last section was simplification, the story is a bit more complex. each identity has a bunch of
//! "devices", and one of the device kind is "http", meaning you have access to a HTTP server you
//! want to share over kulfi.
//!
//! kulfi is aware of all such devices too, and the http device configuration stores the scheme/IP/
//! port/extra headers etc, so it can simply forward the request to that server.
//!
//! but it does something a bit more interesting, it first makes a http request to the device's
//! parent identity's corresponding fastn server, `/kulfi/v1/identity/{device-id}/http/<remote-id>/`,
//! so the fastn server can decide if this remote can access this device or not. we do not implement
//! permission system in kulfi itself, and rely on fastn's permission system.
//!
//! once we get a go ahead from fastn, we go and do the proxy pass business.
//!
//! if you notice we mentioned extra headers, this is in case you want to "manipulate" the http
//! request before you send, such extra headers and other "manipulation hints" can also be returned
//! by the fastn's api call.

mod proxy_pass;
mod server;

pub use proxy_pass::proxy_pass;

pub async fn start(
    control_port: u16,
    id: String,
    graceful: fastn_net::Graceful,
    id_map: fastn_net::IDMap,
    client_pools: fastn_net::HttpConnectionPools,
    peer_connections: fastn_net::PeerStreamSenders,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{control_port}"))
        .await
        .wrap_err_with(
            || "can not listen to port 80, is it busy, or you do not have root access?",
        )?;
    tracing::info!("Listening on http://{id}.localhost.direct");

    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            val = listener.accept() => {
                let id_map= id_map.clone();
                let client_pools = client_pools.clone();
                let graceful = graceful.clone();
                let peer_connections = peer_connections.clone();
                match val {
                    Ok((stream, _addr)) => {
                        tokio::spawn(async move { server::handle_connection(stream, graceful, id_map, client_pools, peer_connections).await });
                    },
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                    }
                }
            }
        }
    }

    Ok(())
}
