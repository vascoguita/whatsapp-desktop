(() => {
  document.addEventListener(
    "paste",
    (event) => {
      if (event.clipboardData?.items && event.clipboardData.items.length > 0) {
        return;
      }
      var invoke = window.__TAURI_INTERNALS__.invoke;
      invoke("plugin:clipboard-manager|read_image")
        .then((rid) =>
          Promise.all([
            invoke("plugin:image|rgba", { rid: rid }),
            invoke("plugin:image|size", { rid: rid }),
          ]),
        )
        .then((results) => {
          var canvas = document.createElement("canvas");
          canvas.width = results[1].width;
          canvas.height = results[1].height;
          canvas
            .getContext("2d")
            .putImageData(
              new ImageData(
                new Uint8ClampedArray(results[0]),
                results[1].width,
                results[1].height,
              ),
              0,
              0,
            );
          canvas.toBlob((blob) => {
            if (!blob) {
              console.warn(
                "clipboard image paste fallback: canvas.toBlob produced no blob",
              );
              return;
            }
            var dataTransfer = new DataTransfer();
            dataTransfer.items.add(
              new File([blob], "pasted-image.png", { type: "image/png" }),
            );
            (
              event.target ||
              document.activeElement ||
              document.body
            ).dispatchEvent(
              new ClipboardEvent("paste", {
                clipboardData: dataTransfer,
                bubbles: true,
                cancelable: true,
              }),
            );
          }, "image/png");
        })
        .catch((err) => {
          console.warn("clipboard image paste fallback failed:", err);
        });
    },
    true,
  );
})();
