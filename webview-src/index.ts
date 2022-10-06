import { BrowserOptions, init as SentryInit } from "@sentry/browser";
import { Integration } from "@sentry/types";
import {TauriIntegration} from './integration'

declare global {
  interface Window {
    __SENTRY_DEBUG__: boolean
  }
}

export function init(options?: BrowserOptions): void {
  let opts: BrowserOptions & { integrations: Integration[] } = {
    // We don't send from the browser but a DSN is required for the SDK to work
    dsn: "https://123456@dummy.dsn/0",
    // We don't want to track browser sessions
    autoSessionTracking: false,
    // We replace this with true or false before injecting this code into the browser
    debug: window.__SENTRY_DEBUG__,
  
    ...options,

    // whatever the user configures, the Tauri integration must alway be set
    integrations: [new TauriIntegration()]
  }

  if (typeof options?.integrations === "function") {
    opts.integrations.push(...options.integrations(opts.integrations))
  } else if (!!options?.integrations) {
    opts.integrations.push(...opts.integrations)
  }

  SentryInit(opts);
}

export * from "@sentry/browser"
