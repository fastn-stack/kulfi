# ChangeLog

## 19 June 2025

### malai 0.2.7

- Implemented `malai keygen` to generate a new kulfi identity. This is useful
  if you don't want to store your kulfi identity in keychain.
- All subcmds (`malai http`, `malai tcp`, etc) that interact with the system
  keyring alternatively read from `KULFI_SECRET_KEY` env variable first for
  secret value or the `.malai.secret-key` file if it's in CWD.

## 06 June 2025

### malai 0.2.6

- Add HTTP streaming support. This helps in handling large HTTP responses (like
  media files for `malai folder`) without running into high memory usage.
- implemented `malai http-proxy` and `malai http-proxy-remote` pair to proxy
  HTTP requests to a remote HTTP server over the kulfi network.

## 20 May 2025

### malai 0.2.5

- `malai {http, tcp}-bridge`: `port` is now optional, if you don't provide a
  port, it will be assigned a random port.
- fix: [malai tcp bridge was only handling one concurrent
  connection](https://github.com/fastn-stack/kulfi/issues/61), now it can
  handle multiple connections.

## 14 May 2025

### malai 0.2.4

- fixed: [`malai http-bridge` was giving intermittent `connection refused`
  error][1]
- fixed: `malai http-bridge` used to not cleanly exit because iroh connection
  cleanup was buggy.

[1]: https://github.com/fastn-stack/kulfi/issues/60

## 06 May 2025

### malai 0.2.3

- Implemented `malai tcp` and `malai tcp-bridge` to expose any TCP service over
  kulfi network.
- Implemented `malai folder`. You can now share a folder with people without
  having to manually run another HTTP server. Requires `--public` flag as no
  ACL yet, also readonly mode for now.

## 30 April 2025

### malai 0.2.2

- Implemented `malai browse`. You can now browse a malai powered site without
  using any bridge.
- `malai http` subcommand requires a `--public` flag to run. This will be made
  optional when we have access control.
- `malai.sh/install.sh`: refuse to install on non-Apple M series Macs. This is
  to prevent segfaults on Intel Macs. See [issue
  #28](https://github.com/fastn-stack/kulfi/issues/28).

## 23 April 2025

### malai 0.2.1

This is a minor release with not changes to `malai`. We've restricted the
release binary to be only available for **Apple M series Macs (arm64)**. This
is done because the x86_64 build is segfaulting when run on intel macs and we
can't figure out the cause.

More details at: https://github.com/fastn-stack/kulfi/issues/28

## 22 April 2025

### malai 0.2.0

- Feat: `ctrl+c` to print info. Quick succession of `ctrl+c` within 3 seconds to
  exit. [More details](https://github.com/fastn-stack/kulfi/discussions/9)
- Feat: Configurable HTTP bridge address in the
  output. [More details](https://github.com/fastn-stack/kulfi/discussions/17)
- Breaking: Rename subcommands `expose-http` -> `http` and `expose-tcp` ->
  `tcp`.
- Breaking [Networking Internals]: Merged `Protocol::Identity` with
  `Protocol::Http`, this means a `malai 0.1` http-bridge can not connect with
  `malai 0.2 http`.

## 17 April 2025

### malai 0.1.1

- Colored output for `malai expose-http` command. Now prints id52, the local
  service it's exposing and, a `<id52>.kulfi.app link`.
- Install script for linux and mac at `malai.sh/install.sh`. Run
  `source < "$(curl -fsSL https://malai.sh/install.sh)"`.

## 16 April 2025

### malai 0.1.0

- Initial release of `malai` binary.
