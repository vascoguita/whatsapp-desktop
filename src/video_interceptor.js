(function () {
  var TAG = "[video-interceptor]";
  var log = console.log.bind(console, TAG);

  var realCreateObjectURL = URL.createObjectURL.bind(URL);
  var realRevokeObjectURL = URL.revokeObjectURL.bind(URL);
  var blobRegistry = new Map();

  URL.createObjectURL = function (obj) {
    var url = realCreateObjectURL(obj);
    if (obj instanceof Blob && (!obj.type || obj.type.indexOf("video/") === 0)) {
      blobRegistry.set(url, obj);
      log("captured blob", url, obj.type || "(no type)", obj.size + "b");
    }
    return url;
  };

  URL.revokeObjectURL = function (url) {
    if (blobRegistry.delete(url)) {
      log("revoked blob", url);
    }
    return realRevokeObjectURL(url);
  };

  var extractions = new WeakMap();

  function overlay(video) {
    if (!document.getElementById("video-interceptor-style")) {
      var style = document.createElement("style");
      style.id = "video-interceptor-style";
      style.textContent = "@keyframes video-interceptor-spin{to{transform:rotate(360deg)}}";
      document.head.appendChild(style);
    }

    var el = document.createElement("div");
    el.style.cssText =
      "position:fixed;pointer-events:none;z-index:2147483647;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)";
    var spin = document.createElement("div");
    spin.style.cssText =
      "width:32px;height:32px;border-radius:50%;border:3px solid rgba(255,255,255,.35);border-top-color:#fff;animation:video-interceptor-spin .8s linear infinite";
    el.appendChild(spin);
    document.body.appendChild(el);

    function place() {
      if (!document.body.contains(video)) {
        el.style.display = "none";
        return;
      }
      el.style.display = "flex";
      var rect = video.getBoundingClientRect();
      el.style.left = rect.left + "px";
      el.style.top = rect.top + "px";
      el.style.width = rect.width + "px";
      el.style.height = rect.height + "px";
    }
    place();
    var id = setInterval(place, 150);

    return function () {
      clearInterval(id);
      el.remove();
    };
  }

  function blobToDataURL(blob) {
    return new Promise(function (resolve, reject) {
      var reader = new FileReader();
      reader.onload = function () {
        resolve(reader.result);
      };
      reader.onerror = function () {
        reject(reader.error);
      };
      reader.readAsDataURL(blob);
    });
  }

  function fetchToDataURL(src) {
    log("extracting via fetch (blob not captured)", src);
    var controller = new AbortController();
    var timer = setTimeout(function () {
      log("fetch timed out, aborting", src);
      controller.abort();
    }, 10000);

    var promise = fetch(src, { signal: controller.signal })
      .then(function (response) {
        return response.blob();
      })
      .then(blobToDataURL);

    function clearTimer() {
      clearTimeout(timer);
    }
    promise.then(clearTimer, clearTimer);
    return promise;
  }

  function withTimeout(promise, ms, onTimeout) {
    return new Promise(function (resolve, reject) {
      var timer = setTimeout(function () {
        onTimeout();
        reject(new Error("timed out after " + ms + "ms"));
      }, ms);
      promise.then(
        function (result) {
          clearTimeout(timer);
          resolve(result);
        },
        function (err) {
          clearTimeout(timer);
          reject(err);
        }
      );
    });
  }

  function extract(video, src) {
    var blob = blobRegistry.get(src);
    var started = Date.now();
    var promise;

    if (blob) {
      log("extracting via captured blob", src);
      promise = withTimeout(blobToDataURL(blob), 5000, function () {
        log("blob read stalled (likely revoked mid-read), falling back to fetch", src);
      }).catch(function () {
        return fetchToDataURL(src);
      });
    } else {
      promise = fetchToDataURL(src);
    }

    promise.then(
      function () {
        log("extraction succeeded", src, Date.now() - started + "ms");
      },
      function (err) {
        log("extraction failed", src, err);
      }
    );

    extractions.set(video, { src: src, promise: promise });
    return promise;
  }

  function blobSrc(video) {
    var src = video.currentSrc || video.src;
    return src && src.indexOf("blob:") === 0 ? src : null;
  }

  function ensureExtraction(video, src) {
    var existing = extractions.get(video);
    return existing && existing.src === src ? existing.promise : extract(video, src);
  }

  function swap(video) {
    var src = blobSrc(video);
    if (!src) {
      log("nothing to swap, proceeding to native play", video.currentSrc || video.src);
      return Promise.resolve();
    }

    log("swapping before play", src);
    var promise = ensureExtraction(video, src);
    var hide = overlay(video);

    return promise
      .then(function (dataUrl) {
        video.src = dataUrl;
        video.load();
        extractions.delete(video);
        log("swapped to data URL, native play proceeding", src);
      })
      .catch(function (err) {
        log("swap failed, falling back to native blob playback", src, err);
      })
      .then(hide);
  }

  var nativePlay = HTMLMediaElement.prototype.play;
  HTMLMediaElement.prototype.play = function () {
    var video = this;
    if (video.tagName !== "VIDEO") {
      return nativePlay.call(video);
    }
    log("play() called", video.currentSrc || video.src, video);
    return swap(video).then(function () {
      return nativePlay.call(video);
    });
  };

  function attach(video) {
    if (video.__videoInterceptorAttached) {
      return;
    }
    video.__videoInterceptorAttached = true;
    log("attached to video element", video);

    video.addEventListener(
      "loadstart",
      function () {
        var src = blobSrc(video);
        if (src) {
          log("loadstart, prefetching extraction", src);
          ensureExtraction(video, src);
        }
      },
      true
    );
  }

  function scan(node) {
    if (node.tagName === "VIDEO") {
      attach(node);
    }
    if (node.querySelectorAll && node.childElementCount) {
      node.querySelectorAll("video").forEach(attach);
    }
  }

  new MutationObserver(function (mutations) {
    mutations.forEach(function (mutation) {
      mutation.addedNodes.forEach(function (node) {
        if (node.nodeType === 1) {
          scan(node);
        }
      });
    });
  }).observe(document.documentElement, { childList: true, subtree: true });

  document.addEventListener("DOMContentLoaded", function () {
    scan(document.documentElement);
  });

  log("initialized");
})();
