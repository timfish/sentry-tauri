import { getCurrentScope, getGlobalScope, getIsolationScope } from '@sentry/browser';
import { invoke } from '@tauri-apps/api/core';
function mergedScope() {
    const scopes = [getGlobalScope(), getIsolationScope(), getCurrentScope()].map((s) => s.getScopeData());
    const merge = (key) => Object.assign({}, ...scopes.map((s) => s[key]));
    return { user: merge('user'), tags: merge('tags'), extra: merge('extra'), contexts: merge('contexts') };
}
function toJson(value) {
    try {
        return JSON.stringify(value);
    }
    catch {
        return undefined;
    }
}
function diff(sent, next) {
    const changes = {};
    const keys = new Set();
    for (const [key, value] of Object.entries(next)) {
        const json = value === undefined ? undefined : toJson(value);
        if (json === undefined) {
            continue;
        }
        keys.add(key);
        if (sent.get(key) !== json) {
            sent.set(key, json);
            changes[key] = value;
        }
    }
    for (const key of sent.keys()) {
        if (!keys.has(key)) {
            sent.delete(key);
            changes[key] = null;
        }
    }
    return Object.keys(changes).length ? changes : undefined;
}
/**
 * Creates a tracker that returns only what changed since the last call.
 */
export function createScopeDiffer() {
    const sentUser = new Map();
    const sentTags = new Map();
    const sentExtra = new Map();
    const sentContexts = new Map();
    return (scope) => {
        const update = {};
        if (diff(sentUser, (scope.user || {}))) {
            update.user = sentUser.size ? Object.fromEntries([...sentUser].map(([k, v]) => [k, JSON.parse(v)])) : null;
        }
        const tags = diff(sentTags, scope.tags || {});
        if (tags)
            update.tags = tags;
        const extra = diff(sentExtra, scope.extra || {});
        if (extra)
            update.extra = extra;
        const contexts = diff(sentContexts, scope.contexts || {});
        if (contexts)
            update.contexts = contexts;
        return Object.keys(update).length ? update : undefined;
    };
}
/**
 * Sends user, tags, extra and contexts to the Rust SDK when they change.
 * Breadcrumbs go through `beforeBreadcrumb` instead.
 */
export function scopeToRustIntegration() {
    return {
        name: 'ScopeToRust',
        setup() {
            const differ = createScopeDiffer();
            let failed = false;
            let queued = false;
            const send = () => {
                queued = false;
                const scope = differ(mergedScope());
                if (!scope || failed) {
                    return;
                }
                invoke('plugin:sentry|scope', { scope }).catch((e) => {
                    console.error('Failed to send scope to Rust:', e);
                    failed = true;
                });
            };
            // Many scope changes in one task cause one IPC call
            const listener = () => {
                if (!queued) {
                    queued = true;
                    queueMicrotask(send);
                }
            };
            getGlobalScope().addScopeListener(listener);
            getIsolationScope().addScopeListener(listener);
            getCurrentScope().addScopeListener(listener);
            listener();
        },
    };
}
