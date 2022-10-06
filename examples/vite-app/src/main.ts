import { invoke } from '@tauri-apps/api';
import * as Sentry from '../../../'

Sentry.init()

function aFunctionThatThrows() {
  throw new Error("This is a JavaScript Error");
}

window.addEventListener("DOMContentLoaded", () => {
  window.document
    .getElementById("js-breadcrumb")
    ?.addEventListener("click", () => {
      Sentry.addBreadcrumb({
        message: "This is a breadcrumb from JavaScript",
      });
    });

  window.document
    .getElementById("js-error")
    ?.addEventListener("click", () => {
      aFunctionThatThrows();
    });

  window.document
    .getElementById("rust-breadcrumb")
    ?.addEventListener("click", () => {
      invoke("rust_breadcrumb");
    });

  window.document
    .getElementById("rust-panic")
    ?.addEventListener("click", () => {
      invoke("rust_panic");
    });

  window.document
    .getElementById("native-crash")
    ?.addEventListener("click", () => {
      invoke("native_crash");
    });
});
