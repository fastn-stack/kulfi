# ChangeLog

## Unreleased

### malai 0.3.0

- Remove dependency on the `keyring` crate. This allows use to use `malai`
where keyring does not exist (AWS EC2 for example).
- `malai keygen`: Generate new secret keys. These keys are required by various
other subcommands to functions and are stored in user's data dir
($HOME/.local/share/) by default.
- Add `-k` (`--secret-key-path`) flag to `http`, `tcp` and, `folder` subcmd
that takes a path to the key generate via `malai keygen`.

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
  #28](https://github.com/kulfi-project/kulfi/issues/28).

## 23 April 2025

### malai 0.2.1

This is a minor release with not changes to `malai`. We've restricted the
release binary to be only available for **Apple M series Macs (arm64)**. This
is done because the x86_64 build is segfaulting when run on intel macs and we
can't figure out the cause.

More details at: https://github.com/kulfi-project/kulfi/issues/28

## 22 April 2025

### malai 0.2.0

- Feat: `ctrl+c` to print info. Quick succession of `ctrl+c` within 3 seconds to
  exit. [More details](https://github.com/kulfi-project/kulfi/discussions/9)
- Feat: Configurable HTTP bridge address in the
  output. [More details](https://github.com/kulfi-project/kulfi/discussions/17)
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
