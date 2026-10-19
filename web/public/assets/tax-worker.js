import init, { assess_backdoor_roth_json, kernel_version } from './kernel/oathkeeper_wasm.js?v=0.4.1';
const ready = init();
self.onmessage = async ({data}) => {
  try {
    await ready;
    if (kernel_version() !== '0.4.1') throw new Error('Kernel version mismatch. Reload the application.');
    self.postMessage({result:JSON.parse(assess_backdoor_roth_json(data.input))});
  } catch(error) {
    self.postMessage({error:String(error)});
  }
};
