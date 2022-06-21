# `sentry-tauri`

An experimental Tauri Plugin for Sentry.

Captures and reports to Sentry:

- JavaScript errors and breadcrumbs in Tauri windows using `@sentry/browser`
- Panics and breadcrumbs in Rust via the Sentry Rust SDK
- Native crashes as minidumps

## Installation

Add the plugin as a dependency in `Cargo.toml`:

```toml
[dependencies]
sentry = "0.27"
sentry-tauri = "0.1"
```

```rust
fn main() {
  sentry_tauri::init(
      sentry::release_name!(),
      |_| {
          sentry::init((
              "__YOUR_DSN__",
              sentry::ClientOptions {
                  release: sentry::release_name!(),
                  ..Default::default()
              },
          ))
      },
      |sentry_plugin| {
          tauri::Builder::default()
              .plugin(sentry_plugin)
              .run(tauri::generate_context!())
              .expect("error while running tauri application");
      },
  );
}
```

## What is going on here? 🤔

- Injects and initialises `@sentry/browser` in every web-view
- Includes a `TauriIntegration` that intercepts events and breadcrumbs and passes
  them to Rust via the Tauri `invoke` API
- Tauri + `serde` + existing Sentry Rust types = Deserialisation mostly Just Works™️
- Uses [`sentry-rust-minidump`](https://github.com/timfish/sentry-rust-minidump)
  which in turn uses the `minidumper` and `crash-handler` crates to capture
  minidumps in pure Rust and sends them as attachments using the Sentry Rust SDK

## Points to Note 📝

- There is currently no breadcrumb and scope synchronisation to the crash
  reporting process so these are missing from minidump events

## Example App

Clone this repository and install dependencies:

```shell
> cd examples/basic-app
> yarn install
```

In `examples/basic-app/src-tauri/src/main.rs` replace `__YOUR_DSN__` with your DSN

Run in development mode:

```shell
> yarn tauri dev
```
