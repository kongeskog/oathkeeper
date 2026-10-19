import init, { analyze_json, kernel_version } from './kernel/oathkeeper_wasm.js';
const ready = init();
self.onmessage = async ({ data }) => {
  try {
    await ready;
    if (kernel_version() !== '0.4.1') throw new Error('Kernel version mismatch. Reload the application.');
    const result = JSON.parse(analyze_json(data.context, data.sensitivity));
    self.postMessage({ result });
  } catch (error) {
    self.postMessage({ error: String(error) });
  }
};
