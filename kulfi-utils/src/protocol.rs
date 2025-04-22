/// why are we handling protocols ourselves and not using the built-in APNS feature?
/// ================================================================================
///
/// first, I would re-emphasize that we could have used APNS feature of iroh[1], they have built
/// in protocol handling support. but it does not work because if you want to use more than one
/// protocol, you have to create more than one connection with the peer.
///
/// [1]: https://docs.rs/iroh/latest/iroh/endpoint/struct.Builder.html#method.alpns
///
/// why would we want that? say a peer wants to do multiple types of things at the same time. for
/// example, do a ping to check if connection is open, or to send both http and tcp proxy at the
/// same time. consider I am on call with you but also browsing a folder you have shared.
///
/// from the docs it is not clear if creating another connection, after one connection is already
/// established is a cheap operation or not. in my opinion, it cannot be cheap because ALPN is used
/// as part of TLS connection handshake process.
///
/// this is how the `client hello` message looks like during initial TLS connection handshake:
///
/// > Handshake Type: Client Hello (1)
/// >  Length: 141
/// >  Version: TLS 1.2 (0x0303)
/// >  Random: dd67b5943e5efd0740519f38071008b59efbd68ab3114587...
/// >  Session ID Length: 0
/// >  Cipher Suites Length: 10
/// >  Cipher Suites (5 suites)
/// >  Compression Methods Length: 1
/// >  Compression Methods (1 method)
/// >  Extensions Length: 90
/// >  [other extensions omitted]
/// >  Extension: application_layer_protocol_negotiation (len=14)
/// >      Type: application_layer_protocol_negotiation (16)
/// >      Length: 14
/// >      ALPN Extension Length: 12
/// >      ALPN Protocol
/// >          ALPN string length: 2
/// >          ALPN Next Protocol: h2
/// >          ALPN string length: 8
/// >          ALPN Next Protocol: http/1.1
///
/// as you see, the ALPN is part of the `client hello` message, and it is sent during the initial
/// connection handshake. so if we want to use more than one protocol, we have to do one more
/// `client hello` hand-shake proces.
///
/// so we are using multiple [bidirectional][3] streams over a single connection. each new stream
/// con be used for a same or different protocol.
///
/// [3]: https://docs.rs/iroh/latest/iroh/endpoint/struct.Connection.html#method.open_bi
///
/// note: this is not a settled decision. if we are doing audio video streaming, we may not get the
/// optimal performance, and we may have to use multiple connections; this approach is to be
/// verified in the future.
///
/// the protocol "protocol"
/// =======================
///
/// the protocol: the peer / side that wants to communicate will be considered the "client", and
/// will initiate the bidirectional stream using `iroh::Connection::open_bi()` method. the server
/// will have an infinite loop to accept incoming bidirectional streams. for the loop to end, the
/// client must send a "quit" message and wait for ack from the server before closing the connection.
///
/// the bidirectional stream will contain new line terminal JSON text indicating the protocol, and
/// the rest of the message will be handled by the protocol-specific handler.
///
/// the protocol JSON line will be called header line, or stream header.
///
/// the stream header can contain protocol-specific information also, e.g., the request to proxy to
/// a server may include information about the server to proxy to in the protocol header. so that
/// the lower level protocol handler need not worry about further ways to extract protocol-specific
/// data.
///
/// security philosophy: more protocols, more liabilities
/// =====================================================
///
/// the goal of the kulfi is to make sure there are only a few protocols. all protocol
/// handlers are security risk, they are written in Rust, possibly using C and other libraries.
/// their code has to be reviewed for potential security issues.
///
/// this is why fastn is a full stack web application. fastn programs are compiled in JS code, and
/// in future to webassembly, and JS engines have decent security sandbox. we do not allow npm/deno
/// etc., and only run the most sandboxed, browser like JS code. fastn applications can also use
/// webassembly compiled code, which again is sandboxed.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Protocol {
    /// client can send this message to check if the connection is open / healthy.
    Ping,
    /// client may not be using NTP, or may only have p2p access and no other internet access, in
    /// which case it can ask for the time from the peers and try to create a consensus.
    WhatTimeIsIt,
    /// client wants to make an HTTP request to a device whose ID is specified. note that the exact
    /// ip:port is not known to peers, they only the "device id" for the service. server will figure
    /// out the ip:port from the device id.
    Http,
    /// if the client wants their traffic to route via this server, they can send this. for this to
    /// work, the person owning the device must have created a SOCKS5 device, and allowed this peer
    /// to access it.
    Socks5,
    Tcp,
    // TODO: RTP/"RTCP" for audio video streaming
}

/// Iroh supports multiple protocols, and we do need multiple protocols, lets say one for proxying
/// TCP connection, another for proxying HTTP connection, and so on. But if we use different APNS
/// to handle them, we will end up creating more connections than minimally required (one connection
/// can only talk one APNS). So, we use a single APNS for all the protocols, and we use the first
/// line of the input to determine the protocol.
pub const APNS_IDENTITY: &[u8] = b"/kulfi/identity/0.1";
