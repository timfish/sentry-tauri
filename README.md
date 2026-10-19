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
- Events show where they came from with the `event.process` tag
  - `rust` for Rust events, panics and minidumps, `webview` for browser events
    and transactions
  - Browser events and transactions also get the `tauri.webview` and
    `tauri.window` tags with the labels of the webview and window

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

## Cargo features

All features are on by default:

- `minidump`: enables the `minidump` feature of `sentry` to capture native
  crashes
- `wry`: enables the `wry` feature of `tauri` to add the webview version to
  the `browser` context
- `webview-crashes`: captures webview process crashes. See
  [Webview Process Crashes](#webview-process-crashes). It enables `wry` and
  the `release-health` feature of `sentry`

If your app does not use the wry runtime, turn off the default features and
enable only `minidump`:

```toml
tauri-plugin-sentry = { version = "0.8", default-features = false, features = ["minidump"] }
```

This also turns off `webview-crashes`. Without `wry`, events have no
`browser` context with the webview version.

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

The integration also adds this context to events from Rust and the browser:

- `app`: name, version, identifier, start time, architecture, whether a
  window has focus (`in_foreground`) and the build type (`app-store` for Mac
  App Store builds, `windows-store` for packaged Windows apps)
- `runtime`: Tauri and its version
- `browser`: the webview (WebView2, WKWebView, WebKitGTK or Android System
  WebView) and its version
- `device`: the resolution and scale factor of the primary monitor

The webview version needs the `wry` feature. See
[Cargo features](#cargo-features).

The Plugin:

- By default injects and initialises `@sentry/browser` in every web-view
- Includes custom `transport` and `beforeBreadcrumb` hook that passes events and
  breadcrumbs to the Rust SDK via the Tauri `invoke` API
- Sends the user, tags, extra and contexts that you set in the browser to the
  Rust SDK, so Rust panics and events include them too
- Tauri + `serde` + existing Sentry Rust types = Deserialisation mostly Just
  Works™️
- Adds breadcrumbs for Tauri app, window and webview events

## Breadcrumbs for Tauri Events

The plugin adds breadcrumbs with the category `tauri` for app, window and
webview events. The message has the form `app.ready`, `window.focus` or
`webview.page-load-finished`, and the data includes the window or webview
label. Navigation and page load breadcrumbs include the URL.

By default, events that fire at a high rate (`resize`, `move`, `drag-enter`,
`drag-over` and `drag-leave`) do not create breadcrumbs. Window titles are not
captured unless you enable `capture_window_titles`.

```rust
use tauri_plugin_sentry::{BreadcrumbOptions, EventFilter, Options, Sentry};

let sentry_plugin = Sentry::with_options(Options {
    breadcrumbs: BreadcrumbOptions {
        // No breadcrumbs for app events
        app: EventFilter::None,
        // Only these window events
        window: EventFilter::Only(vec!["focus".into(), "blur".into()]),
        // All webview events
        webview: EventFilter::All,
        capture_window_titles: true,
    },
    ..Default::default()
});
```

Use `BreadcrumbOptions::disabled()` to turn off all of these breadcrumbs.

## Webview Process Crashes

Webviews run in separate processes, so the minidump integration does not
capture their crashes. With the `webview-crashes` feature, when a webview
process exits, the plugin captures an event with the `event.process: webview`,
`tauri.webview` and `tauri.window` tags, and adds a breadcrumb. For crashes
and out-of-memory exits, it also ends the session with the `crashed` status.

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

## Custom Sentry Browser Configuration

By default the plugin injects a pre-minified version of `@sentry/browser`. If
you want to configure Sentry in the browser yourself, you can disable the
injection and pass the default config to `Sentry.init`.

Disable automatic injection:

```rust
use tauri_plugin_sentry::{JavaScriptOptions, Options, Sentry};

let sentry_plugin = Sentry::with_options(Options {
    javascript: JavaScriptOptions::no_injection(),
    ..Default::default()
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
