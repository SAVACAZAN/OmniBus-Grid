import test from 'node:test';
import assert from 'node:assert/strict';
import { enabledModules, moduleLoader, moduleContext } from '../ui/core/modules.js';
import { registry as actualRegistry } from '../ui/modules/registry.js';

test('all eight app feature combinations isolate AI from Grid and Charts', async () => {
  for (const grid of [false, true]) for (const charts of [false, true]) for (const ai of [false, true]) {
    const enabled = enabledModules({ grid, charts, ai }, actualRegistry);
    assert.deepEqual(enabled.map(m => m.id), Object.entries({ grid, charts, ai }).filter(([, on]) => on).map(([id]) => id));
  }
  const definition = actualRegistry.find(m => m.id === 'ai');
  const context = moduleContext(definition, async () => {});
  await assert.rejects(context.invoke('tick_bot'), /not owned/);
});

test('all four configurations expose only enabled modules and never import disabled code', async () => {
  for (const grid of [false, true]) for (const charts of [false, true]) {
    const imports = [];
    const registry = ['grid', 'charts'].map(id => ({ id, load: async () => { imports.push(id); return {}; } }));
    const enabled = enabledModules({ grid, charts }, registry);
    const loader = moduleLoader(enabled, async () => ({}));
    assert.deepEqual(imports, []);
    for (const id of ['grid', 'charts']) {
      if ({ grid, charts }[id]) await loader.get(id);
      else await assert.rejects(loader.get(id), /disabled/);
    }
    assert.deepEqual(imports, enabled.map(module => module.id));
  }
});

test('concurrent activation mounts once, preserves state and cleans up once', async () => {
  let mounts = 0, disposed = 0;
  const loader = moduleLoader([{ id: 'grid', load: async () => ({}) }], async () => {
    mounts++;
    return { value: 7, dispose: () => disposed++ };
  });
  const [a, b] = await Promise.all([loader.get('grid'), loader.get('grid')]);
  assert.equal(a, b);
  a.value = 42;
  assert.equal((await loader.get('grid')).value, 42);
  assert.equal(mounts, 1);
  await loader.dispose();
  assert.equal(disposed, 1);
});

test('one failed module can retry without breaking another', async () => {
  let attempts = 0;
  const loader = moduleLoader([
    { id: 'grid', load: async () => ({}) },
    { id: 'charts', load: async () => { if (++attempts === 1) throw new Error('offline'); return {}; } }
  ], async () => ({}));
  const grid = await loader.get('grid');
  await assert.rejects(loader.get('charts'), /offline/);
  await loader.get('charts');
  assert.equal(await loader.get('grid'), grid);
  assert.equal(attempts, 2);
});

test('missing, unknown and non-boolean settings are rejected', () => {
  const registry = [{ id: 'grid' }, { id: 'charts' }];
  for (const config of [null, [], {}, { grid: 'false', charts: true }, { grid: true, charts: true, ai: true }]) {
    assert.throws(() => enabledModules(config, registry));
  }
});

test('Charts cannot call Grid backend commands through its context', async () => {
  const calls = [];
  const invoke = async (...args) => calls.push(args);
  const charts = moduleContext({ id: 'charts', commands: [] }, invoke);
  await assert.rejects(charts.invoke('create_paper_bot'), /not owned/);
  assert.equal(calls.length, 0);
  const grid = moduleContext({ id: 'grid', commands: ['list_bots'] }, invoke);
  await grid.invoke('list_bots');
  assert.deepEqual(calls, [['list_bots', {}]]);
});
