# Feature modules

## Turn features on or off

Edit `modules.json` beside `Omibus Grid Research.exe`, then close and reopen the app:

```json
{
  "grid": true,
  "charts": false,
  "ai": true
}
```

`true` enables a feature; `false` disables it. Grid, Charts and AI Research are available and enabled in the shipped configuration. AI training starts only when you press Start daily learning (then resumes on future app launches until stopped). All modules may be disabled. Use actual JSON booleans, not quoted strings. JSON does not support comments. Unknown keys and invalid values are errors. Older files without `ai` default it to false, preserving existing installations.

The desktop reads `modules.json` from the executable's directory once at startup. If that file is absent, it uses the defaults compiled from the project's root `modules.json`. Thus the executable can run alone, but keep the configuration next to it to change features without rebuilding. For `cargo tauri dev`, put overrides beside the debug executable or change the project defaults and restart development.

Disabling Grid does not delete `paper-bots.json`. Re-enabling it restores access to the same saved bots. The application identifier and data location are unchanged.

These are **runtime feature switches**: disabled module assets are still packaged in the executable, but are not imported or mounted. Their styles are not loaded. Disabled Grid does not initialize or read its paper store, and its commands cannot execute. Disabled Charts creates no iframe or TradingView connection. Enabled Charts starts only on its first visit and remains mounted when switching tabs to retain the chart session. This does not provide compile-time removal or third-party plugin installation.

## Structure

```text
modules.json                    Feature switches / compiled defaults
ui/index.html                   Shared application shell
ui/shell.js                     Navigation, lazy loading, clock, error handling
ui/styles.css                   Shared design and controls
ui/core/modules.js              Configuration, lifecycle cache, command interface
ui/modules/registry.js          Explicit module registration
ui/modules/grid/                Grid view, styles and UI behavior
ui/modules/charts/              Chart view, styles and UI behavior
ui/modules/ai/                  Live research chart, input selection and custom rules
src-tauri/src/modules/ai.rs     Training supervisor and read-only chart command
ai-worker/                     Research models, indicators, patterns and runtime
src-tauri/src/main.rs           Backend composition and command registration
src-tauri/src/module_config.rs  Configuration validation and feature guards
src-tauri/src/modules/grid.rs   Grid commands and paper-store lifecycle
crates/omibus-core/             Independent grid and paper calculations
```

## Add a future module

1. Create `ui/modules/<id>/index.js`, `view.js`, and `styles.css`. Scope CSS to `#<id>-page` and DOM queries to the supplied root so modules do not affect one another.
2. Export `mount(root, context)`. It may be async and may return `activate()`, `deactivate()` and `dispose()` lifecycle methods. The shell calls mount once on first use, activate on visits, deactivate when leaving, and dispose on page teardown. Modules own and clean up their timers, subscriptions, and external connections. Only keep background work while hidden when the feature needs it.
3. Add an entry to `ui/modules/registry.js` with id, label, shortLabel, icon, stylesheet URL, allowed command names, and a dynamic `load` import. No module should import another module's screen or manipulate its DOM.
4. Add a boolean field to `ModuleConfig` and a corresponding key to root `modules.json`. Update external configuration files when adding a module. New optional fields should default to false for older installations. AI Research is implemented; see [AI_RESEARCH.md](AI_RESEARCH.md).
5. If the module needs native services, place them in `src-tauri/src/modules/<id>.rs`, register their commands in `main.rs`, initialize only when enabled, and enforce the feature flag on every command. The frontend context is an organization boundary; the Rust guards are what enforce native access. Local modules are trusted application code, not a sandbox for untrusted plugins.
6. Put shared domain services behind explicit interfaces; pass them through context or native commands rather than reaching into another module's private state. Declare and validate feature dependencies before adding a module that requires another one.

Minimal UI module:

```js
export function mount(root, context) {
  root.textContent = 'New feature';
  return {
    activate() {},
    deactivate() {},
    dispose() { root.replaceChildren(); }
  };
}
```

## Validate and build

```powershell
node --test tests/modules.test.mjs tests/ai-settings.test.mjs
cargo test --workspace --locked
.\ai-worker\runtime\python.exe -I ai-worker\test.py
cargo tauri build --no-bundle -- --locked
Copy-Item -LiteralPath .\target\release\omibus-desktop.exe -Destination '.\Omibus Grid Research.exe' -Force
```

Close the app before replacing its executable. No npm installation is needed. `ui/package.json` only declares JavaScript module syntax for development tools.

For browser-only UI checks, run `node tools/preview.cjs` and open `http://127.0.0.1:4174/ui/`. The preview reads the same root configuration, but cannot run Rust paper commands. Desktop startup uses the native `get_modules` command instead of fetching a configuration URL.
