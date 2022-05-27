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
    sentry_tauri::init(
        sentry::release_name!(),
        |_| {
            sentry::init((
                "__YOUR_DSN__",
                sentry::ClientOptions {
                    release: sentry::release_name!(),
                    ..Default::default()
                },
            ))
        },
        |sentry_plugin| {
            tauri::Builder::default()
                .plugin(sentry_plugin)
                .invoke_handler(tauri::generate_handler![
                    rust_breadcrumb,
                    rust_panic,
                    native_crash
                ])
                .run(tauri::generate_context!())
                .expect("error while running tauri application");
        },
    );
}
