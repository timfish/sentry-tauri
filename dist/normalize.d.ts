import type { Event } from '@sentry/core';
/**
 * Replaces the webview origin in URLs in an event with `app:///`. Mutates the
 * passed in event.
 *
 * @param event The event to normalize.
 * @param origin The webview origin, for example `tauri://localhost`.
 */
export declare function normalizeUrls(event: Event, origin: string): Event;
/**
 * Replaces the webview origin with `app:///` so URLs match on every platform.
 * Tauri serves the app from `tauri://localhost` on macOS, iOS and Linux and
 * from `http://tauri.localhost` on Windows and Android.
 */
export declare const normalizeUrlsIntegration: () => import("@sentry/core").Integration;
