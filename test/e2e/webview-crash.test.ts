import type { Envelope, Event } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { ENVIRONMENT, RELEASE, itemsOfType, runScenario } from './runner';

const findCrash = (envelopes: Envelope[]) =>
  itemsOfType<Event>(envelopes, 'event').find((event) => event.message?.includes("'webview' process exited"));

// WebKit on macOS gives no reason for the exit.
const [reason, level] = process.platform === 'darwin' ? ['abnormal-exit', 'warning'] : ['crashed', 'fatal'];

describe('e2e', () => {
  test('Webview crash', async () => {
    const envelopes = await runScenario('webview-crash', (e) => !!findCrash(e));

    expect(findCrash(envelopes)).toMatchObject({
      message: `'webview' process exited with '${reason}'`,
      level,
      release: RELEASE,
      environment: ENVIRONMENT,
      tags: {
        'event.process': 'webview',
        'tauri.webview': 'main',
        'tauri.window': 'main',
      },
    });
  });
});
