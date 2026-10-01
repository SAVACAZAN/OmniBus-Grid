function sameTriggers(a, b) {
  return ['buy','sell'].every(side => Array.isArray(a?.[side]) && Array.isArray(b?.[side]) &&
    a[side].length === b[side].length && a[side].every((rule,i) => {
      const other=b[side][i];
      return rule.left===other.left && rule.op===other.op && rule.right===other.right &&
        (rule.right!=='number' || rule.value===other.value);
    }));
}

export function sameProfile(a, b) {
  if (!a || !b) return false;
  return Object.keys(a).length === Object.keys(b).length && Object.keys(a).every(key =>
    key === 'inputs' ? Array.isArray(b.inputs) && JSON.stringify([...a.inputs].sort()) === JSON.stringify([...b.inputs].sort()) : key === 'triggers' ? sameTriggers(a[key],b[key]) : a[key] === b[key]);
}

export function configFromForm(data) {
  const config = Object.fromEntries(data);
  config.inputs = data.getAll('inputs').sort();
  if (!config.inputs.length) throw new Error('Select at least one indicator or pattern to learn.');
  config.symbol = config.symbol.trim().toUpperCase();
  config.triggers = {buy:[],sell:[]};
  for (const name of ['horizon','history_bars','fee_bps','slippage_bps','threshold','take_profit_pct']) config[name] = Number(config[name]);
  return config;
}

export function chartConfig(config) { return {symbol:config.symbol,interval:config.interval,inputs:[...config.inputs].sort(),triggers:config.triggers}; }

export function chartMatchesModel(chart,config) { return sameProfile(chartConfig(chart),chartConfig(config)); }

export function selectedFeatureCount(catalog, inputs) {
  return new Set(catalog.inputs.filter(item => inputs.includes(item.id)).flatMap(item => item.features)).size;
}
