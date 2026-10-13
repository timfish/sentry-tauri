// The same snippet that @sentry/vite-plugin adds to each output file.
!(function () {
  try {
    var e = typeof window !== 'undefined' ? window : {};
    var n = new e.Error().stack;
    n &&
      ((e._sentryDebugIds = e._sentryDebugIds || {}),
      (e._sentryDebugIds[n] = '3bd0e8d1-0f5c-4d6e-9a43-1d2b6c7e8f90'),
      (e._sentryDebugIdIdentifier = 'sentry-dbid-3bd0e8d1-0f5c-4d6e-9a43-1d2b6c7e8f90'));
  } catch (e) {}
})();

window.throwWithDebugId = function () {
  throw new Error('debug id error');
};
