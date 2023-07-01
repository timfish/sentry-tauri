#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use sentry_tauri::sentry;

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

fn main() {
    let client = sentry::init((
        "https://233a45e5efe34c47a3536797ce15dafa@o447951.ingest.sentry.io/5650507",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            debug: true,
            ..Default::default()
        },
    ));

    let _guard = sentry_tauri::minidump::init(&client);

    tauri::Builder::default()
        .plugin(sentry_tauri::plugin())
        .invoke_handler(tauri::generate_handler![
            rust_breadcrumb,
            rust_panic,
            native_crash
        ])
        .run(tauri::generate_context!())
        .expect("error while starting tauri app");
}
