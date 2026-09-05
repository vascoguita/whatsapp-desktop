(function () {
  var invoke = window.__TAURI_INTERNALS__.invoke;
  var LEVELS = { warn: 4, error: 5 };

  function stringify(arg) {
    if (typeof arg === "string") {
      return arg;
    }
    if (arg instanceof Error) {
      return arg.stack || arg.message;
    }
    if (typeof arg !== "object" || arg === null) {
      return String(arg);
    }
    try {
      return JSON.stringify(arg, null, 2);
    } catch (e) {
      return String(arg);
    }
  }

  function forward(level, message) {
    invoke("plugin:log|log", { level: level, message: message }).catch(function () {});
  }

  Object.keys(LEVELS).forEach(function (method) {
    var original = console[method];
    console[method] = function (...args) {
      original.apply(console, args);
      forward(LEVELS[method], args.map(stringify).join(" "));
    };
  });

  window.addEventListener("error", function (event) {
    var message = event.error ? stringify(event.error) : event.message || "unknown error";
    forward(LEVELS.error, "uncaught exception: " + message);
  });

  window.addEventListener("unhandledrejection", function (event) {
    forward(LEVELS.error, "unhandled promise rejection: " + stringify(event.reason));
  });
})();
