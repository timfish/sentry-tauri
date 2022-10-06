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

Next, initialize the Rust half of the plugin:

```rust
fn main() {
    let sentry_options = sentry::ClientOptions {
          dsn:
            "__YOUR_DSN__"
              .into_dsn()
              .expect("failed to parse DSN"),
          release: sentry::release_name!(),
          ..Default::default()
        };

    tauri_plugin_sentry::init(
        |_| sentry::init(init_opts),
        move |sentry_plugin| {
            tauri::Builder::default()
                .plugin(sentry_plugin)
                .run(tauri::generate_context!())
                .expect("error while running tauri application");
        },
    );
}
```

Then you can import and initialize the JavaScript half:

```javascript
import * as Sentry from "tauri-plugin-sentry-api";

Sentry.init();
```

The Tauri Sentry integration supports all options that the regulat `@sentry/browser` SDK does, so you can configure it like you're used to:

```javascript
import { BrowserTracing } from "@sentry/tracing";
import * as Sentry from "tauri-plugin-sentry-api";

Sentry.init({
  integrations: [new BrowserTracing()],

  // Set tracesSampleRate to 1.0 to capture 100%
  // of transactions for performance monitoring.
  // We recommend adjusting this value in production
  tracesSampleRate: 1.0,
});
```

## What is going on here? 🤔

- Includes a `TauriIntegration` that intercepts events and breadcrumbs and passes them to Rust via the Tauri `invoke` API
- Tauri + `serde` + existing Sentry Rust types = Deserialisation mostly Just Works™️
- Uses [`sentry-rust-minidump`](https://github.com/timfish/sentry-rust-minidump) which in turn uses the `minidumper` and `crash-handler` crates to capture minidumps in pure Rust and sends them as attachments using the Sentry Rust SDK

## Points to Note 📝

- There is currently no breadcrumb and scope synchronisation to the crash reporting process so these are missing from minidump events

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
