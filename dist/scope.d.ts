import type { Integration, ScopeData } from '@sentry/core';
type Fields = 'user' | 'tags' | 'extra' | 'contexts';
/**
 * The changes since the last update. A `null` value removes the key from
 * the Rust scope, and a `null` user clears the user.
 */
export interface ScopeUpdate {
    user?: Record<string, unknown> | null;
    tags?: Record<string, unknown>;
    extra?: Record<string, unknown>;
    contexts?: Record<string, unknown>;
}
/**
 * Creates a tracker that returns only what changed since the last call.
 */
export declare function createScopeDiffer(): (scope: Pick<ScopeData, Fields>) => ScopeUpdate | undefined;
/**
 * Sends user, tags, extra and contexts to the Rust SDK when they change.
 * Breadcrumbs go through `beforeBreadcrumb` instead.
 */
export declare function scopeToRustIntegration(): Integration;
export {};
