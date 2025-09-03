import { createTransport } from '@sentry/browser';
import { invoke } from '@tauri-apps/api/core';
// If IPC fails this is usually due to permissions. We quit sending because we
// can't recover from this and we don't want to spam the console with errors.
let ipcFailed = false;
/**
 * Creates a Transport that passes envelopes to the Tauri Rust process.
 */
export function makeRendererTransport(options) {
    return createTransport(options, async (request) => {
        if (ipcFailed) {
            return { statusCode: 200 };
        }
        try {
            await invoke('plugin:sentry|envelope', { envelope: request.body });
        }
        catch (e) {
            console.error('Failed to send envelope to Rust:', e);
            ipcFailed = true;
        }
        // Since the Rust process handles sending of envelopes and rate limiting, we always return 200 OK to the renderers.
        return { statusCode: 200 };
    });
}
/**
 * A `beforeBreadcrumb` hook that sends the breadcrumb to the Rust process via Tauri invoke.
 */
export function sendBreadcrumbToRust(breadcrumb) {
    if (ipcFailed) {
        return null;
    }
    // Ignore IPC breadcrumbs otherwise we'll make an infinite loop
    if (typeof breadcrumb.data?.url === 'string' &&
        (breadcrumb.data.url.startsWith('ipc://') || breadcrumb.data.url.match(/^https?:\/\/ipc\.localhost/))) {
        return null;
    }
    invoke('plugin:sentry|breadcrumb', { breadcrumb }).catch((e) => {
        console.error('Failed to send breadcrumb to Rust:', e);
        ipcFailed = true;
    });
    // We don't collect breadcrumbs in the renderer since they are passed to Rust
    return null;
}
/**
 * Default options for the Sentry browser SDK to pass events and breadcrumbs to
 * the Rust SDK.
 */
export const defaultOptions = {
    // We don't send from the browser but a DSN is required for the SDK to start
    dsn: 'https://123456@dummy.dsn/0',
    // We want to track app sessions rather than browser sessions
    integrations: (integrations) => integrations.filter((i) => i.name !== 'BrowserSession'),
    transport: makeRendererTransport,
    beforeBreadcrumb: sendBreadcrumbToRust,
};
