use sentry::{
    protocol::{Event, Map, SessionStatus, Value},
    Breadcrumb, Hub, Level,
};
use tauri::{Runtime, Webview};

use crate::process::WebviewTags;

/// Captures the termination of a WebKit web content process.
///
/// WebKit gives no reason for the termination. Call this from
/// `tauri::Builder::on_web_content_process_terminate` on macOS and iOS.
/// That hook replaces the Tauri default, which reloads the webview, so
/// reload the webview yourself after you call this function.
pub fn web_content_process_terminated<R: Runtime>(webview: &Webview<R>) {
    capture(&WebviewTags::new(webview), "abnormal-exit", Map::new());
}

/// Adds handlers that capture webview process failures. WebView2 and
/// WebKitGTK allow more than one handler, so this keeps the wry handlers.
pub(crate) fn install<R: Runtime>(webview: &Webview<R>) {
    #[cfg(any(
        windows,
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        let tags = WebviewTags::new(webview);
        let _ = webview.with_webview(move |platform| platform::install(platform, tags));
    }

    #[cfg(not(any(
        windows,
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let _ = webview;
}

/// Uses the same levels as the Electron SDK.
fn level_for_reason(reason: &str) -> Level {
    match reason {
        "abnormal-exit" | "killed" => Level::Warning,
        "crashed" | "oom" | "launch-failed" | "integrity-failure" => Level::Fatal,
        _ => Level::Debug,
    }
}

fn capture(tags: &WebviewTags, reason: &str, mut data: Map<String, Value>) {
    let hub = Hub::current();
    let level = level_for_reason(reason);
    let message = format!("'webview' process exited with '{reason}'");

    if level >= Level::Warning {
        hub.with_scope(
            |scope| tags.apply_to_scope(scope),
            || {
                hub.capture_event(Event {
                    message: Some(message.clone()),
                    level,
                    ..Default::default()
                })
            },
        );
    }

    data.insert("reason".into(), reason.into());
    data.insert("label".into(), tags.webview().into());
    hub.add_breadcrumb(Breadcrumb {
        ty: "process".into(),
        category: Some("child-process".into()),
        message: Some(message),
        level,
        data,
        ..Default::default()
    });

    if level == Level::Fatal {
        hub.end_session_with_status(SessionStatus::Crashed);
    }
}

#[cfg(windows)]
mod platform {
    use sentry::protocol::Map;
    use tauri::webview::PlatformWebview;

    use crate::process::WebviewTags;
    use webview2_com::{
        Microsoft::Web::WebView2::Win32::{
            ICoreWebView2ProcessFailedEventArgs, ICoreWebView2ProcessFailedEventArgs2,
            COREWEBVIEW2_PROCESS_FAILED_KIND,
            COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_FRAME_RENDER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_GPU_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE,
            COREWEBVIEW2_PROCESS_FAILED_KIND_SANDBOX_HELPER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_UTILITY_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_REASON, COREWEBVIEW2_PROCESS_FAILED_REASON_CRASHED,
            COREWEBVIEW2_PROCESS_FAILED_REASON_LAUNCH_FAILED,
            COREWEBVIEW2_PROCESS_FAILED_REASON_OUT_OF_MEMORY,
            COREWEBVIEW2_PROCESS_FAILED_REASON_TERMINATED,
        },
        ProcessFailedEventHandler,
    };
    use windows_core::Interface;

    pub(super) fn install(platform: PlatformWebview, tags: WebviewTags) {
        let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
            if let Some(args) = args {
                on_process_failed(&tags, &args);
            }
            Ok(())
        }));

        unsafe {
            if let Ok(webview) = platform.controller().CoreWebView2() {
                let mut token = 0;
                let _ = webview.add_ProcessFailed(&handler, &mut token);
            }
        }
    }

    fn on_process_failed(tags: &WebviewTags, args: &ICoreWebView2ProcessFailedEventArgs) {
        let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
        if unsafe { args.ProcessFailedKind(&mut kind) }.is_err() {
            return;
        }

        // The process is still alive, so this is not an exit.
        if kind == COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE {
            return;
        }

        let mut data = Map::new();
        data.insert("kind".into(), kind_name(kind).into());

        // Older WebView2 runtimes do not have this interface.
        let mut reason = "abnormal-exit";
        if let Ok(args2) = args.cast::<ICoreWebView2ProcessFailedEventArgs2>() {
            let mut value = COREWEBVIEW2_PROCESS_FAILED_REASON::default();
            if unsafe { args2.Reason(&mut value) }.is_ok() {
                reason = reason_name(value);
            }

            let mut exit_code = 0;
            if unsafe { args2.ExitCode(&mut exit_code) }.is_ok() {
                data.insert("exit_code".into(), exit_code.into());
            }
        }

        super::capture(tags, reason, data);
    }

    fn kind_name(kind: COREWEBVIEW2_PROCESS_FAILED_KIND) -> &'static str {
        match kind {
            COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED => "browser",
            COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED => "render",
            COREWEBVIEW2_PROCESS_FAILED_KIND_FRAME_RENDER_PROCESS_EXITED => "frame-render",
            COREWEBVIEW2_PROCESS_FAILED_KIND_GPU_PROCESS_EXITED => "gpu",
            COREWEBVIEW2_PROCESS_FAILED_KIND_UTILITY_PROCESS_EXITED => "utility",
            COREWEBVIEW2_PROCESS_FAILED_KIND_SANDBOX_HELPER_PROCESS_EXITED => "sandbox-helper",
            _ => "unknown",
        }
    }

    fn reason_name(reason: COREWEBVIEW2_PROCESS_FAILED_REASON) -> &'static str {
        match reason {
            COREWEBVIEW2_PROCESS_FAILED_REASON_CRASHED => "crashed",
            COREWEBVIEW2_PROCESS_FAILED_REASON_OUT_OF_MEMORY => "oom",
            COREWEBVIEW2_PROCESS_FAILED_REASON_LAUNCH_FAILED => "launch-failed",
            COREWEBVIEW2_PROCESS_FAILED_REASON_TERMINATED => "killed",
            _ => "abnormal-exit",
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod platform {
    use sentry::protocol::Map;
    use tauri::webview::PlatformWebview;
    use webkit2gtk::{WebProcessTerminationReason, WebViewExt};

    use crate::process::WebviewTags;

    pub(super) fn install(platform: PlatformWebview, tags: WebviewTags) {
        platform
            .inner()
            .connect_web_process_terminated(move |_, reason| {
                let reason = match reason {
                    WebProcessTerminationReason::Crashed => "crashed",
                    WebProcessTerminationReason::ExceededMemoryLimit => "oom",
                    WebProcessTerminationReason::TerminatedByApi => "clean-exit",
                    _ => "abnormal-exit",
                };
                super::capture(&tags, reason, Map::new());
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags() -> WebviewTags {
        WebviewTags::from_labels("main", "main-window")
    }

    #[test]
    fn captures_event_and_breadcrumb() {
        let events = sentry::test::with_captured_events(|| {
            capture(&tags(), "crashed", Map::new());
            sentry::capture_message("next", Level::Info);
        });

        assert_eq!(events.len(), 2);
        let crash = &events[0];
        assert_eq!(
            crash.message.as_deref(),
            Some("'webview' process exited with 'crashed'")
        );
        assert_eq!(crash.level, Level::Fatal);
        assert_eq!(crash.tags["event.process"], "webview");
        assert_eq!(crash.tags["tauri.webview"], "main");
        assert_eq!(crash.tags["tauri.window"], "main-window");

        let breadcrumb = &events[1].breadcrumbs.values[0];
        assert_eq!(breadcrumb.category.as_deref(), Some("child-process"));
        assert_eq!(breadcrumb.data["reason"], "crashed");
        assert_eq!(breadcrumb.data["label"], "main");
    }

    #[test]
    fn clean_exit_adds_only_breadcrumb() {
        let events = sentry::test::with_captured_events(|| {
            capture(&tags(), "clean-exit", Map::new());
            sentry::capture_message("next", Level::Info);
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].breadcrumbs.values[0].level, Level::Debug);
    }
}
