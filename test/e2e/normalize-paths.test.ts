import type { Envelope, Event } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const WEBVIEW_ORIGINS = /tauri:\/\/|tauri\.localhost|ipc\.localhost/;

function frameFilenames(event: Event): string[] {
  return (event.exception?.values || []).flatMap((e) => (e.stacktrace?.frames || []).map((f) => f.filename || ''));
}

describe('normalize paths', () => {
  test('JavaScript error frames use app:///', async () => {
    const [event] = itemsOfType(await runScenario('js-error', hasEvent), 'event');

    const filenames = frameFilenames(event);
    expect(filenames.length).toBeGreaterThan(0);
    for (const filename of filenames) {
      expect(filename).toMatch(/^app:\/\/\//);
      expect(filename).not.toMatch(WEBVIEW_ORIGINS);
    }
  });

  test('debug images use app:/// and keep the debug ID', async () => {
    const [event] = itemsOfType(await runScenario('debug-id', hasEvent), 'event');

    for (const filename of frameFilenames(event)) {
      expect(filename).toMatch(/^app:\/\/\//);
    }
    expect(event.debug_meta?.images).toEqual([
      {
        type: 'sourcemap',
        debug_id: '3bd0e8d1-0f5c-4d6e-9a43-1d2b6c7e8f90',
        code_file: 'app:///debug-id.js',
      },
    ]);
  });

  test('transactions have no webview origin', async () => {
    const envelopes = await runScenario('transaction', (e) => itemsOfType(e, 'transaction').length > 0);
    const [transaction] = itemsOfType(envelopes, 'transaction');

    expect(transaction.request?.url).toMatch(/^app:\/\/\//);
    expect(JSON.stringify(transaction)).not.toMatch(WEBVIEW_ORIGINS);
  });
});
