## v0.5.0

- feat: Update `sentry` to v0.42 and `sentry-rust-minidump` to v0.13
- feat: Stop re-exporting `sentry` from `tauri-plugin-sentry`

## v0.4.0

- fix: Don't capture breadcrumbs for IPC messages because this results in infinite loops.

## v0.3.1

- fix: Disable `minidumper` on iOS since it's not compatible (#19)

## v0.3.0

- Update to Sentry v0.35.0
- Fix IPC check on Windows (#16)

## v0.2.1

- Fix README.md

## v0.2.0

- Use a transport and pass complete envelopes to the Rust process. This allows
  us to support attachments and Replays. You';; now need to pass the `&client`
  to the plugin `init` function.

## v0.1.1

- Fix README.md

## v0.1.0

- First release supporting Tauri v2
