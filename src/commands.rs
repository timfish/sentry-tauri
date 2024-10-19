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

    let envelope = Envelope::from_slice(&buffer);

    if let Ok(envelope) = envelope {
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
                .map(|item| match item {
                    EnvelopeItem::Attachment(attachment) => Some(attachment.clone()),
                    _ => None,
                })
                .flatten()
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
