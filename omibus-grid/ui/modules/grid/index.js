import view from './view.js';
export async function mount(root, context) {
root.innerHTML = view;
/* Omibus Grid UI. All strategy calculations and state changes run in Rust. */
const $ = (selector) => root.querySelector(selector);
const state = { bots: [], selectedId: null, preview: null, busy: false };

function el(tag, cls, value) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (value !== undefined) n.textContent = String(value);
  return n;
}
function fmt(value, digits = 6) {
  if (value === null || value === undefined || !Number.isFinite(Number(value))) return '—';
  const n = Number(value);
  if (n !== 0 && Math.abs(n) < 0.000001) return n.toPrecision(4);
  return n.toLocaleString('en-US', { maximumFractionDigits: digits });
}
function notice(message, error = false) {
  const n = $('#notice');
  n.textContent = String(message);
  n.classList.toggle('error', error);
  n.hidden = false;
}
function value(id) { return Number($(`#${id}`).value); }
function config() {
  return {
    symbol: $('#symbol').value.trim(),
    lower_price: value('lower-price'), upper_price: value('upper-price'),
    nr_of_grids: value('grid-count'), amount: value('amount'),
    amount_type: $('#amount-type').value, amount_unit: $('#amount-unit').value,
    incremental_pct_buy: value('increment-buy'), incremental_pct_sell: value('increment-sell'),
    side: $('#side').value, grid_type: $('#grid-type').value, fee_pct: value('fee')
  };
}
const command = context.invoke;
async function busy(work) {
  if (state.busy) return;
  state.busy = true;
  const controls = [...root.querySelectorAll('button')].map(button => [button, button.disabled]);
  controls.forEach(([button]) => { button.disabled = true; });
  try { await work(); }
  catch (error) { notice(String(error), true); }
  finally { state.busy = false; controls.forEach(([button, disabled]) => { button.disabled = disabled; }); }
}

function renderPreview(p) {
  $('#preview-empty').hidden = true;
  $('#preview-content').hidden = false;
  $('#map-low').textContent = fmt(p.levels[0]);
  $('#map-high').textContent = fmt(p.levels[p.levels.length - 1]);
  $('#map-current').textContent = fmt(value('current-price'));
  $('#buy-count').textContent = p.buy_orders;
  $('#sell-count').textContent = p.sell_orders;
  $('#quote-needed').textContent = fmt(p.buy_reserve_quote, 2);
  $('#base-needed').textContent = fmt(p.sell_reserve_base);
  $('#order-total').textContent = `${p.orders.length} LEVELS ACTIVE`;
  const body = $('#preview-orders'); body.replaceChildren();
  for (const order of p.orders) {
    const row = el('tr');
    row.append(el('td', '', String(order.level).padStart(2, '0')));
    const sideCell = el('td'); sideCell.append(el('span', `side-pill ${order.side}`, order.side.toUpperCase()));
    row.append(sideCell, el('td', '', fmt(order.price)), el('td', '', fmt(order.amount, 8)), el('td', '', fmt(order.total, 2)));
    body.append(row);
  }
}
async function preview() {
  const p = await command('preview_grid', { config: config(), currentPrice: value('current-price') });
  state.preview = p;
  renderPreview(p);
  return p;
}
async function refreshBots() {
  state.bots = await command('list_bots');
  state.bots.sort((a, b) => b.id - a.id);
  if (!state.bots.some(b => b.id === state.selectedId)) state.selectedId = state.bots[0]?.id ?? null;
  renderBots();
}
function renderBots() {
  $('#bot-count').textContent = `${state.bots.length} BOT${state.bots.length === 1 ? '' : 'S'}`;
  const list = $('#bot-list'); list.replaceChildren();
  if (!state.bots.length) list.append(el('div', 'list-placeholder', 'No bots yet. Create one above to begin.'));
  for (const b of state.bots) {
    const card = el('button', `bot-card ${b.id === state.selectedId ? 'selected' : ''}`);
    card.type = 'button';
    const top = el('div', 'bot-card-top'); top.append(el('strong', '', b.name), el('span', `status ${b.status}`, b.status));
    const bottom = el('div', 'bot-card-bottom');
    bottom.append(el('span', '', b.config.symbol), el('span', '', `${b.orders.filter(o => o.status === 'open').length} open · ${b.fills.length} fills`));
    card.append(top, bottom);
    card.addEventListener('click', () => { state.selectedId = b.id; renderBots(); });
    list.append(card);
  }
  renderDetail();
}
function metric(label, value, extraClass = '') {
  const div = el('div'); div.append(el('label', '', label), el('strong', extraClass, value)); return div;
}
function renderDetail() {
  const area = $('#bot-detail'); area.replaceChildren();
  const b = state.bots.find(x => x.id === state.selectedId);
  if (!b) { area.append(el('div', 'list-placeholder', 'Select a bot to see its wallet, orders, and fills.')); return; }
  const head = el('div', 'detail-head');
  const title = el('div'); title.append(el('h3', '', b.name), el('small', '', `${b.config.symbol} · #${b.id} · ${b.config.grid_type}`));
  head.append(title, el('span', `status ${b.status}`, b.status)); area.append(head);
  const w = b.wallet;
  const equity = w.quote_free + w.quote_locked + (w.base_free + w.base_locked) * b.current_price;
  const change = equity - (b.initial_quote + b.initial_base * b.current_price);
  const metrics = el('div', 'detail-metrics');
  metrics.append(metric('LAST PRICE', fmt(b.current_price)), metric('EQUITY / QUOTE', fmt(equity, 2)),
    metric('VS HOLD / QUOTE', `${change >= 0 ? '+' : ''}${fmt(change, 2)}`, change >= 0 ? 'buy' : 'sell'),
    metric('TOTAL FILLS', b.fills.length)); area.append(metrics);

  const tickRow = el('div', 'tick-row');
  const tickLabel = el('label', '', 'Feed a manual market price');
  const tickInput = el('input'); tickInput.type = 'number'; tickInput.step = 'any'; tickInput.min = '0'; tickInput.value = String(b.current_price);
  tickLabel.append(tickInput); tickRow.append(tickLabel);
  const tickButton = el('button', 'small-btn', 'Process price →'); tickButton.type = 'button';
  tickButton.disabled = b.status !== 'running';
  tickButton.addEventListener('click', () => busy(async () => {
    const before = b.fills.length;
    const updated = await command('tick_bot', { id: b.id, price: Number(tickInput.value) });
    notice(`${updated.fills.length - before} order(s) filled at this paper tick.`);
    await refreshBots();
  }));
  tickRow.append(tickButton); area.append(tickRow);

  const actions = el('div', 'bot-actions');
  for (const [label, action] of (b.status === 'running' ? [['Pause', 'pause']] : b.status === 'paused' ? [['Resume', 'resume']] : [])) {
    const button = el('button', 'small-btn', label); button.type = 'button';
    button.addEventListener('click', () => busy(async () => { await command('bot_action', { id: b.id, action }); await refreshBots(); }));
    actions.append(button);
  }
  if (b.status !== 'stopped') {
    const stop = el('button', 'small-btn danger', 'Stop & release reserves'); stop.type = 'button';
    stop.addEventListener('click', () => {
      if (confirm('Stop this paper bot and cancel its remaining simulated orders?'))
        busy(async () => { await command('bot_action', { id: b.id, action: 'stop' }); await refreshBots(); });
    }); actions.append(stop);
  } else {
    const remove = el('button', 'small-btn danger', 'Delete bot'); remove.type = 'button';
    remove.addEventListener('click', () => {
      if (confirm('Permanently delete this paper bot and its fill history?'))
        busy(async () => { await command('delete_bot', { id: b.id }); await refreshBots(); });
    }); actions.append(remove);
  }
  area.append(actions);

  area.append(el('div', 'mini-title', 'PAPER WALLET · FREE / RESERVED'));
  const wallet = el('div', 'wallet-row');
  for (const [label, n] of [['BASE FREE', w.base_free], ['BASE RESERVED', w.base_locked], ['QUOTE FREE', w.quote_free], ['QUOTE RESERVED', w.quote_locked]]) {
    const cell = el('div', '', label); cell.append(el('strong', '', fmt(n, 8))); wallet.append(cell);
  }
  area.append(wallet);

  area.append(el('div', 'mini-title', `OPEN ORDERS · ${b.orders.filter(o => o.status === 'open').length}`));
  const openHistory = el('div', 'history');
  const open = b.orders.filter(o => o.status === 'open');
  if (!open.length) openHistory.append(el('div', 'history-line', 'No open paper orders'));
  for (const order of open) {
    const line = el('div', 'history-line');
    line.append(el('span', order.side === 'buy' ? 'BUY' : 'SELL', `${order.side.toUpperCase()} · level ${order.level}`),
      el('span', '', `${fmt(order.amount, 8)} @ ${fmt(order.price)}`)); openHistory.append(line);
  }
  area.append(openHistory);

  area.append(el('div', 'mini-title', `RECENT FILLS · ${b.fills.length}`));
  const fills = el('div', 'history');
  if (!b.fills.length) fills.append(el('div', 'history-line', 'No fills yet. Feed a price that touches a grid level.'));
  for (const fill of b.fills.slice(-15).reverse()) {
    const line = el('div', 'history-line');
    line.append(el('span', fill.side, `${fill.side.toUpperCase()} · ${fmt(fill.amount, 8)} @ ${fmt(fill.price)}`),
      el('span', '', `fee ${fmt(fill.fee_amount, 8)} ${fill.fee_asset}`)); fills.append(line);
  }
  area.append(fills);
  for (const warning of b.warnings.slice(-3)) area.append(el('div', 'warning', `⚠ ${warning}`));
}

async function initialize() {
  $('#preview-btn').addEventListener('click', () => busy(async () => { await preview(); notice('Grid preview updated. Balances are shown beside the orders.'); }));
  $('#grid-form').addEventListener('submit', e => {
    e.preventDefault(); busy(async () => {
      await preview();
      const b = await command('create_paper_bot', {
        name: $('#name').value, config: config(), currentPrice: value('current-price'),
        initialBase: value('initial-base'), initialQuote: value('initial-quote')
      });
      state.selectedId = b.id;
      notice(`Paper bot “${b.name}” created with ${b.orders.length} reserved orders.`);
      await refreshBots();
    });
  });
  $('#grid-form').addEventListener('input', () => {
    state.preview = null; $('#preview-content').hidden = true; $('#preview-empty').hidden = false;
  });
  try { await refreshBots(); }
  catch (error) { notice(String(error), true); }
}
await initialize();
return {};
}
