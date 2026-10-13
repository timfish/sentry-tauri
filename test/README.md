# End-to-end tests

The e2e tests run a real Tauri app against a mock Sentry server and check
the envelopes it receives.

## Run

```shell
> yarn install
> yarn e2e
```

`yarn e2e` runs two steps that you can also run alone:

- `yarn e2e:build` bundles the frontend with Rollup and builds the test app
  with `cargo build`. The app enables the `tauri/custom-protocol` feature,
  so the webview loads the bundled frontend and not a dev server.
- `yarn e2e:test` runs the tests with Vitest.

Set `DEBUG=1` to print the app output for each test.

On Linux, run the tests under `xvfb-run` if there is no display.

## How it works

- `e2e/app` is the test app. It is one binary for all scenarios.
- `e2e/runner.ts` starts the mock server from `e2e/server.ts`, then starts
  the app with these environment variables:
  - `SENTRY_E2E_SCENARIO`: the scenario to run
  - `SENTRY_E2E_DSN`: the DSN of the mock server
- The frontend in `e2e/app/src/main.js` gets the scenario from the
  `scenario` command and runs it. Then it flushes Sentry and calls the
  `exit` command.
- The runner waits until the envelopes it needs arrive, then stops the
  app. If they do not arrive before the timeout, the test fails and shows
  the app output.

## Add a scenario

1. Add a function to `scenarios` in `e2e/app/src/main.js`. If it needs
   Rust code, add a command to `e2e/app/src-tauri/src/main.rs`.
2. Add a test to `e2e/e2e.test.ts` that calls `runScenario` with the
   scenario name and a function that returns `true` when the envelopes it
   needs have arrived. Check them with `toMatchObject`.
3. Run `yarn e2e`.
