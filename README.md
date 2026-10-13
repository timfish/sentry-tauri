# `tauri-plugin-sentry`

A Sentry plugin for Tauri v2.

It's perfectly reasonable to use Sentry's Rust and browser SDKs separately in a
Tauri app. However, this plugin passes browser breadcrumbs and events through
the Rust backend which has a number of advantages:

- Browser events are enriched with Rust, OS and device context
  - Events from both Rust and browser will have the same app and device context
- Breadcrumbs are merged from both the Rust and browser SDKs
  - See what was happening in the Rust backend and the browser frontend at the
    time of the event

## Installation

The `minidump` feature of `tauri-plugin-sentry` is on by default and enables
the `minidump` feature of `sentry`, so you don't need to add `sentry-minidump`
as a dependency.

Add `sentry` and `tauri-plugin-sentry` to dependencies in `Cargo.toml`:

```toml
[dependencies]
sentry = "0.49"
tauri-plugin-sentry = "0.8"
```

Run one of these commands to add the capabilities:

- npm: `npm run tauri add sentry`
- yarn: `yarn run tauri add sentry`
- pnpm: `pnpm tauri add sentry`
- cargo: `cargo tauri add sentry`

however, make sure that you have `sentry:default` in your capabilities:

###### src-tauri/capabilities/\*.json

```json
{
  "$schema": "./../gen/schemas/windows-schema.json",
  "identifier": "main",
  "local": true,
  "windows": ["main"],
  "permissions": [
    "sentry:default" // <- important
  ]
}
```

## Usage

This example also shows usage of
[`sentry-minidump`](https://crates.io/crates/sentry-minidump) which captures
minidumps for native crashes from a separate crash reporter process. It is
available on Linux, macOS and Windows.

```rust
use sentry;
use tauri_plugin_sentry::Sentry;

pub fn run() {
    let sentry_plugin = Sentry::new();

    let options = sentry::ClientOptions::new()
        .add_integration(sentry_plugin.clone())
        .dsn("__YOUR_DSN__")
        .maybe_release(sentry::release_name!())
        .auto_session_tracking(true);

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    let options = options.add_integration(sentry::integrations::minidump::MinidumpIntegration::new());

    // Caution! Everything before here runs in both app and crash reporter processes
    let _guard = sentry::init(options);
    // Everything after here runs in only the app process

    tauri::Builder::default()
        .plugin(sentry_plugin)
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
```

`Sentry` is both a Sentry integration and a Tauri plugin. Add a clone to the
Sentry options before `sentry::init` and pass the original to Tauri. The
integration adds `tauri` and `cargo:tauri-plugin-sentry` to the SDK metadata
of every event, including minidumps from the crash reporter process.

The Plugin:

- By default injects and initialises `@sentry/browser` in every web-view
- Includes custom `transport` and `beforeBreadcrumb` hook that passes events and
  breadcrumbs to the Rust SDK via the Tauri `invoke` API
- Tauri + `serde` + existing Sentry Rust types = Deserialisation mostly Just
  Works™️

## Webview Process Crashes

Webviews run in separate processes, so the minidump integration does not
capture their crashes. The `webview-crashes` feature is on by default. When a
webview process exits, the plugin captures an event with the
`event.process: webview` tag and the webview label, and adds a breadcrumb. For
crashes and out-of-memory exits, it also ends the session with the `crashed`
status.

- **Windows**: Uses the WebView2 `ProcessFailed` event, which gives the
  reason.
- **Linux**: Uses the WebKitGTK `web-process-terminated` signal, which gives
  the reason.
- **macOS and iOS**: WebKit gives no reason and Tauri only exposes this hook
  on the app builder. Call `web_content_process_terminated` from the hook.
  The hook replaces the Tauri default, so reload the webview yourself:

```rust
let builder = tauri::Builder::default();

#[cfg(any(target_os = "macos", target_os = "ios"))]
let builder = builder.on_web_content_process_terminate(|webview| {
    tauri_plugin_sentry::web_content_process_terminated(webview);
    let _ = webview.reload();
});
```

- **Android**: Not supported.

This feature enables the `wry` feature of `tauri` and the `release-health`
feature of `sentry`.

## Custom Sentry Browser Configuration

By default the plugin injects a pre-minified version of `@sentry/browser`. If
you want to configure Sentry in the browser yourself, you can disable the
injection and pass the default config to `Sentry.init`.

Disable automatic injection:

```rust
use tauri_plugin_sentry::{JavaScriptOptions, Options, Sentry};

let sentry_plugin = Sentry::with_options(Options {
    javascript: JavaScriptOptions::no_injection(),
});

// Add `sentry_plugin.clone()` to the Sentry options as above

tauri::Builder::default()
    .plugin(sentry_plugin)
    .run(tauri::generate_context!())
    .expect("error while running tauri app");
```

```ts
import { defaultOptions } from "tauri-plugin-sentry-api";
import * as Sentry from "@sentry/browser";

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
