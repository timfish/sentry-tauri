#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use sentry;
use tauri_plugin_sentry;

#[tauri::command]
fn rust_breadcrumb() {
    sentry::add_breadcrumb(sentry::Breadcrumb {
        message: Some("This is a breadcrumb from Rust".to_owned()),
        ..Default::default()
    })
}

#[tauri::command]
fn rust_panic() {
    panic!("This is a panic from Rust");
}

#[tauri::command]
fn native_crash() {
    unsafe { sadness_generator::raise_segfault() }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let options = sentry::ClientOptions::new()
        .dsn("https://233a45e5efe34c47a3536797ce15dafa@o447951.ingest.sentry.io/5650507")
        .maybe_release(sentry::release_name!())
        .debug(true);

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    let options = options.add_integration(sentry::integrations::minidump::MinidumpIntegration::new());

    let client = sentry::init(options);

    tauri::Builder::default()
        .plugin(tauri_plugin_sentry::init(&client))
        .invoke_handler(tauri::generate_handler![
            rust_breadcrumb,
            rust_panic,
            native_crash
        ])
        .run(tauri::generate_context!())
        .expect("error while starting tauri app");
}
