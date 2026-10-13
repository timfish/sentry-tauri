import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    root: fileURLToPath(new URL('.', import.meta.url)),
    include: ['./**/*.test.ts'],
    fileParallelism: false,
    testTimeout: 60_000,
    retry: process.env.CI ? 2 : 0,
    reporters: ['verbose'],
  },
});
