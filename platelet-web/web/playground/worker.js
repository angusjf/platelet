self.onmessage = async function (evt) {
  let data = evt.data;
  let { default: init, render_files } = await import("https://angusjf.com/platelet/pkg/platelet_web.js");
  await init();
  let result = render_files(data.root, JSON.stringify(data.files), data.json);
  postMessage(result);
};
