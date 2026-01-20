# Kulfi & Malai

Open Source, General Purpose, Sovereign, Decentralized, Peer to Peer Internet.

---

## Highlights

- Share your local HTTP/TCP with anyone, without any central server.
- Use public `*.kulfi.site` bridge to access exposed http service or host your own bridge using `malai http-bridge` subcmd.
- Built on top of [iroh][iroh], a p2p networking library.

This project is backed by [FifthTry](https://fifthtry.com/), the creators of [fastn][fastn].

## Malai

Malai is a simple tool that can be used to expose any local service (HTTP, TCP
and, SSH, etc.) to the world. It can be paired up with an ACL system (like
Kulfi) to control access to the exposed services.

Learn more at https://malai.sh.

### Install `malai`

```bash
curl -fsSL https://malai.sh/install.sh | sh
```

## Kulfi

Kulfi is a peer to peer network, free from any corporate control. Data stays
with the user, and devices controlled by the user, and not with some central
company.

Kulfi will soon be available as an binary that you can download and run on your
computer. We will support Linux, Windows and MacOS from day one. We also want to
create Apps that can be distributed through App Stores, and also support mobile
devices.

To learn more about how Kulfi works, see
Journeys [here](https://kulfi.app/doc/journeys/).

`kulfi` and `malai` are built on top of [iroh][iroh], and uses [BitTorrent's
Mainline DHT][MainlineDHT] for peer discovery.


[fastn]: https://fastn.com

[iroh]: https://www.iroh.computer

[MainlineDHT]: https://en.wikipedia.org/wiki/Mainline_DHT

## Licence

This project is licensed under the [UPL](LICENSE) license. UPL is MIT like
license, with Apache 2.0 like patent grant clause.

## Contributing

We welcome contributions to Kulfi & Malai. Please read the
[CONTRIBUTING.md][cont] file for details on how to contribute.

[cont]: CONTRIBUTING.md
