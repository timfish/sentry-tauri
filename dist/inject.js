import * as Sentry from "@sentry/browser";
import { defaultOptions } from "./";
window.Sentry = Sentry;
Sentry.init({
    ...defaultOptions,
    // We replace this with true or false before injecting this code into the browser
    debug: __DEBUG__,
});
