use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Runtime,
};

pub use sentry;
pub use sentry_rust_minidump as minidump;

#[derive(Debug, Clone)]
pub struct JavaScriptOptions {
    inject: bool,
    debug: bool,
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

pub fn init_with_options<R: Runtime>(options: Options) -> TauriPlugin<R> {
    let mut plugin_builder = Builder::<R>::new("sentry")
        .invoke_handler(generate_handler![commands::event, commands::breadcrumb]);

    if options.javascript.inject {
        plugin_builder = plugin_builder.js_init_script(
            include_str!("../dist/inject.min.js")
                .replace("__DEBUG__", &format!("{}", options.javascript.debug)),
        );
    };

    plugin_builder.build()
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    init_with_options(Default::default())
}
