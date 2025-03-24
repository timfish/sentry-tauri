use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use sentry;
#[cfg(all(not(target_os = "ios"), feature = "minidump"))]
pub use sentry_rust_minidump as minidump;

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
        plugin_builder = plugin_builder.js_init_script(
            include_str!("../dist/inject.min.js")
                .replace("__DEBUG__", &format!("{}", options.javascript.debug)),
        );
    };

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
