use std::time::Duration;

use tauri::{webview::PageLoadEvent, AppHandle, RunEvent, State, WebviewWindowBuilder};
use tauri_plugin_sentry::{JavaScriptOptions, Options, Sentry};

mod webview_crash;

struct Scenario(String);

#[tauri::command]
fn scenario(scenario: State<'_, Scenario>) -> String {
    println!("[e2e] frontend asked for scenario '{}'", scenario.0);
    scenario.0.clone()
}

#[tauri::command]
fn rust_breadcrumb() {
    sentry::add_breadcrumb(sentry::Breadcrumb {
        message: Some("rust breadcrumb".into()),
        ..Default::default()
    });
}

#[tauri::command]
fn rust_panic() {
    panic!("rust panic");
}

#[tauri::command]
fn native_crash() {
    unsafe { sadness_generator::raise_segfault() }
}

#[tauri::command]
fn exit(app: AppHandle) {
    if let Some(client) = sentry::Hub::main().client() {
        client.flush(Some(Duration::from_secs(5)));
    }
    app.exit(0);
}

fn main() {
    let name = std::env::var("SENTRY_E2E_SCENARIO").unwrap_or_default();
    let dsn = std::env::var("SENTRY_E2E_DSN").expect("SENTRY_E2E_DSN must be set");

    // The `transaction` scenario starts the browser SDK itself so it can
    // turn on tracing.
    let javascript = if name == "transaction" {
        JavaScriptOptions::no_injection()
    } else {
        JavaScriptOptions::default()
    };
    let sentry_plugin = Sentry::with_options(Options { javascript });

    let options = sentry::ClientOptions::new()
        .add_integration(sentry_plugin.clone())
        .dsn(&dsn)
        .release("sentry-tauri-e2e@1.0.0")
        .environment("e2e")
        .debug(true);

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    let options =
        options.add_integration(sentry::integrations::minidump::MinidumpIntegration::new());

    let _guard = sentry::init(options);

    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.on_web_content_process_terminate(|webview| {
        tauri_plugin_sentry::web_content_process_terminated(webview);
        let _ = webview.reload();
    });

    builder
        .plugin(sentry_plugin)
        .manage(Scenario(name))
        .on_page_load(|_, payload| {
            let event = match payload.event() {
                PageLoadEvent::Started => "started",
                PageLoadEvent::Finished => "finished",
            };
            println!("[e2e] page load {event}: {}", payload.url());
        })
        .invoke_handler(tauri::generate_handler![
            scenario,
            rust_breadcrumb,
            rust_panic,
            native_crash,
            webview_crash::crash_webview,
            exit
        ])
        .setup(|app| {
            println!("[e2e] setup");
            let config = app.config().app.windows[0].clone();
            WebviewWindowBuilder::from_config(app.handle(), &config)?.build()?;
            println!("[e2e] main window created");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri app")
        .run(|_, event| {
            if let RunEvent::Ready = event {
                println!("[e2e] event loop ready");
            }
        });
}
