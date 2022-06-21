mod options;
mod plugin;

pub fn init<Runtime, IntoOpt, SentryInitFn, RunAppFn>(
    options: IntoOpt,
    init_sentry: SentryInitFn,
    run_app: RunAppFn,
) where
    Runtime: tauri::Runtime,
    IntoOpt: Into<options::Options>,
    SentryInitFn: FnOnce(bool) -> sentry::ClientInitGuard,
    RunAppFn: FnOnce(tauri::plugin::TauriPlugin<Runtime>),
{
    let options = options.into();

    sentry_rust_minidump::init(Some(&options.release), init_sentry, || {
        run_app(plugin::build(&options));
    })
}
