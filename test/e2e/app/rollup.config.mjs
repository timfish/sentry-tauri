import { fileURLToPath } from 'node:url';
import resolve from '@rollup/plugin-node-resolve';

export default {
  input: fileURLToPath(new URL('./src/main.js', import.meta.url)),
  output: {
    file: fileURLToPath(new URL('./dist/main.js', import.meta.url)),
    format: 'es',
  },
  plugins: [resolve({ browser: true })],
};
