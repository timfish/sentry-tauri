// webview-src/index.ts
import { init as SentryInit } from "@sentry/browser";
export * from "@sentry/browser";
function init(options) {
  SentryInit({
    ...window.__SENTRY_INIT__,
    autoSessionTracking: false,
    ...options
  });
}
export {
  init
};
