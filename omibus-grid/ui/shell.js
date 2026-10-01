import { registry } from './modules/registry.js';
import { enabledModules, moduleContext, moduleLoader } from './core/modules.js';

const nav = document.getElementById('module-nav');
const pages = document.getElementById('module-pages');
const message = document.getElementById('shell-message');
const clock = () => { document.getElementById('clock').textContent = new Date().toISOString().slice(11, 19) + ' UTC'; };
clock();
const clockTimer = setInterval(clock, 1000);
const nativeInvoke = window.__TAURI__?.core?.invoke;
const invoke = (name, args) => nativeInvoke ? nativeInvoke(name, args) : Promise.reject(new Error('Open the desktop app to use the paper engine.'));

async function readConfig() {
  if (nativeInvoke) return invoke('get_modules');
  const response = await fetch('../modules.json');
  if (!response.ok) throw new Error('Cannot load modules.json. For browser preview, serve the repository root and open /ui/.');
  return response.json();
}
function loadStyle(href) {
  return new Promise((resolve, reject) => {
    const link = document.createElement('link'); link.rel = 'stylesheet'; link.href = href;
    link.onload = () => resolve(link);
    link.onerror = () => { link.remove(); reject(new Error('Cannot load module stylesheet.')); };
    document.head.append(link);
  });
}
let loader;
async function start() {
  const enabled = enabledModules(await readConfig(), registry);
  if (!enabled.length) {
    message.textContent = 'No modules enabled. Set a module to true in modules.json beside the executable, then restart.';
    return;
  }
  loader = moduleLoader(enabled, async (definition, code) => {
    const style = await loadStyle(definition.stylesheet);
    const root = document.createElement('main');
    root.id = `${definition.id}-page`; root.className = `${definition.id}-page`; root.hidden = true;
    pages.append(root);
    try {
      const instance = await code.mount(root, moduleContext(definition, invoke));
      return { root, activate: () => instance?.activate?.(), deactivate: () => instance?.deactivate?.(),
        dispose: () => { instance?.dispose?.(); root.remove(); style.remove(); } };
    } catch (error) { root.remove(); style.remove(); throw error; }
  });
  let active;
  let loading = false;
  async function activate(definition) {
    if (loading) return;
    loading = true;
    const buttons = [...nav.querySelectorAll('button')];
    buttons.forEach(button => { button.disabled = true; });
    message.hidden = false; message.classList.remove('error'); message.textContent = `Opening ${definition.label}…`;
    let next;
    try {
      next = await loader.get(definition.id);
      if (active && active !== next) { await active.deactivate(); active.root.hidden = true; }
      next.root.hidden = false;
      await next.activate(); active = next;
      for (const button of buttons) {
        const selected = button.id === `nav-${definition.id}`;
        button.classList.toggle('active', selected); button.setAttribute('aria-pressed', String(selected));
      }
      document.getElementById('module-label').textContent = definition.shortLabel;
      message.hidden = true;
    } catch (error) {
      if (next && next !== active) next.root.hidden = true;
      if (active) active.root.hidden = false;
      message.textContent = `${error.message || error} Select the module again to retry.`;
      message.classList.add('error');
    } finally { loading = false; buttons.forEach(button => { button.disabled = false; }); }
  }
  for (const definition of enabled) {
    const button = document.createElement('button');
    button.type = 'button'; button.id = `nav-${definition.id}`; button.className = 'rail-tab';
    button.textContent = definition.icon; button.title = definition.label;
    button.setAttribute('aria-label', definition.label); button.setAttribute('aria-pressed', 'false');
    button.setAttribute('aria-controls', `${definition.id}-page`);
    button.addEventListener('click', () => activate(definition)); nav.append(button);
  }
  await activate(enabled[0]);
}
window.addEventListener('pagehide', () => { clearInterval(clockTimer); void loader?.dispose(); }, { once: true });
start().catch(error => { message.textContent = `Cannot start workspace: ${error.message || error}`; message.classList.add('error'); });
