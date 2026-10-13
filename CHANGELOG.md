# Changelog

## v0.8.0

- feat: Add `tauri_plugin_sentry::Sentry`, which is both a Tauri plugin and a Sentry integration. Add a clone to `sentry::ClientOptions` before `sentry::init` and pass the original to `tauri::Builder::plugin`. The integration adds `tauri` to the SDK integrations and adds the `cargo:tauri-plugin-sentry` package to the SDK metadata of events, which includes minidump events from the crash reporter process. `init`, `init_with_options` and `init_with_no_injection` still work. (#41)
- feat: Add the `cargo:tauri-plugin-sentry` package to the SDK metadata of transactions from the browser (#41)

## v0.7.0

- feat: Replace `sentry-rust-minidump` with `sentry-minidump`. The `minidump` feature now enables the `minidump` feature of `sentry`. Add `sentry::integrations::minidump::MinidumpIntegration` to your `sentry::ClientOptions` instead of calling `tauri_plugin_sentry::minidump::init`. The `tauri_plugin_sentry::minidump` re-export is removed (#40)

## v0.6.0

- feat: Update `sentry` to v0.49 and `sentry-rust-minidump` to v0.17 (#39)
- fix: Forward browser envelopes that `sentry` cannot parse, such as events with source map debug images from `@sentry/vite-plugin`. Before, the plugin dropped these events (#39)
- fix: Generate the permission for the `envelope` command. It had the name `event` before (#37)
- fix: Disable the default features of `tauri` (#34)

## v0.5.0

- feat: Update `sentry` to v0.42, `sentry-rust-minidump` to v0.13 and `@sentry/browser` to v10
- feat: Stop re-exporting `sentry`. Add `sentry` to your own dependencies.
- feat: Remove the features that enabled `sentry` features. Enable them on `sentry` directly.
- feat: Add the `minidump` feature, on by default, which makes `sentry-rust-minidump` optional (#28)

## v0.4.1

- fix: Stop sending envelopes and breadcrumbs to Rust after the first IPC failure. This failure usually comes from missing permissions (#26)

## v0.4.0

- feat: Update `sentry` to v0.36 and `sentry-rust-minidump` to v0.10
- feat: Add features that enable the matching `sentry` features (#23)
- fix: Don't capture breadcrumbs for IPC requests to `https://ipc.localhost`, because this causes infinite loops (#24)

## v0.3.1

- fix: Disable `minidumper` on iOS because it is not compatible (#19)

## v0.3.0

- feat: Update `sentry` to v0.35 and `sentry-rust-minidump` to v0.9 (#18)
- fix: Detect IPC requests to `http://ipc.localhost`, which Windows uses (#16)

## v0.2.0

- feat: Send complete envelopes from the browser to Rust with a custom transport. This adds support for attachments and replays. You must now pass `&client` to the plugin `init` function (#15)

## v0.1.1

- fix: README (#14)

## v0.1.0

- First release as `tauri-plugin-sentry`, with support for Tauri v2 (#13)

## `sentry-tauri` (Tauri v1)

Before v0.1.0 of `tauri-plugin-sentry`, this repository published the `sentry-tauri` crate for Tauri v1.

### v0.3.0

- feat: Update `sentry` to v0.32 and `sentry-rust-minidump` to v0.7 (#9)

### v0.2.0

- feat: Update `sentry` to v0.31 and `sentry-rust-minidump` to v0.6

### v0.1.0

- First release, with `sentry` v0.27 and native crash capture through `sentry-rust-minidump`
