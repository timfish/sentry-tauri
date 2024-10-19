import * as Sentry from '@sentry/browser';
import { defaultOptions } from './';

declare const __DEBUG__: boolean;

declare global {
  interface Window {
    Sentry: typeof Sentry;
  }
}

window.Sentry = Sentry;

Sentry.init({
  ...defaultOptions,
  // We replace this with true or false before injecting this code into the browser
  debug: __DEBUG__,
});
