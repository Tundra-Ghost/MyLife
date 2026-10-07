# MyLife

A local-first Windows 11 app to run your whole life from one place.
The full design is in [docs/SPEC.md](docs/SPEC.md).

Status: Phase 1, step 1. See [docs/CHANGELOG.md](docs/CHANGELOG.md).

## Install

1. Open the [Releases page](https://github.com/Tundra-Ghost/MyLife/releases).
2. Download the `.msi` from the newest release.
3. Run it. Windows may say "Windows protected your PC" because the installer isn't code-signed. Click **More info**, then **Run anyway**.

To update, do the same with a newer release. It upgrades in place and keeps your data.
The version you have shows at the bottom of the sidebar.

A new release is built every time a PR is merged into `main`.

## Build it yourself on Windows

One-time setup:

1. Install [Rust](https://rustup.rs) (default options).
2. Install [Node.js 22 LTS](https://nodejs.org).
3. Install Visual Studio Build Tools with "Desktop development with C++".
4. Install [Strawberry Perl](https://strawberryperl.com). It builds the encryption library.
5. WebView2 already ships with Windows 11.

Then, in this folder:

```
npm install
npm run tauri dev
```

The first build takes several minutes. Later builds are fast.

To make an installer (MSI): `npm run tauri build`. It lands in `src-tauri/target/release/bundle/msi`.

## Tests

```
npm test                         # UI tests (Vitest)
npm run typecheck                # TypeScript
cd src-tauri && cargo test       # Rust tests
```

## Where data lives

`%APPDATA%\com.tundraghost.mylife\mylife.db`, encrypted with your app password.
Backups go in the `backups` folder next to it. If you forget the app password, the data can't be recovered.

## Docs

- [SPEC.md](docs/SPEC.md): the design. Exported from the MyLife Design Document.
- [DECISIONS.md](docs/DECISIONS.md): defaults picked where the spec had a gap.
- [CHANGELOG.md](docs/CHANGELOG.md): what changed.
- [IDEAS.md](docs/IDEAS.md): ideas parked for later.
