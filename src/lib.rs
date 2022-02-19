use sentry::{protocol::Event, transports::DefaultTransportFactory, types::ParseDsnError, *};
use sentry_contrib_breakpad::{Error as BreakpadError, *};
use std::sync::Arc;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime, Window,
};
use thiserror::Error;

pub use sentry::{add_breadcrumb, capture_error, Breadcrumb};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing DSN: {0}")]
    ParseDsnError(#[from] ParseDsnError),
    #[error("Breakpad error: {0}")]
    BreakpadError(#[from] BreakpadError),
}

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

pub fn init<R: Runtime>(
    dsn: &str,
    release: Option<&str>,
) -> Result<((ClientInitGuard, BreakpadIntegration), TauriPlugin<R>), Error> {
    let transport = BreakpadTransportFactory::new(
        CrashSendStyle::SendNextSession,
        Arc::new(DefaultTransportFactory {}),
    );

    let sentry_options = ClientOptions {
        #[cfg(debug_assertions)]
        debug: true,
        dsn: dsn.into_dsn()?,
        release: release.map(|r| r.to_string().into()),
        transport: Some(Arc::new(transport)),
        ..Default::default()
    };

    let sentry_guard = sentry::init(sentry_options);

    let breakpad_guard = BreakpadIntegration::new(
        "C:\\Users\\tim\\Documents\\Repositories\\tauri-test-app",
        InstallOptions::BothHandlers,
        Hub::current(),
    )?;

    let javascript = include_str!("../webview-dist/index.min.js");

    #[cfg(not(debug_assertions))]
    let javascript = javascript.replace("\"{{debug}}\"", "false");
    #[cfg(debug_assertions)]
    let javascript = javascript.replace("\"{{debug}}\"", "true");

    Ok((
        (sentry_guard, breakpad_guard),
        Builder::new("sentry")
            .js_init_script(javascript)
            .invoke_handler(generate_handler![event, breadcrumb])
            .build(),
    ))
}
