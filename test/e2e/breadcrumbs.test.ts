import type { Breadcrumb, Envelope } from '@sentry/core';
import { describe, expect, test } from 'vitest';
import { itemsOfType, runScenario } from './runner';

const hasEvent = (envelopes: Envelope[]) => itemsOfType(envelopes, 'event').length > 0;

const tauriBreadcrumb = (message: string, data?: Record<string, unknown>) =>
  expect.objectContaining({
    type: 'ui',
    category: 'tauri',
    message,
    ...(data ? { data: expect.objectContaining(data) } : {}),
  });

describe('Tauri breadcrumbs', () => {
  test('JavaScript error has breadcrumbs for app, window and webview events', async () => {
    const envelopes = await runScenario('js-error', hasEvent);
    const [event] = itemsOfType(envelopes, 'event');
    const breadcrumbs = (event.breadcrumbs as { values: Breadcrumb[] } | undefined)?.values ?? [];

    expect(breadcrumbs).toEqual(
      expect.arrayContaining([
        tauriBreadcrumb('app.ready'),
        tauriBreadcrumb('window.created', { label: 'main' }),
        tauriBreadcrumb('webview.created', { label: 'main' }),
        tauriBreadcrumb('webview.page-load-started', { label: 'main', url: expect.stringContaining('localhost') }),
        tauriBreadcrumb('webview.page-load-finished', { label: 'main', url: expect.stringContaining('localhost') }),
      ]),
    );

    const tauri = breadcrumbs.filter((b) => b.category === 'tauri');
    expect(tauri.some((b) => b.data && 'title' in b.data)).toBe(false);
    expect(JSON.stringify(tauri)).not.toContain('sentry-tauri-e2e');
  });
});
