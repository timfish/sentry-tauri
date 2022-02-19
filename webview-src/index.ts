import * as Sentry from "@sentry/browser";
import { getCurrentHub } from "@sentry/core";
import { invoke } from "@tauri-apps/api/tauri";
import { Event, EventProcessor, Integration } from "@sentry/types";

declare global {
  interface Window {
    Sentry: any;
  }
}

class TauriIntegration implements Integration {
  public static id: string = "TauriIntegration";

  public name: string = TauriIntegration.id;

  public setupOnce(
    addGlobalEventProcessor: (callback: EventProcessor) => void
  ): void {
    // Intercept events and sends them to Rust
    addGlobalEventProcessor(async (event: Event) => {
      // The Sentry Rust type de-serialisation doesn't like these in their
      // current state
      delete event.sdk;
      delete event.breadcrumbs;
      // This will be overridden in the host
      delete event.environment;
      // This isn't in the Rust types
      delete event.sdkProcessingMetadata;

      await invoke("plugin:sentry|event", { event });

      // Stop events from being sent from the browser
      return null;
    });

    // Intercept scope updates and send breadcrumbs to Rust
    const scope = getCurrentHub().getScope();
    if (scope) {
      scope.addScopeListener(async (updatedScope) => {
        // _breadcrumbs is private
        for (const breadcrumb of (updatedScope as any)._breadcrumbs) {
          await invoke("plugin:sentry|breadcrumb", { breadcrumb });
        }
        // Clear so we don't send these again
        updatedScope.clearBreadcrumbs();
      });
    }
  }
}

Sentry.init({
  // We don't send from the browser but a DSN is required for the SDK to work
  dsn: "https://123456@dummy.dsn/0",
  // We don't want to track browser sessions
  autoSessionTracking: false,
  // We replace this with true or false before injecting this code into the browser
  debug: "{{debug}}" as unknown as boolean,
  // At some point someone will want to customise this...
  integrations: [new TauriIntegration()],
});

// Expose to users so Sentry.capture* works
window.Sentry = Sentry;
