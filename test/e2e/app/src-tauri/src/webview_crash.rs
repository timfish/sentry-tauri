use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Webview;

static CRASHED: AtomicBool = AtomicBool::new(false);

/// Crashes the web content process of the webview once. Returns false if
/// it already did, so the reloaded page can finish the scenario.
#[tauri::command]
pub fn crash_webview(webview: Webview) -> bool {
    if CRASHED.swap(true, Ordering::SeqCst) {
        return false;
    }

    println!("[e2e] crashing webview '{}'", webview.label());
    // Crash after the command returns its response.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        crash(webview);
    });
    true
}

#[cfg(target_os = "linux")]
fn crash(_webview: Webview) {
    // The name in `comm` has at most 15 characters.
    let pids = descendants(std::process::id())
        .into_iter()
        .filter(|pid| {
            std::fs::read_to_string(format!("/proc/{pid}/comm"))
                .is_ok_and(|comm| comm.trim() == "WebKitWebProces")
        })
        .collect::<Vec<_>>();

    println!("[e2e] sending SIGSEGV to WebKitWebProcess {pids:?}");
    for pid in pids {
        let _ = std::process::Command::new("kill")
            .args(["-SEGV", &pid.to_string()])
            .status();
    }
}

#[cfg(target_os = "linux")]
fn descendants(root: u32) -> Vec<u32> {
    let parents = std::fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
            let stat = std::fs::read_to_string(entry.path().join("stat")).ok()?;
            // The process name can hold spaces, so read after the last ')'.
            let ppid = stat.rsplit(')').next()?.split_whitespace().nth(1)?;
            Some((pid, ppid.parse::<u32>().ok()?))
        })
        .collect::<Vec<_>>();

    let mut found = vec![root];
    let mut i = 0;
    while i < found.len() {
        let parent = found[i];
        found.extend(
            parents
                .iter()
                .filter(|(_, ppid)| *ppid == parent)
                .map(|(pid, _)| *pid),
        );
        i += 1;
    }
    found.split_off(1)
}

#[cfg(windows)]
fn crash(webview: Webview) {
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows_core::w;

    let _ = webview.with_webview(|platform| unsafe {
        println!("[e2e] calling Page.crash");
        let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(())));
        if let Ok(core) = platform.controller().CoreWebView2() {
            let result = core.CallDevToolsProtocolMethod(w!("Page.crash"), w!("{}"), &handler);
            println!("[e2e] Page.crash result: {result:?}");
        }
    });
}

#[cfg(target_os = "macos")]
fn crash(webview: Webview) {
    use objc2::{msg_send, runtime::AnyObject};

    let _ = webview.with_webview(|platform| {
        // `_webProcessIdentifier` is private, but this is only a test app.
        let view = platform.inner() as *mut AnyObject;
        let pid: i32 = unsafe { msg_send![view, _webProcessIdentifier] };
        println!("[e2e] killing WebContent process {pid}");
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    });
}

#[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
fn crash(_webview: Webview) {}
