import * as Sentry from '@sentry/browser';
import { invoke } from '@tauri-apps/api/core';
import { defaultOptions } from '../../../../dist/index.js';

const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function throwUncaught(fn) {
  setTimeout(fn);
}

function loadScript(src) {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = src;
    script.onload = resolve;
    script.onerror = reject;
    document.head.appendChild(script);
  });
}

function setScope() {
  window.Sentry.setUser({ id: 'e2e-user', email: 'e2e@example.com' });
  window.Sentry.setTag('e2e-tag', 'tag value');
  window.Sentry.setExtra('e2e-extra', { nested: true });
  window.Sentry.setContext('e2e-context', { key: 'value' });
}

const scenarios = {
  async 'js-error'() {
    await invoke('rust_breadcrumb');
    window.Sentry.addBreadcrumb({ message: 'js breadcrumb' });
    await delay(100);
    throwUncaught(() => {
      throw new Error('js error');
    });
  },
  async 'rust-panic'() {
    await invoke('rust_panic');
  },
  async 'native-crash'() {
    await invoke('native_crash');
  },
  async transaction() {
    Sentry.init({ ...defaultOptions, tracesSampleRate: 1 });
    window.Sentry = Sentry;
    await Sentry.startSpan({ name: 'e2e transaction', op: 'e2e' }, () => delay(50));
  },
  async 'scope-sync-panic'() {
    setScope();
    await delay(100);
    await invoke('rust_panic');
  },
  async 'scope-sync-async'() {
    setScope();
    await delay(100);
    await invoke('rust_async_message');
  },
  async 'scope-sync-crash'() {
    setScope();
    await delay(100);
    await invoke('native_crash');
  },
  async 'scope-sync-clear-user'() {
    setScope();
    await delay(100);
    window.Sentry.setUser(null);
    await delay(100);
    await invoke('rust_panic');
  },
  async 'debug-id'() {
    await loadScript('debug-id.js');
    throwUncaught(window.throwWithDebugId);
  },
};

async function run() {
  const scenario = await invoke('scenario');
  console.log(`Running scenario: ${scenario}`);
  const fn = scenarios[scenario];
  if (!fn) {
    throw new Error(`Unknown scenario: ${scenario}`);
  }
  await fn();
  await delay(1000);
  await window.Sentry.flush(5000);
}

run()
  .catch((error) => console.error(error))
  .finally(() => invoke('exit'));
