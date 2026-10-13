import type { Envelope, Event } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const webviewTags = {
  'event.process': 'webview',
  'tauri.webview': 'main',
  'tauri.window': 'main',
};

describe('process tags', () => {
  test('JavaScript error', async () => {
    const envelopes = await runScenario('js-error', hasEvent);
    const [event] = itemsOfType<Event>(envelopes, 'event');

    expect(event.tags).toMatchObject(webviewTags);
  });

  test('JavaScript transaction', async () => {
    const envelopes = await runScenario('transaction', (e) => itemsOfType(e, 'transaction').length > 0);
    const [transaction] = itemsOfType<Event>(envelopes, 'transaction');

    // sentry-rust cannot parse the float timestamps of browser
    // transactions. The plugin forwards them as they came, without
    // tags. When they parse, check for the webview tags here.
    expect(transaction.transaction).toBe('e2e transaction');
    expect(transaction.tags?.['event.process']).toBeUndefined();
  });

  test('Rust panic', async () => {
    const envelopes = await runScenario('rust-panic', hasEvent);
    const [event] = itemsOfType<Event>(envelopes, 'event');

    expect(event.tags).toMatchObject({ 'event.process': 'rust' });
  });

  test('Native crash', async () => {
    const findMinidump = (envelopes: Envelope[]) =>
      envelopes.find(([, items]) =>
        items.some(([header]) => header.type === 'attachment' && header.attachment_type === 'event.minidump'),
      );
    const envelope = findMinidump(await runScenario('native-crash', (e) => !!findMinidump(e))) as Envelope;
    const [event] = itemsOfType<Event>([envelope], 'event');

    expect(event.tags).toMatchObject({ 'event.process': 'rust' });
  });
});
