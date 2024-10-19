import { BrowserOptions } from "@sentry/browser";
import { Breadcrumb, Event } from "@sentry/types";
/**
 * A simple `beforeSend` that sends the envelope to the Rust process via Tauri invoke.
 */
export declare function sendEventToRust(event: Event): null;
/**
 * A simple `beforeBreadcrumb` hook that sends the breadcrumb to the Rust process via Tauri invoke.
 */
export declare function sendBreadcrumbToRust(breadcrumb: Breadcrumb): Breadcrumb | null;
/**
 * Default options for the Sentry browser SDK to pass events and breadcrumbs to
 * the Rust SDK.
 */
export declare const defaultOptions: BrowserOptions;
