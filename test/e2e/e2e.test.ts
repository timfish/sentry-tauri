import type { Envelope, Event } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { ENVIRONMENT, RELEASE, itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const pluginPackage = expect.arrayContaining([expect.objectContaining({ name: 'cargo:tauri-plugin-sentry' })]);

describe('e2e', () => {
  test('JavaScript error', async () => {
    const envelopes = await runScenario('js-error', hasEvent);
    const [event] = itemsOfType(envelopes, 'event');

    expect(event).toMatchObject({
      platform: 'javascript',
      release: RELEASE,
      environment: ENVIRONMENT,
      exception: { values: [expect.objectContaining({ type: 'Error', value: 'js error' })] },
      contexts: {
        os: expect.any(Object),
        device: expect.any(Object),
        rust: expect.any(Object),
      },
      sdk: { name: 'sentry.javascript.browser', packages: pluginPackage },
      breadcrumbs: {
        values: expect.arrayContaining([
          expect.objectContaining({ message: 'rust breadcrumb' }),
          expect.objectContaining({ message: 'js breadcrumb' }),
        ]),
      },
    });
  });

  test('Rust panic', async () => {
    const envelopes = await runScenario('rust-panic', hasEvent);
    const [event] = itemsOfType(envelopes, 'event');

    expect(event).toMatchObject({
      level: 'fatal',
      release: RELEASE,
      environment: ENVIRONMENT,
      exception: { values: [expect.objectContaining({ type: 'panic', value: 'rust panic' })] },
      contexts: { os: expect.any(Object), rust: expect.any(Object) },
      sdk: { name: 'sentry.rust', packages: pluginPackage },
    });
  });

  test('Native crash', async () => {
    const findMinidump = (envelopes: Envelope[]) =>
      envelopes.find(([, items]) =>
        items.some(([header]) => header.type === 'attachment' && header.attachment_type === 'event.minidump'),
      );
    const envelope = findMinidump(await runScenario('native-crash', (e) => !!findMinidump(e))) as Envelope;

    const [event] = itemsOfType([envelope], 'event');
    expect(event).toMatchObject({
      level: 'fatal',
      release: RELEASE,
      environment: ENVIRONMENT,
      sdk: { packages: pluginPackage },
    });

    const [, minidump] = envelope[1].find(([header]) => header.type === 'attachment') ?? [];
    expect(Buffer.from(minidump as Uint8Array).subarray(0, 4).toString()).toBe('MDMP');
  });

  test('JavaScript transaction', async () => {
    const envelopes = await runScenario('transaction', (e) => itemsOfType(e, 'transaction').length > 0);
    const [transaction] = itemsOfType(envelopes, 'transaction');

    // sentry-rust cannot parse the float timestamps of browser
    // transactions, so the plugin forwards them as they came.
    expect(transaction).toMatchObject({
      type: 'transaction',
      transaction: 'e2e transaction',
      contexts: { trace: { op: 'e2e' } },
      sdk: { name: 'sentry.javascript.browser' },
    });
  });

  test('JavaScript error with debug IDs', async () => {
    const envelopes = await runScenario('debug-id', hasEvent);
    const [event] = itemsOfType<Event>(envelopes, 'event');

    // sentry-rust cannot parse source map debug images, so the plugin
    // forwards these envelopes as they came from the browser SDK.
    expect(event).toMatchObject({
      platform: 'javascript',
      exception: { values: [expect.objectContaining({ type: 'Error', value: 'debug id error' })] },
      sdk: { name: 'sentry.javascript.browser' },
      debug_meta: {
        images: [
          {
            type: 'sourcemap',
            debug_id: '3bd0e8d1-0f5c-4d6e-9a43-1d2b6c7e8f90',
            code_file: expect.stringContaining('debug-id.js'),
          },
        ],
      },
    });
  });
});
