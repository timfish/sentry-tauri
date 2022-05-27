use sentry::{protocol::Event, *};
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime, Window,
};

pub use sentry::{add_breadcrumb, capture_error, Breadcrumb};

#[tauri::command]
fn event<R: Runtime>(_app: AppHandle<R>, _window: Window<R>, mut event: Event<'static>) {
    event.platform = "javascript".into();
    // Add other things. Window context?
    capture_event(event);
}

#[tauri::command]
fn breadcrumb<R: Runtime>(_app: AppHandle<R>, _window: Window<R>, breadcrumb: Breadcrumb) {
    // Differentiate between JavaScript and Rust breadcrumbs?
    add_breadcrumb(breadcrumb);
}

pub fn init<Runtime, Release, SentryInitFn, RunAppFn>(
    release: Option<Release>,
    init_sentry: SentryInitFn,
    run_app: RunAppFn,
) where
    Runtime: tauri::Runtime,
    Release: Into<String>,
    SentryInitFn: FnOnce(bool) -> sentry::ClientInitGuard,
    RunAppFn: FnOnce(TauriPlugin<Runtime>),
{
    let javascript = include_str!("../webview-dist/index.min.js");

    #[cfg(not(debug_assertions))]
    let javascript = javascript.replace("\"{{debug}}\"", "false");
    #[cfg(debug_assertions)]
    let javascript = javascript.replace("\"{{debug}}\"", "true");

    let plugin = Builder::new("sentry")
        .js_init_script(javascript)
        .invoke_handler(generate_handler![event, breadcrumb])
        .build();

    let run_app_wrap = || {
        run_app(plugin);
    };

    sentry_rust_minidump::init(release, init_sentry, run_app_wrap)
}
