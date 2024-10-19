import { type BrowserOptions, createTransport } from '@sentry/browser';
import { invoke } from '@tauri-apps/api/core';
import type {
  BaseTransportOptions,
  Breadcrumb,
  Transport,
  TransportMakeRequestResponse,
  TransportRequest,
} from '@sentry/types';

/**
 * Creates a Transport that passes envelopes to the Tauri Rust process.
 */
export function makeRendererTransport(options: BaseTransportOptions): Transport {
  return createTransport(options, async (request: TransportRequest): Promise<TransportMakeRequestResponse> => {
    invoke('plugin:sentry|envelope', { envelope: request.body });
    // Since the Rust process handles sending of envelopes and rate limiting, we always return 200 OK to the renderers.
    return { statusCode: 200 };
  });
}

/**
 * A `beforeBreadcrumb` hook that sends the breadcrumb to the Rust process via Tauri invoke.
 */
export function sendBreadcrumbToRust(breadcrumb: Breadcrumb): Breadcrumb | null {
  // Ignore IPC breadcrumbs otherwise we'll make an infinite loop
  if (typeof breadcrumb.data?.url === 'string' && breadcrumb.data.url.startsWith('ipc://')) {
    return null;
  }

  invoke('plugin:sentry|breadcrumb', { breadcrumb });
  // We don't collect breadcrumbs in the renderer since they are passed to Rust
  return null;
}

/**
 * Default options for the Sentry browser SDK to pass events and breadcrumbs to
 * the Rust SDK.
 */
export const defaultOptions: BrowserOptions = {
  // We don't send from the browser but a DSN is required for the SDK to start
  dsn: 'https://123456@dummy.dsn/0',
  // We want to track app sessions rather than browser sessions
  autoSessionTracking: false,
  transport: makeRendererTransport,
  beforeBreadcrumb: sendBreadcrumbToRust,
};
