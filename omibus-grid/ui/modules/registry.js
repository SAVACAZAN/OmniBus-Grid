// Explicit module registration. Configuration cannot provide arbitrary code paths.
export const registry = [
  {
    id: 'grid', label: 'Grid workspace', shortLabel: 'GRID', icon: '▦',
    stylesheet: new URL('./grid/styles.css', import.meta.url).href,
    commands: ['preview_grid', 'list_bots', 'create_paper_bot', 'tick_bot', 'bot_action', 'delete_bot'],
    load: () => import('./grid/index.js')
  },
  {
    id: 'charts', label: 'Market charts', shortLabel: 'CHARTS', icon: '⌁',
    stylesheet: new URL('./charts/styles.css', import.meta.url).href,
    commands: [], load: () => import('./charts/index.js')
  },
  {
    id: 'ai', label: 'AI Research', shortLabel: 'AI LAB', icon: '✧',
    stylesheet: new URL('./ai/styles.css', import.meta.url).href,
    commands: ['ai_start', 'ai_stop', 'ai_status', 'ai_chart_snapshot'], load: () => import('./ai/index.js')
  }
];
