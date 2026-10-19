import type { Envelope } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const user = { id: 'e2e-user', email: 'e2e@example.com' };
const tags = { 'e2e-tag': 'tag value' };

const scope = {
  user,
  tags,
  extra: { 'e2e-extra': { nested: true } },
  contexts: { 'e2e-context': { key: 'value' } },
};

describe('scope sync', () => {
  test('Rust panic in a sync command has the webview scope', async () => {
    const [event] = itemsOfType(await runScenario('scope-sync-panic', hasEvent), 'event');

    expect(event).toMatchObject({
      exception: { values: [expect.objectContaining({ type: 'panic', value: 'rust panic' })] },
      ...scope,
    });
  });

  test('Rust message from an async command has the webview scope', async () => {
    const [event] = itemsOfType(await runScenario('scope-sync-async', hasEvent), 'event');

    expect(event).toMatchObject({ message: 'rust async message', ...scope });
  });

  // sentry-minidump 0.49.3 does not send the app scope to the crash
  // reporter. Enable this when a release includes scope sync.
  test.skip('Native crash has the webview user and tags', async () => {
    const hasMinidump = (envelopes: Envelope[]) =>
      envelopes.some(([, items]) => items.some(([header]) => header.attachment_type === 'event.minidump'));
    const [event] = itemsOfType(await runScenario('scope-sync-crash', hasMinidump), 'event');

    expect(event).toMatchObject({ user, tags });
  });

  test('setUser(null) clears the user', async () => {
    const [event] = itemsOfType(await runScenario('scope-sync-clear-user', hasEvent), 'event');

    expect(event).toMatchObject({ tags });
    expect(event?.user).toBeUndefined();
  });
});
