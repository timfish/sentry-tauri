mod plugin;

pub fn init<'a, R, SentryInitFn, RunAppFn>(init_sentry: SentryInitFn, run_app: RunAppFn)
where
  R: tauri::Runtime,
  SentryInitFn: FnOnce(bool) -> sentry::ClientInitGuard,
  RunAppFn: FnOnce(tauri::plugin::TauriPlugin<R>),
{
  let sentry = init_sentry(false);

  let options = sentry.options().clone();

  sentry_rust_minidump::init(
    options.release.clone(),
    |_| sentry,
    || {
      run_app(plugin::init(options));
    },
  )
}
