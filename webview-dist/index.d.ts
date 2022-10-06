import { BrowserOptions } from '@sentry/browser';
export * from '@sentry/browser';

declare global {
    interface Window {
        __SENTRY_DEBUG__: boolean;
    }
}
declare function init(options?: BrowserOptions): void;

export { init };
