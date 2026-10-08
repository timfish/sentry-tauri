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
            sentry_client.send_envelope(add_sdk_package(envelope));
        }
    }
}

fn add_sdk_package(envelope: Envelope) -> Envelope {
    let mut out = Envelope::new().with_headers(envelope.headers().clone());
    for mut item in envelope.into_items() {
        if let EnvelopeItem::Transaction(ref mut transaction) = item {
            if let Some(sdk) = transaction.sdk.as_mut() {
                crate::add_sdk_package(sdk);
            }
        }
        out.add_item(item);
    }
    out
}

#[tauri::command]
pub fn breadcrumb(breadcrumb: Breadcrumb) {
    sentry::add_breadcrumb(breadcrumb);
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use sentry::protocol::{Attachment, ClientSdkInfo, Transaction};

    use super::*;

    #[test]
    fn adds_sdk_package_to_transactions() {
        let mut envelope = Envelope::new();
        envelope.add_item(Transaction {
            sdk: Some(Cow::Owned(ClientSdkInfo {
                name: "sentry.javascript.browser".into(),
                version: "10.0.0".into(),
                integrations: vec![],
                packages: vec![],
            })),
            ..Default::default()
        });
        envelope.add_item(Attachment {
            buffer: b"data".to_vec(),
            filename: "file.txt".into(),
            ..Default::default()
        });

        let envelope = add_sdk_package(envelope);
        let items = envelope.items().collect::<Vec<_>>();
        assert_eq!(items.len(), 2);

        let EnvelopeItem::Transaction(transaction) = items[0] else {
            panic!("expected transaction");
        };
        let sdk = transaction.sdk.as_ref().unwrap();
        assert_eq!(sdk.name, "sentry.javascript.browser");
        assert!(sdk
            .packages
            .iter()
            .any(|p| p.name == "cargo:tauri-plugin-sentry"));
    }
}
