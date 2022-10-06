// webview-src/index.ts
import { init as SentryInit } from "@sentry/browser";

// webview-src/integration.ts
import { getCurrentHub } from "@sentry/browser";
import { invoke } from "@tauri-apps/api/tauri";
var _TauriIntegration = class {
  constructor() {
    this.name = _TauriIntegration.id;
  }
  setupOnce(addGlobalEventProcessor) {
    addGlobalEventProcessor(async (event) => {
      delete event.sdk;
      delete event.breadcrumbs;
      delete event.environment;
      delete event.sdkProcessingMetadata;
      await invoke("plugin:sentry|event", { event });
      return null;
    });
    const scope = getCurrentHub().getScope();
    if (scope) {
      scope.addScopeListener(async (updatedScope) => {
        for (const breadcrumb of updatedScope._breadcrumbs) {
          await invoke("plugin:sentry|breadcrumb", { breadcrumb });
        }
        updatedScope.clearBreadcrumbs();
      });
    }
  }
};
var TauriIntegration = _TauriIntegration;
TauriIntegration.id = "TauriIntegration";

// webview-src/index.ts
export * from "@sentry/browser";
function init(options) {
  let opts = {
    dsn: "https://123456@dummy.dsn/0",
    autoSessionTracking: false,
    debug: window.__SENTRY_DEBUG__,
    ...options,
    integrations: [new TauriIntegration()]
  };
  if (typeof (options == null ? void 0 : options.integrations) === "function") {
    opts.integrations.push(...options.integrations(opts.integrations));
  } else if (!!(options == null ? void 0 : options.integrations)) {
    opts.integrations.push(...opts.integrations);
  }
  SentryInit(opts);
}
export {
  init
};
