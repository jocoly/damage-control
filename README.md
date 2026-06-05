# Knight Shift

Knight Shift is a small Tauri + SvelteKit prototype about earning Influence from real-world keyboard and mouse activity.

## Install

Windows users can download and run:

```text
release/knight-shift-installer.exe
```

The installer includes the Knight Shift app and handles the Microsoft Edge WebView2 runtime if it is missing. A machine that does not already have WebView2 may need internet access during installation so the runtime bootstrapper can download.

## Development

```sh
pnpm install
pnpm tauri dev
```

## Checks

```sh
pnpm check
cd src-tauri
cargo test
```

## Release Build

```sh
pnpm tauri build
```

The Windows installer is generated under `src-tauri/target/release/bundle/nsis/`.
