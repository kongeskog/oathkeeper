import init, { assess_policy_json, kernel_version } from './kernel/oathkeeper_wasm.js?v=0.5.0';
const ready = init({module_or_path:new URL('./kernel/oathkeeper_wasm_bg.wasm?v=0.5.0',import.meta.url)});
self.onmessage = async ({data}) => {
  try {
    await ready;
    if (kernel_version() !== '0.5.0') throw new Error('Kernel version mismatch. Reload the application.');
    self.postMessage({result:JSON.parse(assess_policy_json(data.request))});
  } catch(error) { self.postMessage({error:String(error)}); }
};
