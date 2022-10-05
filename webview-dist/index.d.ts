import { BrowserOptions } from '@sentry/browser';

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
declare function init(options?: BrowserOptions): void;

export { init };
