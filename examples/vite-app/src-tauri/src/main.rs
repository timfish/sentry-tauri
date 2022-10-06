#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use sentry::IntoDsn;

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
    #[allow(deref_nullptr)]
    unsafe {
        *std::ptr::null_mut() = true;
    }
}

fn main() {
    let sentry_options = sentry::ClientOptions {
        dsn: "https://cd1169c0f3334d53b97db60d1ca1ac01@o4503930527088640.ingest.sentry.io/4503930528399360".into_dsn().expect("failed to parse DSN"),
        release: sentry::release_name!(),
        ..Default::default()
    };

    sentry_tauri::init(
        |_| sentry::init(sentry_options),
        |sentry_plugin| {
            tauri::Builder::default()
                .plugin(sentry_plugin)
                .invoke_handler(tauri::generate_handler![rust_breadcrumb, rust_panic, native_crash])
                .run(tauri::generate_context!())
                .expect("error while running tauri application");
        },
    );
}
