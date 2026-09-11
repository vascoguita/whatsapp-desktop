(function () {
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

  function extract(video, src) {
    var controller = new AbortController();
    var timer = setTimeout(function () {
      controller.abort();
    }, 10000);

    var promise = fetch(src, { signal: controller.signal })
      .then(function (response) {
        return response.blob();
      })
      .then(function (blob) {
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
      });

    function clearTimer() {
      clearTimeout(timer);
    }
    promise.then(clearTimer, clearTimer);
    promise.catch(function () {});

    extractions.set(video, { src: src, promise: promise });
    return promise;
  }

  function blobSrc(video) {
    var src = video.currentSrc || video.src;
    return src && src.indexOf("blob:") === 0 && !video.autoplay ? src : null;
  }

  function ensureExtraction(video, src) {
    var existing = extractions.get(video);
    return existing && existing.src === src ? existing.promise : extract(video, src);
  }

  function swap(video) {
    var src = blobSrc(video);
    if (!src) {
      return Promise.resolve();
    }

    var promise = ensureExtraction(video, src);
    var hide = overlay(video);

    return promise
      .then(function (dataUrl) {
        video.src = dataUrl;
        video.load();
        extractions.delete(video);
      })
      .catch(function (err) {
        console.warn("video interception failed:", err);
      })
      .then(hide);
  }

  var nativePlay = HTMLMediaElement.prototype.play;
  HTMLMediaElement.prototype.play = function () {
    var video = this;
    if (video.tagName !== "VIDEO") {
      return nativePlay.call(video);
    }
    return swap(video).then(function () {
      return nativePlay.call(video);
    });
  };

  function attach(video) {
    if (video.__videoInterceptorAttached) {
      return;
    }
    video.__videoInterceptorAttached = true;

    video.addEventListener(
      "loadstart",
      function () {
        var src = blobSrc(video);
        if (src) {
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
})();
