use std::borrow::Cow;

use sentry::ClientOptions;
use serde::Serialize;
use tauri::{
  plugin::{self, TauriPlugin},
  Runtime,
};

#[derive(Debug, Serialize)]
struct JavaScriptInit<'a> {
  dsn: String,
  release: Option<Cow<'a, str>>,
  environment: Option<Cow<'a, str>>,
  debug: bool,
}

impl<'a> From<ClientOptions> for JavaScriptInit<'a> {
  fn from(opts: ClientOptions) -> Self {
    Self {
      dsn: opts.dsn.expect("A DSN must be configured").to_string(),
      release: opts.release,
      environment: opts.environment,
      debug: opts.debug,
    }
  }
}

pub fn init<R: Runtime>(options: sentry::ClientOptions) -> TauriPlugin<R> {
  let mut plugin_builder = plugin::Builder::new("sentry");

  let js_init = JavaScriptInit::from(options);

  let js_init_script = format!(
    "window.__SENTRY_INIT__ = JSON.parse({:?})",
    serde_json::to_string(&js_init).unwrap()
  );

  plugin_builder = plugin_builder.js_init_script(js_init_script);

  plugin_builder.build()
}
