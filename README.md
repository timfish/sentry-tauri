# `sentry-tauri`

A Sentry plugin for Tauri v2.

It's perfectly reasonable to use Sentry's Rust and browser SDKs separately in a
Tauri app. However, this plugin passes browser breadcrumbs and events through 
the Rust backend which has a number of advantages:

- Browser events are enriched with Rust, OS and device context
  - Events from both Rust and browser will have the same context for filtering
- Breadcrumbs are merged from both the Rust and browser SDKs
  - See what was happening in the Rust backend and the browser frontend at the
    time of the event

## Installation

This example also shows usage of
[`sentry_rust_minidump`](https://github.com/timfish/sentry-rust-minidump) which
allows you to capture minidumps for native crashes from a separate crash
reporting process.

Add the required dependencies in `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-sentry = { git = "https://github.com/timfish/sentry-tauri", branch = "v2" }
```
`sentry` and `sentry-rust-minidump` are re-exported by `sentry-tauri` so you
don't need to add them as dependencies.

Run one of these commands to add the capabilities:
- npm: `npm run tauri add sentry`
- yarn: `yarn run tauri add sentry`
- pnpm: `pnpm tauri add sentry`
- cargo: `cargo tauri add sentry`

however, make sure that you have `sentry:default` in your capabilities:

###### src-tauri/capabilities/*.json
```json
{
  "$schema": "./../gen/schemas/windows-schema.json",
  "identifier": "main",
  "local": true,
  "windows": [
    "main"
  ],
  "permissions": [
    "sentry:default" // <- important
  ]
}
```

Also make sure that you are using the `sentry` as the example below. keep in mind that it's just an example and you are free to use the `sentry` package (that is just re-exported from this package) anyway you want.

```rust
// src/lib.rs

pub fn run() {
    let client = tauri_plugin_sentry::sentry::init((
        "__YOUR_DSN__",
        tauri_plugin_sentry::sentry::ClientOptions {
            release: tauri_plugin_sentry::sentry::release_name!(),
            ..Default::default()
        },
    ));

    // Everything before here runs in both app and crash reporter processes
    let _guard = tauri_plugin_sentry::minidump::init(&client);
    // Everything after here runs in only the app process

    tauri::Builder::default()
        .plugin(tauri_plugin_sentry::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
```

## The Plugin

- By default injects and initialises `@sentry/browser` in every web-view
- Includes `beforeSend` and `beforeBreadcrumb` hooks that intercept events and breadcrumbs and passes
  them to the Rust SDK via the Tauri `invoke` API
- Tauri + `serde` + existing Sentry Rust types = Deserialisation mostly Just
  Works™️

## Custom Sentry Browser Configuration

By default the plugin injects a pre-minified version of `@sentry/browser`. If
you want to configure Sentry in the browser yourself, you can disable the
injection and pass the default config to `Sentry.init`.

Disable automatic injection:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_sentry::init_with_no_injection())
    .run(tauri::generate_context!())
    .expect("error while running tauri app");
```

```ts
import { defaultOptions } from 'tauri-plugin-sentry-api';
import * as Sentry from '@sentry/browser';

Sentry.init({
  ...defaultOptions,
  /**
   * Your custom configuration here
   */
});
```

## Example App

Clone this repository and install dependencies:

```shell
> yarn install
```

In `examples/basic-app/src-tauri/src/main.rs` replace the DSN with your DSN

Run in development mode:

```shell
> yarn example
```
