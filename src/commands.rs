use sentry::{protocol::EnvelopeItem, Breadcrumb, Client, Envelope};
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum Buffer {
    Text(String),
    Raw(Vec<u8>),
}

#[tauri::command]
pub fn envelope(sentry_client: State<'_, Client>, envelope: Buffer) {
    let buffer = match envelope {
        Buffer::Text(str) => str.into_bytes(),
        Buffer::Raw(vec) => vec,
    };

    let parsed = Envelope::from_slice(&buffer);

    // sentry-rust's typed envelope parser can't yet deserialize the `debug_meta`
    // source-map images that @sentry/vite-plugin's debug-ID injection adds to
    // every event (sentry's `DebugImage` enum has no `sourcemap` variant and no
    // catch-all), so `from_slice` fails and the event would be silently dropped.
    // Forward the raw envelope bytes instead — they reach Sentry with
    // `debug_meta` intact, so server-side source-map symbolication still works.
    //
    // https://github.com/getsentry/sentry-rust/issues/1267
    if parsed.is_err() {
        if let Ok(raw) = Envelope::from_bytes_raw(buffer) {
            sentry_client.send_envelope(raw);
        }
        return;
    }

    if let Ok(envelope) = parsed {
        if let Some(mut event) = envelope.event().cloned() {
            event.platform = "javascript".into();

            // These come from the Rust config, so remove what came from the
            // browser SDK
            event.release = None;
            event.environment = None;
            event.dist = None;

            // We delete the user agent header so Sentry doesn't display weird browsers
            if let Some(ref mut req) = event.request {
                req.headers.remove("User-Agent");
            }

            // We need to pull any attachments out of the envelope and add them
            // to the scope when we capture the event.
            let attachments = envelope
                .items()
                .filter_map(|item| match item {
                    EnvelopeItem::Attachment(attachment) => Some(attachment.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();

            sentry::with_scope(
                |scope| {
                    for attachment in attachments {
                        scope.add_attachment(attachment);
                    }
                },
                || {
                    sentry::capture_event(event);
                },
            )
        } else {
            sentry_client.send_envelope(envelope);
        }
    }
}

#[tauri::command]
pub fn breadcrumb(breadcrumb: Breadcrumb) {
    sentry::add_breadcrumb(breadcrumb);
}
