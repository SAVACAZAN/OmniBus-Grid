export function enabledModules(config, registry) {
  if (!config || typeof config !== 'object' || Array.isArray(config)) throw new Error('Module configuration must be an object.');
  const known = new Set(registry.map(module => module.id));
  for (const key of Object.keys(config)) if (!known.has(key)) throw new Error(`Unknown module: ${key}`);
  for (const { id } of registry) if (typeof config[id] !== 'boolean') throw new Error(`Module ${id} must be true or false.`);
  return registry.filter(module => config[module.id]);
}

export function moduleContext(definition, invoke) {
  return Object.freeze({
    invoke(name, args = {}) {
      if (!definition.commands.includes(name)) return Promise.reject(new Error(`Command ${name} is not owned by ${definition.id}.`));
      return invoke(name, args);
    }
  });
}

export function moduleLoader(enabled, mount) {
  const instances = new Map();
  return {
    async get(id) {
      const definition = enabled.find(module => module.id === id);
      if (!definition) throw new Error(`Module ${id} is disabled or unknown.`);
      if (!instances.has(id)) {
        const pending = Promise.resolve().then(() => definition.load()).then(code => mount(definition, code));
        instances.set(id, pending);
        pending.catch(() => instances.delete(id));
      }
      return instances.get(id);
    },
    async dispose() {
      const settled = await Promise.allSettled(instances.values());
      for (const result of settled) if (result.status === 'fulfilled') result.value.dispose?.();
      instances.clear();
    }
  };
}
