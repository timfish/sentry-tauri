import type { Envelope, Event } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const WEBVIEW_NAME: Record<string, string> = {
  win32: 'WebView2',
  darwin: 'WKWebView',
  linux: 'WebKitGTK',
};

const runtime = { name: 'Tauri', version: expect.stringMatching(/^2\./) };

const appAndWebview = {
  app: {
    app_name: 'sentry-tauri-e2e',
    app_version: '0.1.0',
    app_identifier: 'io.sentry.tauri.e2e',
    app_start_time: expect.any(String),
    app_arch: expect.any(String),
  },
  runtime,
  browser: {
    name: WEBVIEW_NAME[process.platform],
    version: expect.stringMatching(/.+/),
  },
  device: {
    screen_resolution: expect.stringMatching(/^\d+x\d+$/),
    screen_density: expect.any(Number),
  },
};

async function firstEvent(scenario: string): Promise<Event> {
  const [event] = itemsOfType<Event>(await runScenario(scenario, hasEvent), 'event');
  return event;
}

describe('context', () => {
  test('JavaScript error', async () => {
    const event = await firstEvent('js-error');
    expect(event.contexts).toMatchObject(appAndWebview);
  });

  test('Rust panic', async () => {
    const event = await firstEvent('rust-panic');
    expect(event.contexts).toMatchObject(appAndWebview);
  });

  test('Native crash', async () => {
    const findMinidump = (envelopes: Envelope[]) =>
      envelopes.find(([, items]) =>
        items.some(([header]) => header.type === 'attachment' && header.attachment_type === 'event.minidump'),
      );
    const envelope = findMinidump(await runScenario('native-crash', (e) => !!findMinidump(e))) as Envelope;
    const [event] = itemsOfType<Event>([envelope], 'event');

    // The crash reporter process has no Tauri app, so it can only add
    // `runtime`. The published sentry-minidump does not send the app's
    // scope contexts to the reporter, so `app`, `browser` and the screen
    // values are missing.
    expect(event.contexts).toMatchObject({ runtime });
  });
});
