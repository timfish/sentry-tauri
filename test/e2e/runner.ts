import { spawn } from 'node:child_process';
import { join } from 'node:path';
import type { Envelope, EnvelopeItem, Event } from '@sentry/core';
import { createSentryTestServer } from './server';

const APP_DIR = join(__dirname, 'app', 'src-tauri');
const TARGET_DIR = process.env.CARGO_TARGET_DIR || join(APP_DIR, 'target');
const BINARY = join(TARGET_DIR, 'debug', `sentry-tauri-e2e${process.platform === 'win32' ? '.exe' : ''}`);

export const RELEASE = 'sentry-tauri-e2e@1.0.0';
export const ENVIRONMENT = 'e2e';

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Returns the payloads of all items of the given type. */
export function itemsOfType<T = Event>(envelopes: Envelope[], type: string): T[] {
  return envelopes.flatMap((envelope) =>
    (envelope[1] as EnvelopeItem[]).filter(([header]) => header.type === type).map(([, payload]) => payload as T),
  );
}

/**
 * Runs the test app with a scenario and collects the envelopes the mock
 * server receives until `done` returns true.
 *
 * The app is killed if it is still running after `done` or the timeout.
 */
export async function runScenario(
  scenario: string,
  done: (envelopes: Envelope[]) => boolean,
  timeoutMs = 30_000,
): Promise<Envelope[]> {
  const envelopes: Envelope[] = [];
  const server = await createSentryTestServer((envelope) => envelopes.push(envelope));

  let log = '';
  let exited = false;
  const app = spawn(BINARY, [], {
    env: { ...process.env, SENTRY_E2E_SCENARIO: scenario, SENTRY_E2E_DSN: server.dsn },
  });
  app.stdout.on('data', (data) => {
    log += data;
  });
  app.stderr.on('data', (data) => {
    log += data;
  });
  // Use `exit` and not `close`. The crash reporter process inherits the
  // stdio pipes, so `close` waits for it as well.
  app.on('exit', (code, signal) => {
    exited = true;
    log += `\n[app exited with code ${code} signal ${signal}]\n`;
  });
  app.on('error', (error) => {
    exited = true;
    log += `\n[failed to start app: ${error}]\n`;
  });

  try {
    const start = Date.now();
    while (!done(envelopes)) {
      if (Date.now() - start > timeoutMs) {
        throw new Error(
          `Timed out after ${timeoutMs}ms in scenario '${scenario}'.\n` +
            `Received envelopes:\n${JSON.stringify(envelopes, null, 2)}\nApp output:\n${log}`,
        );
      }
      await delay(100);
    }

    // Give the app time to exit by itself.
    for (let i = 0; i < 50 && !exited; i++) {
      await delay(100);
    }

    if (process.env.DEBUG) {
      console.log(log);
    }

    return envelopes;
  } finally {
    if (!exited) {
      app.kill('SIGKILL');
    }
    await server.close();
  }
}
