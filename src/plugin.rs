use std::borrow::Cow;

use sentry::{ClientOptions, protocol::Event, Breadcrumb, capture_event, add_breadcrumb};
use serde::Serialize;
use tauri::{
  plugin::{self, TauriPlugin},
  Runtime,
};

#[tauri::command]
fn event(mut event: Event<'static>) {
    event.platform = "javascript".into();
    // Add other things. Window context?
    capture_event(event);
}

#[tauri::command]
fn breadcrumb(breadcrumb: Breadcrumb) {
    // Differentiate between JavaScript and Rust breadcrumbs?
    add_breadcrumb(breadcrumb);
}

pub fn init<R: Runtime>(options: sentry::ClientOptions) -> TauriPlugin<R> {
  let js_init = JavaScriptInit::from(options);

  let js_init_script = format!(
    "window.__SENTRY_INIT__ = JSON.parse({:?})",
    serde_json::to_string(&js_init).unwrap()
  );

  plugin::Builder::new("sentry")
    .invoke_handler(tauri::generate_handler![event, breadcrumb])
    .js_init_script(js_init_script)
    .build()
}

#[derive(Debug, Serialize)]
struct JavaScriptInit<'a> {
  dsn: String,
  release: Option<Cow<'a, str>>,
  environment: Option<Cow<'a, str>>,
  debug: bool,
}

impl<'a> From<ClientOptions> for JavaScriptInit<'a> {
  fn from(opts: ClientOptions) -> Self {
    let dsn = opts
      .dsn
      .expect("A DSN must be configured")
      .to_string()
      // to_string() on DSN produces an invalid DSN string, we strip the superflous colons here.
      // see for details https://github.com/getsentry/sentry-rust/issues/505
      .replace(":@", "@");

    Self {
      dsn,
      release: opts.release,
      environment: opts.environment,
      debug: opts.debug,
    }
  }
}