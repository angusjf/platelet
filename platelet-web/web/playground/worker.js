let render_files;

self.onmessage = async function (evt) {
  let data = evt.data;
  if (!render_files) {
    let platelet = await import("https://angusjf.com/platelet/pkg/platelet_web.js");
    await platelet.default();
    console.log(render_files);
    render_files = platelet.render_files;
  }
  try {
    let result = render_files(data.root, JSON.stringify(data.files), data.json);
    postMessage({ ok: true, result });
  } catch (error) {
    console.error(error);
    postMessage({ ok: false, error: error.toString() });
  }
};

postMessage({ ok: true, result: "loaded!" });
