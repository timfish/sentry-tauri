use std::{borrow::Cow, sync::Arc};

use sentry::{
    protocol::{ClientSdkPackage, Event},
    ClientOptions, Hub,
};
use tauri::{
    generate_handler,
    ipc::Invoke,
    plugin::{Builder, Plugin, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[derive(Debug, Clone)]
pub struct JavaScriptOptions {
    pub inject: bool,
    pub debug: bool,
}

impl JavaScriptOptions {
    pub fn no_injection() -> Self {
        Self {
            inject: false,
            ..Default::default()
        }
    }
}

impl Default for JavaScriptOptions {
    fn default() -> Self {
        Self {
            inject: true,
            #[cfg(not(debug_assertions))]
            debug: false,
            #[cfg(debug_assertions)]
            debug: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub javascript: JavaScriptOptions,
}

mod commands;

fn js_init_script(options: &JavaScriptOptions) -> String {
    include_str!("../dist/inject.min.js").replace("__DEBUG__", &format!("{}", options.debug))
}

/// A Tauri plugin that is also a Sentry integration.
///
/// Add a clone to the Sentry `ClientOptions` before `sentry::init` and
/// pass the original to `tauri::Builder::plugin`.
#[derive(Debug, Clone, Default)]
pub struct Sentry {
    options: Arc<Options>,
}

impl Sentry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(options: Options) -> Self {
        Self {
            options: Arc::new(options),
        }
    }
}

impl sentry::Integration for Sentry {
    fn name(&self) -> &'static str {
        "tauri"
    }

    fn process_event(
        &self,
        mut event: Event<'static>,
        _options: &ClientOptions,
    ) -> Option<Event<'static>> {
        if let Some(sdk) = event.sdk.as_mut().map(Cow::to_mut) {
            sdk.packages.push(ClientSdkPackage {
                name: "cargo:tauri-plugin-sentry".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            });
        }
        Some(event)
    }
}

impl<R: Runtime> Plugin<R> for Sentry {
    fn name(&self) -> &'static str {
        "sentry"
    }

    fn initialize(
        &mut self,
        app: &AppHandle<R>,
        _config: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = Hub::main()
            .client()
            .ok_or("sentry::init must be called before the Sentry plugin is initialized")?;
        app.manage((*client).clone());
        Ok(())
    }

    fn initialization_script(&self) -> Option<String> {
        self.options
            .javascript
            .inject
            .then(|| js_init_script(&self.options.javascript))
    }

    fn extend_api(&mut self, invoke: Invoke<R>) -> bool {
        let handler: fn(Invoke<R>) -> bool =
            generate_handler![commands::breadcrumb, commands::envelope];
        handler(invoke)
    }
}

pub fn init_with_options<R: Runtime>(
    sentry_client: &sentry::Client,
    options: Options,
) -> TauriPlugin<R> {
    let sentry_client = sentry_client.clone();

    let mut plugin_builder = Builder::<R>::new("sentry")
        .invoke_handler(generate_handler![commands::breadcrumb, commands::envelope])
        .setup(move |app, _| {
            app.manage(sentry_client);
            Ok(())
        });

    if options.javascript.inject {
        plugin_builder = plugin_builder.js_init_script(js_init_script(&options.javascript));
    }

    plugin_builder.build()
}

pub fn init<R: Runtime>(sentry_client: &sentry::Client) -> TauriPlugin<R> {
    init_with_options(sentry_client, Default::default())
}

pub fn init_with_no_injection<R: Runtime>(sentry_client: &sentry::Client) -> TauriPlugin<R> {
    init_with_options(
        sentry_client,
        Options {
            javascript: JavaScriptOptions::no_injection(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_adds_sdk_metadata() {
        let options = ClientOptions::new().add_integration(Sentry::new());
        let events = sentry::test::with_captured_events_options(
            || {
                sentry::capture_message("test", sentry::Level::Info);
            },
            options,
        );

        let sdk = events[0].sdk.as_ref().unwrap();
        assert!(sdk.integrations.iter().any(|i| i == "tauri"));
        assert!(sdk
            .packages
            .iter()
            .any(|p| p.name == "cargo:tauri-plugin-sentry"));
    }
}
