use std::{
    collections::{btree_map::Entry, HashSet},
    sync::{Arc, Mutex, OnceLock, PoisonError},
    time::SystemTime,
};

use sentry::{
    protocol::{AppContext, BrowserContext, Context, Event, Map, RuntimeContext, Value},
    Hub,
};
use tauri::{AppHandle, RunEvent, Runtime, WindowEvent};

#[derive(Debug, Clone, PartialEq)]
struct Screen {
    resolution: String,
    density: f64,
}

/// App, runtime, webview and screen context for events.
///
/// The plugin fills it in the app process. The integration and a main hub
/// event processor merge it into events. The crash reporter process gets
/// the processor output with the synced scope.
#[derive(Debug)]
pub(crate) struct TauriContext {
    start_time: SystemTime,
    app: OnceLock<AppContext>,
    webview: OnceLock<BrowserContext>,
    screen: Mutex<Option<Screen>>,
    /// Unset until the first focus event.
    focused: Mutex<Option<HashSet<String>>>,
}

impl Default for TauriContext {
    fn default() -> Self {
        Self {
            start_time: SystemTime::now(),
            app: OnceLock::new(),
            webview: OnceLock::new(),
            screen: Mutex::new(None),
            focused: Mutex::new(None),
        }
    }
}

impl TauriContext {
    pub(crate) fn initialize<R: Runtime>(self: &Arc<Self>, app: &AppHandle<R>) {
        let info = app.package_info();
        let mut other = Map::new();
        other.insert("app_arch".into(), std::env::consts::ARCH.into());
        let _ = self.app.set(AppContext {
            app_name: Some(info.name.clone()),
            app_version: Some(info.version.to_string()),
            app_identifier: Some(app.config().identifier.clone()),
            app_start_time: Some(self.start_time),
            build_type: build_type(),
            other,
            ..Default::default()
        });

        #[cfg(feature = "wry")]
        if let Some(name) = webview_name() {
            let context = self.clone();
            // On Android this waits for an activity and the main thread.
            let _ = std::thread::Builder::new()
                .name("sentry-webview-version".into())
                .spawn(move || {
                    if let Ok(version) = tauri::webview_version() {
                        let _ = context.webview.set(BrowserContext {
                            name: Some(name.into()),
                            version: Some(version),
                            ..Default::default()
                        });
                    }
                });
        }

        // The minidump integration sends the scope of the main hub to the
        // crash reporter, which has no Tauri app.
        let context = self.clone();
        Hub::main().configure_scope(|scope| {
            scope.add_event_processor(move |mut event| {
                context.apply(&mut event);
                Some(event)
            })
        });
    }

    pub(crate) fn on_event<R: Runtime>(&self, app: &AppHandle<R>, event: &RunEvent) {
        match event {
            // The primary monitor is not reliable before the app is ready.
            RunEvent::Ready => self.update_screen(app),
            RunEvent::WindowEvent { label, event, .. } => match event {
                WindowEvent::Focused(focused) => self.set_focused(label, *focused),
                WindowEvent::Destroyed => self.set_focused(label, false),
                WindowEvent::ScaleFactorChanged { .. } => self.update_screen(app),
                _ => {}
            },
            _ => {}
        }
    }

    fn update_screen<R: Runtime>(&self, app: &AppHandle<R>) {
        if let Ok(Some(monitor)) = app.primary_monitor() {
            let size = monitor.size();
            *self.screen.lock().unwrap_or_else(PoisonError::into_inner) = Some(Screen {
                resolution: format!("{}x{}", size.width, size.height),
                density: monitor.scale_factor(),
            });
        }
    }

    fn set_focused(&self, label: &str, focused: bool) {
        let mut guard = self.focused.lock().unwrap_or_else(PoisonError::into_inner);
        let windows = guard.get_or_insert_with(HashSet::new);
        if focused {
            windows.insert(label.to_string());
        } else {
            windows.remove(label);
        }
    }

    fn in_foreground(&self) -> Option<bool> {
        self.focused
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map(|windows| !windows.is_empty())
    }

    /// Adds the values that the event does not have yet.
    pub(crate) fn apply(&self, event: &mut Event<'static>) {
        let contexts = &mut event.contexts;

        let mut app = self.app.get().cloned();
        if let (Some(app), Some(in_foreground)) = (app.as_mut(), self.in_foreground()) {
            app.other
                .insert("in_foreground".into(), Value::Bool(in_foreground));
        }
        if let Some(app) = app {
            match contexts.entry("app".into()) {
                Entry::Vacant(entry) => {
                    entry.insert(Context::App(Box::new(app)));
                }
                Entry::Occupied(mut entry) => {
                    if let Context::App(existing) = entry.get_mut() {
                        merge_app(existing, app);
                    }
                }
            }
        }

        contexts.entry("runtime".into()).or_insert_with(|| {
            Context::Runtime(Box::new(RuntimeContext {
                name: Some("Tauri".into()),
                version: Some(tauri::VERSION.into()),
                ..Default::default()
            }))
        });

        if let Some(webview) = self.webview.get() {
            contexts
                .entry("browser".into())
                .or_insert_with(|| Context::Browser(Box::new(webview.clone())));
        }

        // Only merge into the device context that the Rust context
        // integration adds. Adding it here would stop that integration.
        let screen = self
            .screen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let (Some(screen), Some(Context::Device(device))) = (screen, contexts.get_mut("device"))
        {
            device
                .other
                .entry("screen_resolution".to_string())
                .or_insert_with(|| screen.resolution.into());
            device
                .other
                .entry("screen_density".to_string())
                .or_insert_with(|| screen.density.into());
        }
    }
}

fn merge_app(existing: &mut AppContext, app: AppContext) {
    fn fill<T>(field: &mut Option<T>, value: Option<T>) {
        if field.is_none() {
            *field = value;
        }
    }

    fill(&mut existing.app_name, app.app_name);
    fill(&mut existing.app_version, app.app_version);
    fill(&mut existing.app_identifier, app.app_identifier);
    fill(&mut existing.app_start_time, app.app_start_time);
    fill(&mut existing.build_type, app.build_type);
    for (key, value) in app.other {
        existing.other.entry(key).or_insert(value);
    }
}

#[cfg(feature = "wry")]
fn webview_name() -> Option<&'static str> {
    if cfg!(windows) {
        Some("WebView2")
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        Some("WKWebView")
    } else if cfg!(target_os = "android") {
        Some("Android System WebView")
    } else if cfg!(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )) {
        Some("WebKitGTK")
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn build_type() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let contents = exe.parent()?.parent()?;
    contents
        .join("_MASReceipt")
        .join("receipt")
        .exists()
        .then(|| "app-store".into())
}

#[cfg(windows)]
fn build_type() -> Option<String> {
    use windows_sys::Win32::{
        Foundation::ERROR_INSUFFICIENT_BUFFER, Storage::Packaging::Appx::GetCurrentPackageFullName,
    };

    let mut length = 0;
    // SAFETY: With a zero length and a null buffer, the call only writes
    // the needed length.
    let result = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
    // Without a package identity the call returns
    // APPMODEL_ERROR_NO_PACKAGE.
    (result == ERROR_INSUFFICIENT_BUFFER).then(|| "windows-store".into())
}

#[cfg(not(any(target_os = "macos", windows)))]
fn build_type() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use sentry::protocol::DeviceContext;

    use super::*;

    fn context() -> TauriContext {
        let context = TauriContext::default();
        let _ = context.app.set(AppContext {
            app_name: Some("app".into()),
            app_version: Some("1.0.0".into()),
            app_identifier: Some("com.example.app".into()),
            ..Default::default()
        });
        let _ = context.webview.set(BrowserContext {
            name: Some("WKWebView".into()),
            version: Some("1.2.3".into()),
            ..Default::default()
        });
        *context.screen.lock().unwrap() = Some(Screen {
            resolution: "3840x2160".into(),
            density: 2.0,
        });
        context
    }

    #[test]
    fn adds_contexts() {
        let context = context();
        let mut event = Event::default();
        event.contexts.insert(
            "device".into(),
            Context::Device(Box::new(DeviceContext {
                arch: Some("aarch64".into()),
                ..Default::default()
            })),
        );
        context.apply(&mut event);

        let Some(Context::App(app)) = event.contexts.get("app") else {
            panic!("expected app context");
        };
        assert_eq!(app.app_name.as_deref(), Some("app"));
        assert_eq!(app.app_identifier.as_deref(), Some("com.example.app"));
        assert!(!app.other.contains_key("in_foreground"));

        let Some(Context::Runtime(runtime)) = event.contexts.get("runtime") else {
            panic!("expected runtime context");
        };
        assert_eq!(runtime.name.as_deref(), Some("Tauri"));
        assert_eq!(runtime.version.as_deref(), Some(tauri::VERSION));

        let Some(Context::Browser(browser)) = event.contexts.get("browser") else {
            panic!("expected browser context");
        };
        assert_eq!(browser.name.as_deref(), Some("WKWebView"));

        let Some(Context::Device(device)) = event.contexts.get("device") else {
            panic!("expected device context");
        };
        assert_eq!(device.arch.as_deref(), Some("aarch64"));
        assert_eq!(device.other["screen_resolution"], "3840x2160");
        assert_eq!(device.other["screen_density"], 2.0);
    }

    #[test]
    fn does_not_add_device_context() {
        let mut event = Event::default();
        context().apply(&mut event);
        assert!(!event.contexts.contains_key("device"));
    }

    #[test]
    fn keeps_existing_values() {
        let context = context();
        let mut event = Event::default();
        event.contexts.insert(
            "app".into(),
            Context::App(Box::new(AppContext {
                app_name: Some("other".into()),
                ..Default::default()
            })),
        );
        event.contexts.insert(
            "browser".into(),
            Context::Browser(Box::new(BrowserContext {
                name: Some("Chrome".into()),
                ..Default::default()
            })),
        );
        context.apply(&mut event);

        let Some(Context::App(app)) = event.contexts.get("app") else {
            panic!("expected app context");
        };
        assert_eq!(app.app_name.as_deref(), Some("other"));
        assert_eq!(app.app_version.as_deref(), Some("1.0.0"));

        let Some(Context::Browser(browser)) = event.contexts.get("browser") else {
            panic!("expected browser context");
        };
        assert_eq!(browser.name.as_deref(), Some("Chrome"));
    }

    #[test]
    fn tracks_foreground() {
        let context = context();
        assert_eq!(context.in_foreground(), None);

        context.set_focused("main", true);
        context.set_focused("other", true);
        context.set_focused("main", false);
        assert_eq!(context.in_foreground(), Some(true));

        context.set_focused("other", false);
        assert_eq!(context.in_foreground(), Some(false));

        let mut event = Event::default();
        context.apply(&mut event);
        let Some(Context::App(app)) = event.contexts.get("app") else {
            panic!("expected app context");
        };
        assert_eq!(app.other["in_foreground"], false);
    }
}
