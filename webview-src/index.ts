import { BrowserOptions, init as SentryInit } from "@sentry/browser";

declare global {
  interface Window {
    __SENTRY_INIT__: {
      dsn: string;
      release?: string;
      environment?: string;
      debug: boolean;
    };
  }
}

export function init(options?: BrowserOptions): void {  
  SentryInit({
    ...window.__SENTRY_INIT__,
    // We don't want to track browser sessions
    autoSessionTracking: false,
    ...options,
  });
}

export * from "@sentry/browser"
