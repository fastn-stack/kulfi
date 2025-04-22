# ChangeLog

## Unreleased

### malai 0.2.0

- Rename subcommands `expose-http` -> `http` and `expose-tcp` -> `tcp`.
- `ctrl+c` to print info. Quick succession of `ctrl+c` within 3 seconds to exit.
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
