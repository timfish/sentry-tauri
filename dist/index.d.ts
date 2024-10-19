import { type BrowserOptions } from "@sentry/browser";
import type { BaseTransportOptions, Breadcrumb, Transport } from "@sentry/types";
/**
 * Creates a Transport that passes envelopes to the Tauri Rust process.
 */
export declare function makeRendererTransport(options: BaseTransportOptions): Transport;
/**
 * A `beforeBreadcrumb` hook that sends the breadcrumb to the Rust process via Tauri invoke.
 */
export declare function sendBreadcrumbToRust(breadcrumb: Breadcrumb): Breadcrumb | null;
/**
 * Default options for the Sentry browser SDK to pass events and breadcrumbs to
 * the Rust SDK.
 */
export declare const defaultOptions: BrowserOptions;
