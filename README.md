# Knight Shift

Knight Shift is a small Tauri + SvelteKit prototype about earning Influence from real-world keyboard and mouse activity.

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
