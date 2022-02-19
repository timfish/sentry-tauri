#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
fn rust_breadcrumb() {
    sentry_tauri::add_breadcrumb(sentry_tauri::Breadcrumb {
        message: Some("This is a breadcrumb from Rust".to_string()),
        ..Default::default()
    })
}

#[tauri::command]
fn rust_panic() {
    panic!("This is a panic from Rust");
}

#[tauri::command]
fn native_crash() {
    unsafe {
        *std::ptr::null_mut() = true;
    }
}

fn main() {
    let (_guard, sentry) = sentry_tauri::init(
        "Tauri Test App",
        "__YOUR_DSN__",
        Some(env!("CARGO_PKG_VERSION")),
    )
    .expect("Could not start Sentry");

    tauri::Builder::default()
        .plugin(sentry)
        .invoke_handler(tauri::generate_handler![
            rust_breadcrumb,
            rust_panic,
            native_crash
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
