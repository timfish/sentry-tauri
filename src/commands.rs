use sentry::{add_breadcrumb, capture_event, protocol::Event, Breadcrumb};
use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn event<R: Runtime>(_app: AppHandle<R>, mut event: Event<'static>) {
    event.platform = "javascript".into();
    capture_event(event);
}

#[tauri::command]
pub fn breadcrumb<R: Runtime>(_app: AppHandle<R>, breadcrumb: Breadcrumb) {
    add_breadcrumb(breadcrumb);
}
