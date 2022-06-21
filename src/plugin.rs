use crate::options::Options;
use sentry::{add_breadcrumb, capture_event, protocol::Event, Breadcrumb};
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime, Window,
};

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

pub fn build<R: Runtime>(options: &Options) -> TauriPlugin<R> {
    let mut plugin_builder =
        Builder::new("sentry").invoke_handler(generate_handler![event, breadcrumb]);

    if options.init_javascript_sdk {
        plugin_builder = plugin_builder.js_init_script(
            include_str!("../webview-dist/inject.min.js")
                .replace("__DEBUG__", &format!("{}", options.sentry_debug)),
        );
    }

    plugin_builder.build()
}
