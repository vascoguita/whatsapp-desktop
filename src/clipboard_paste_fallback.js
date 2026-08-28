(function () {
  document.addEventListener(
    "paste",
    function (event) {
      if (event.clipboardData && event.clipboardData.items && event.clipboardData.items.length > 0) {
        return;
      }
      var invoke = window.__TAURI_INTERNALS__.invoke;
      invoke("plugin:clipboard-manager|read_image")
        .then(function (rid) {
          return Promise.all([invoke("plugin:image|rgba", { rid: rid }), invoke("plugin:image|size", { rid: rid })]);
        })
        .then(function (results) {
          var canvas = document.createElement("canvas");
          canvas.width = results[1].width;
          canvas.height = results[1].height;
          canvas
            .getContext("2d")
            .putImageData(new ImageData(new Uint8ClampedArray(results[0]), results[1].width, results[1].height), 0, 0);
          canvas.toBlob(function (blob) {
            if (!blob) {
              return;
            }
            var dataTransfer = new DataTransfer();
            dataTransfer.items.add(new File([blob], "pasted-image.png", { type: "image/png" }));
            (event.target || document.activeElement || document.body).dispatchEvent(
              new ClipboardEvent("paste", { clipboardData: dataTransfer, bubbles: true, cancelable: true })
            );
          }, "image/png");
        })
        .catch(function () {});
    },
    true
  );
})();
