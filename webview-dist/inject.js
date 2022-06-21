import * as Sentry from "@sentry/browser";
import { TauriIntegration } from "./";
Sentry.init({
    // We don't send from the browser but a DSN is required for the SDK to work
    dsn: "https://123456@dummy.dsn/0",
    // We don't want to track browser sessions
    autoSessionTracking: false,
    // We replace this with true or false before injecting this code into the browser
    debug: __DEBUG__,
    // At some point someone will want to customise this...
    integrations: [new TauriIntegration()],
});
window.Sentry = Sentry;
