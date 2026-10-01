<template>
  <div class="page-content">
    <!-- Intro -->
    <div class="intro">
      <div>
        <div class="eyebrow">TRADING LAB / 001</div>
        <h1>Build your grid<span>.</span></h1>
        <p style="color:var(--muted); margin:0">One focused workspace for planning, testing, and tracking a grid strategy.</p>
      </div>
      <div class="mode-card">
        <div class="mode-icon">◇</div>
        <div>
          <strong style="display:block;color:var(--lime);font:12px var(--mono);letter-spacing:1.2px">PAPER MODE</strong>
          <span style="display:block;color:#a9b7a6;font-size:11px;margin-top:4px">Manual price feed · no exchange orders</span>
        </div>
      </div>
    </div>

    <!-- Notice -->
    <div v-if="notice" class="notice" :class="{ error: noticeError }" role="status">{{ notice }}</div>

    <!-- Config + Preview -->
    <div class="main-grid">
      <!-- 01 Configure -->
      <section class="panel setup">
        <div class="panel-heading">
          <div><span class="section-n">01</span><h2>Configure bot</h2></div>
          <span class="heading-sub">STRATEGY SETUP</span>
        </div>
        <form @submit.prevent="createBot">
          <div class="field-row">
            <label>Bot name<input v-model="form.name" maxlength="60" autocomplete="off"></label>
            <label>Symbol<input v-model="form.symbol" maxlength="32" autocomplete="off"></label>
          </div>

          <div class="subheading">PRICE RANGE <span>01 / 03</span></div>
          <div class="field-row three">
            <label>Current price<input v-model.number="form.currentPrice" type="number" min="0" step="any"></label>
            <label>Lower bound<input v-model.number="form.lowerPrice" type="number" min="0" step="any"></label>
            <label>Upper bound<input v-model.number="form.upperPrice" type="number" min="0" step="any"></label>
          </div>
          <div class="field-row">
            <label>Grid intervals<input v-model.number="form.gridCount" type="number" min="2" max="200" step="1"></label>
            <label>Spacing
              <select v-model="form.gridType">
                <option value="linear">Linear</option>
                <option value="geometric">Geometric</option>
              </select>
            </label>
          </div>

          <div class="subheading">ORDER SIZE <span>02 / 03</span></div>
          <div class="field-row three">
            <label>Amount<input v-model.number="form.amount" type="number" min="0" step="any"></label>
            <label>Currency
              <select v-model="form.amountUnit">
                <option value="quote">Quote</option>
                <option value="base">Base</option>
              </select>
            </label>
            <label>Allocation
              <select v-model="form.amountType">
                <option value="per_grid">Per grid</option>
                <option value="total">Total / intervals</option>
                <option value="incremental">Incremental</option>
              </select>
            </label>
          </div>
          <div class="field-row three">
            <label>Initial orders
              <select v-model="form.side">
                <option value="both">Buy + sell</option>
                <option value="buy_only">Start buys only</option>
                <option value="sell_only">Start sells only</option>
              </select>
            </label>
            <label>Buy increment %<input v-model.number="form.incrementBuy" type="number" min="0" max="100" step="any"></label>
            <label>Sell increment %<input v-model.number="form.incrementSell" type="number" min="0" max="100" step="any"></label>
          </div>

          <div class="subheading">PAPER WALLET <span>03 / 03</span></div>
          <div class="field-row three">
            <label>Starting base<input v-model.number="form.initialBase" type="number" min="0" step="any"></label>
            <label>Starting quote<input v-model.number="form.initialQuote" type="number" min="0" step="any"></label>
            <label>Fee per fill %<input v-model.number="form.fee" type="number" min="0" max="5" step="any"></label>
          </div>
          <p class="field-note">Buy orders reserve quote. Sell orders reserve base. Preview checks both before the bot starts.</p>

          <div class="form-actions">
            <button type="button" class="button secondary" :disabled="busy" @click="previewGrid">
              Preview grid <span>↗</span>
            </button>
            <button type="submit" class="button primary" :disabled="busy">
              Create paper bot <span>→</span>
            </button>
          </div>
        </form>
      </section>

      <!-- 02 Preview -->
      <section class="panel preview">
        <div class="panel-heading">
          <div><span class="section-n">02</span><h2>Grid preview</h2></div>
          <span class="heading-sub">BEFORE YOU START</span>
        </div>

        <div v-if="!previewData" class="empty-preview">
          <div class="empty-art">
            <div class="bar b1"></div><div class="bar b2"></div><div class="bar b3"></div>
            <div class="bar b4"></div><div class="bar b5"></div>
          </div>
          <strong>Your levels appear here</strong>
          <p>Enter a range and preview the exact paper orders and funds needed.</p>
        </div>

        <div v-else style="padding:20px 22px">
          <div class="price-map">
            <div class="map-line"></div>
            <div class="map-labels">
              <span>{{ fmt(previewData.levels[0]) }}</span>
              <span>{{ fmt(form.currentPrice) }}</span>
              <span>{{ fmt(previewData.levels[previewData.levels.length - 1]) }}</span>
            </div>
            <div class="map-captions"><span>LOWER</span><span>CURRENT</span><span>UPPER</span></div>
          </div>
          <div class="metrics">
            <div><span>BUY ORDERS</span><strong class="buy">{{ previewData.buy_orders }}</strong></div>
            <div><span>SELL ORDERS</span><strong class="sell">{{ previewData.sell_orders }}</strong></div>
            <div><span>QUOTE NEEDED</span><strong>{{ fmt(previewData.buy_reserve_quote, 2) }}</strong></div>
            <div><span>BASE NEEDED</span><strong>{{ fmt(previewData.sell_reserve_base) }}</strong></div>
          </div>
          <div class="table-label">
            PLANNED LIMIT ORDERS <span>{{ previewData.orders.length }} LEVELS ACTIVE</span>
          </div>
          <div class="order-scroll">
            <table>
              <thead><tr><th>LEVEL</th><th>SIDE</th><th>PRICE</th><th>AMOUNT</th><th>NOTIONAL</th></tr></thead>
              <tbody>
                <tr v-for="o in previewData.orders" :key="o.level + o.side">
                  <td>{{ String(o.level).padStart(2, '0') }}</td>
                  <td><span :class="['side-pill', o.side]">{{ o.side.toUpperCase() }}</span></td>
                  <td>{{ fmt(o.price) }}</td>
                  <td>{{ fmt(o.amount, 8) }}</td>
                  <td>{{ fmt(o.total, 2) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </div>

    <!-- 03 Paper bots -->
    <div class="section-title">
      <div><span class="section-n">03</span><h2>Paper bots</h2></div>
      <span>{{ bots.length }} BOT{{ bots.length === 1 ? '' : 'S' }}</span>
    </div>

    <div class="bottom-grid">
      <!-- Bot list -->
      <section class="panel bot-list">
        <div v-if="!bots.length" class="list-placeholder">No bots yet. Create one above to begin.</div>
        <button
          v-for="b in bots" :key="b.id"
          type="button"
          class="bot-card"
          :class="{ selected: b.id === selectedId }"
          @click="selectedId = b.id"
        >
          <div class="bot-card-top">
            <strong>{{ b.name }}</strong>
            <span :class="['status', b.status]">{{ b.status }}</span>
          </div>
          <div class="bot-card-bottom">
            <span>{{ b.config.symbol }}</span>
            <span>{{ openCount(b) }} open · {{ b.fills.length }} fills</span>
          </div>
        </button>
      </section>

      <!-- Bot detail -->
      <section class="panel bot-detail" style="padding:15px; max-height:510px; overflow:auto">
        <template v-if="selected">
          <div class="detail-head">
            <div>
              <h3>{{ selected.name }}</h3>
              <small>{{ selected.config.symbol }} · #{{ selected.id }} · {{ selected.config.grid_type }}</small>
            </div>
            <span :class="['status', selected.status]">{{ selected.status }}</span>
          </div>

          <div class="detail-metrics">
            <div><label>LAST PRICE</label><strong>{{ fmt(selected.current_price) }}</strong></div>
            <div><label>EQUITY / QUOTE</label><strong>{{ fmt(equity(selected), 2) }}</strong></div>
            <div>
              <label>VS HOLD / QUOTE</label>
              <strong :class="equityChange(selected) >= 0 ? 'buy' : 'sell'">
                {{ equityChange(selected) >= 0 ? '+' : '' }}{{ fmt(equityChange(selected), 2) }}
              </strong>
            </div>
            <div><label>TOTAL FILLS</label><strong>{{ selected.fills.length }}</strong></div>
          </div>

          <!-- Tick -->
          <div class="tick-row">
            <label style="flex:1">
              Feed a manual market price
              <input v-model.number="tickPrice" type="number" step="any" min="0">
            </label>
            <button
              class="small-btn"
              :disabled="selected.status !== 'running' || busy"
              @click="tickBot"
            >Process price →</button>
          </div>

          <!-- Actions -->
          <div class="bot-actions">
            <button v-if="selected.status === 'running'" class="small-btn" :disabled="busy" @click="botAction('pause')">Pause</button>
            <button v-if="selected.status === 'paused'" class="small-btn" :disabled="busy" @click="botAction('resume')">Resume</button>
            <button v-if="selected.status !== 'stopped'" class="small-btn danger" :disabled="busy" @click="confirmStop">Stop & release reserves</button>
            <button v-if="selected.status === 'stopped'" class="small-btn danger" :disabled="busy" @click="confirmDelete">Delete bot</button>
          </div>

          <!-- Wallet -->
          <div class="mini-title">PAPER WALLET · FREE / RESERVED</div>
          <div class="wallet-row">
            <div>BASE FREE<strong>{{ fmt(selected.wallet.base_free, 8) }}</strong></div>
            <div>BASE RESERVED<strong>{{ fmt(selected.wallet.base_locked, 8) }}</strong></div>
            <div>QUOTE FREE<strong>{{ fmt(selected.wallet.quote_free, 8) }}</strong></div>
            <div>QUOTE RESERVED<strong>{{ fmt(selected.wallet.quote_locked, 8) }}</strong></div>
          </div>

          <!-- Open orders -->
          <div class="mini-title">OPEN ORDERS · {{ openCount(selected) }}</div>
          <div class="history">
            <div v-if="!openCount(selected)" class="history-line">No open paper orders</div>
            <div v-for="o in selected.orders.filter(o => o.status === 'open')" :key="o.id" class="history-line">
              <span :class="o.side">{{ o.side.toUpperCase() }} · level {{ o.level }}</span>
              <span>{{ fmt(o.amount, 8) }} @ {{ fmt(o.price) }}</span>
            </div>
          </div>

          <!-- Recent fills -->
          <div class="mini-title">RECENT FILLS · {{ selected.fills.length }}</div>
          <div class="history">
            <div v-if="!selected.fills.length" class="history-line">No fills yet. Feed a price that touches a grid level.</div>
            <div v-for="f in [...selected.fills].reverse().slice(0, 15)" :key="f.order_id" class="history-line">
              <span :class="f.side">{{ f.side.toUpperCase() }} · {{ fmt(f.amount, 8) }} @ {{ fmt(f.price) }}</span>
              <span>fee {{ fmt(f.fee_amount, 8) }} {{ f.fee_asset }}</span>
            </div>
          </div>

          <div v-for="w in selected.warnings.slice(-3)" :key="w" class="warning">⚠ {{ w }}</div>
        </template>
        <div v-else class="list-placeholder">Select a bot to see its wallet, orders, and fills.</div>
      </section>
    </div>

    <footer>
      OMIBUS GRID · LOCAL PAPER TERMINAL
      <span>Simulation uses manual ticks and assumes limit fills at the level price.</span>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useInvoke } from '../composables/useTauri'

const invoke = useInvoke()

// ── Types ──────────────────────────────────────────────────────────────
interface GridConfig {
  symbol: string; lower_price: number; upper_price: number; nr_of_grids: number
  amount: number; amount_type: string; amount_unit: string
  incremental_pct_buy: number; incremental_pct_sell: number
  side: string; grid_type: string; fee_pct: number
}
interface GridOrder { level: number; side: string; price: number; amount: number; total: number }
interface GridPreview {
  levels: number[]; orders: GridOrder[]
  buy_orders: number; sell_orders: number
  buy_reserve_quote: number; sell_reserve_base: number
  avg_buy_price: number; avg_sell_price: number
}
interface Wallet { base_free: number; base_locked: number; quote_free: number; quote_locked: number }
interface PaperOrder { id: number; level: number; side: string; price: number; amount: number; total: number; status: string }
interface Fill { order_id: number; at_ms: number; side: string; price: number; amount: number; fee_amount: number; fee_asset: string }
interface PaperBot {
  id: number; name: string; config: GridConfig; status: string
  current_price: number; initial_base: number; initial_quote: number
  wallet: Wallet; orders: PaperOrder[]; fills: Fill[]; warnings: string[]
}

// ── State ──────────────────────────────────────────────────────────────
const bots = ref<PaperBot[]>([])
const selectedId = ref<number | null>(null)
const previewData = ref<GridPreview | null>(null)
const notice = ref('')
const noticeError = ref(false)
const busy = ref(false)
const tickPrice = ref(100)

const form = ref({
  name: 'Range study 01', symbol: 'DEMO/USDT',
  currentPrice: 100, lowerPrice: 90, upperPrice: 110,
  gridCount: 8, gridType: 'linear',
  amount: 100, amountUnit: 'quote', amountType: 'per_grid',
  side: 'both', incrementBuy: 0, incrementSell: 0,
  initialBase: 5, initialQuote: 1000, fee: 0.1,
})

// ── Derived ────────────────────────────────────────────────────────────
const selected = computed(() => bots.value.find(b => b.id === selectedId.value) ?? null)

watch(form, () => { previewData.value = null }, { deep: true })
watch(selected, b => { if (b) tickPrice.value = b.current_price })

// ── Helpers ────────────────────────────────────────────────────────────
function fmt(v: unknown, digits = 6): string {
  if (v === null || v === undefined || !Number.isFinite(Number(v))) return '—'
  const n = Number(v)
  if (n !== 0 && Math.abs(n) < 0.000001) return n.toPrecision(4)
  return n.toLocaleString('en-US', { maximumFractionDigits: digits })
}

function openCount(b: PaperBot) { return b.orders.filter(o => o.status === 'open').length }

function equity(b: PaperBot) {
  return b.wallet.quote_free + b.wallet.quote_locked + (b.wallet.base_free + b.wallet.base_locked) * b.current_price
}
function equityChange(b: PaperBot) {
  return equity(b) - (b.initial_quote + b.initial_base * b.current_price)
}

function buildConfig(): GridConfig {
  return {
    symbol: form.value.symbol.trim(),
    lower_price: form.value.lowerPrice, upper_price: form.value.upperPrice,
    nr_of_grids: form.value.gridCount, amount: form.value.amount,
    amount_type: form.value.amountType, amount_unit: form.value.amountUnit,
    incremental_pct_buy: form.value.incrementBuy, incremental_pct_sell: form.value.incrementSell,
    side: form.value.side, grid_type: form.value.gridType, fee_pct: form.value.fee,
  }
}

function showNotice(msg: string, error = false) {
  notice.value = msg; noticeError.value = error
}

async function withBusy(fn: () => Promise<void>) {
  if (busy.value) return
  busy.value = true
  try { await fn() } catch (e) { showNotice(String(e), true) } finally { busy.value = false }
}

// ── Commands ───────────────────────────────────────────────────────────
async function refreshBots() {
  bots.value = (await invoke<PaperBot[]>('list_bots')).sort((a, b) => b.id - a.id)
  if (!bots.value.some(b => b.id === selectedId.value))
    selectedId.value = bots.value[0]?.id ?? null
}

async function previewGrid() {
  await withBusy(async () => {
    previewData.value = await invoke<GridPreview>('preview_grid', {
      config: buildConfig(), currentPrice: form.value.currentPrice,
    })
    showNotice('Grid preview updated. Balances are shown beside the orders.')
  })
}

async function createBot() {
  await withBusy(async () => {
    await previewGrid()
    const bot = await invoke<PaperBot>('create_paper_bot', {
      name: form.value.name, config: buildConfig(),
      currentPrice: form.value.currentPrice,
      initialBase: form.value.initialBase, initialQuote: form.value.initialQuote,
    })
    selectedId.value = bot.id
    showNotice(`Paper bot "${bot.name}" created with ${bot.orders.length} reserved orders.`)
    await refreshBots()
  })
}

async function tickBot() {
  if (!selected.value) return
  await withBusy(async () => {
    const before = selected.value!.fills.length
    const updated = await invoke<PaperBot>('tick_bot', { id: selected.value!.id, price: tickPrice.value })
    showNotice(`${updated.fills.length - before} order(s) filled at this paper tick.`)
    await refreshBots()
  })
}

async function botAction(action: string) {
  if (!selected.value) return
  await withBusy(async () => {
    await invoke('bot_action', { id: selected.value!.id, action })
    await refreshBots()
  })
}

function confirmStop() {
  if (confirm('Stop this paper bot and cancel its remaining simulated orders?'))
    botAction('stop')
}

async function confirmDelete() {
  if (!selected.value) return
  if (confirm('Permanently delete this paper bot and its fill history?')) {
    await withBusy(async () => {
      await invoke('delete_bot', { id: selected.value!.id })
      await refreshBots()
    })
  }
}

onMounted(async () => {
  try { await refreshBots() } catch (e) { showNotice(String(e), true) }
})
</script>

<style scoped>
.main-grid {
  display: grid;
  grid-template-columns: minmax(485px, .95fr) minmax(430px, 1.05fr);
  gap: 14px;
}
.panel { background: var(--panel); border: 1px solid var(--line); border-radius: 13px; min-width: 0; }
.panel-heading {
  display: flex; justify-content: space-between; align-items: center;
  padding: 19px 22px; border-bottom: 1px solid var(--line);
}
.panel-heading > div { display: flex; align-items: center; gap: 13px; }
.panel-heading h2 { margin: 0; font-weight: 650; letter-spacing: -.3px; font-size: 17px; }
.setup form { padding: 21px 22px 24px; }
.field-row { display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: 12px; margin-bottom: 13px; }
.field-row.three { grid-template-columns: repeat(3, minmax(0,1fr)); }
.subheading {
  display: flex; justify-content: space-between;
  border-top: 1px solid var(--line); padding-top: 18px; margin: 19px 0 14px;
  color: #d7e8c8; font-size: 10px;
}
.subheading span { color: #64737c; }
.field-note { color: var(--muted); font-size: 11px; line-height: 1.5; margin: 13px 0 20px; }
.form-actions { display: flex; gap: 10px; }
.button {
  border-radius: 8px; padding: 11px 14px; font-weight: 680; font-size: 12px;
  border: 1px solid #53603a; display: flex; justify-content: space-between; align-items: center; gap: 15px;
}
.button span { font-size: 16px; }
.button.primary { background: var(--lime); border-color: var(--lime); color: #18210e; flex: 1; }
.button.primary:hover { background: #d3ff81; }
.button.secondary { background: #1c2820; color: #cfe5b7; flex: 1; }
.button.secondary:hover { border-color: var(--lime); }

.preview { min-height: 570px; }
.empty-preview {
  height: 510px; display: flex; flex-direction: column;
  align-items: center; justify-content: center; text-align: center; padding: 25px;
}
.empty-preview strong { font-size: 16px; font-weight: 600; }
.empty-preview p { max-width: 260px; color: var(--muted); line-height: 1.6; font-size: 12px; }
.empty-art { height: 92px; display: flex; align-items: center; gap: 12px; margin-bottom: 25px; }
.bar { width: 6px; background: #3b5741; border-radius: 4px; }
.b1, .b5 { height: 25px; }
.b2, .b4 { height: 52px; }
.b3 { height: 80px; background: var(--lime); box-shadow: 0 0 24px #b9f05a66; }

.price-map {
  background: linear-gradient(90deg, #182b24, #222a23 50%, #2c2421);
  border: 1px solid var(--line); border-radius: 9px; padding: 22px 18px 13px; margin-bottom: 14px;
}
.map-line {
  height: 3px;
  background: linear-gradient(90deg, #8eddad 0%, #8eddad 50%, #e8ae90 50%, #e8ae90 100%);
  position: relative; margin: 4px 15px 18px;
}
.map-line::before, .map-line::after {
  content: ""; position: absolute; top: -4px; width: 10px; height: 10px;
  background: #b9f05a; border-radius: 50%;
}
.map-line::before { left: -2px; }
.map-line::after { right: -2px; background: #e8ae90; }
.map-labels, .map-captions { display: flex; justify-content: space-between; align-items: center; font-family: var(--mono); }
.map-labels { font-size: 14px; font-weight: 650; }
.map-labels span:nth-child(2) { color: var(--lime); }
.map-captions { font-size: 9px; color: #84938c; margin-top: 5px; letter-spacing: 1px; }
.metrics { display: grid; grid-template-columns: repeat(4,1fr); gap: 7px; margin-bottom: 25px; }
.metrics > div { background: #171e23; border: 1px solid #2a353e; border-radius: 8px; padding: 11px 9px; }
.metrics span { display: block; color: #84949b; font: 9px var(--mono); letter-spacing: .6px; min-height: 22px; }
.metrics strong { font: 600 16px var(--mono); white-space: nowrap; }
.table-label { display: flex; justify-content: space-between; color: #aab9ad; margin-bottom: 10px; font-size: 10px; }
.order-scroll { max-height: 352px; overflow: auto; border: 1px solid #2c373f; border-radius: 8px; }
table { border-collapse: collapse; width: 100%; font: 12px var(--mono); }
th { position: sticky; top: 0; background: #182128; color: #7e9198; text-align: right; font-size: 9px; letter-spacing: .7px; padding: 11px 8px; }
th:first-child { text-align: left; }
td { text-align: right; border-top: 1px solid #263038; padding: 10px 8px; color: #d6e0e3; }
td:first-child { text-align: left; color: #819099; }
tr:hover td { background: #1a2529; }
.side-pill { display: inline-block; font: 9px var(--mono); font-weight: 700; letter-spacing: .7px; border-radius: 4px; padding: 3px 6px; }
.side-pill.buy { background: #1e342a; }
.side-pill.sell { background: #352922; }

.section-title { display: flex; justify-content: space-between; align-items: center; margin: 28px 0 12px; }
.section-title > span { color: #7b8d92; font-size: 10px; font: 10px var(--mono); letter-spacing: 1.5px; }
.bottom-grid { display: grid; grid-template-columns: 350px minmax(0,1fr); gap: 14px; min-height: 260px; }
.bot-list { padding: 15px; max-height: 510px; overflow: auto; }
.list-placeholder { color: var(--muted); font-size: 12px; display: grid; place-items: center; height: 100%; min-height: 220px; text-align: center; }
.bot-card {
  width: 100%; text-align: left; color: inherit; padding: 14px;
  margin-bottom: 8px; border: 1px solid #334148; border-radius: 9px;
  cursor: pointer; background: #151d22;
}
.bot-card.selected { border-color: #7d9e4d; background: #1c2821; }
.bot-card:hover { border-color: #8a9c70; }
.bot-card-top { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
.bot-card strong { font-size: 13px; }
.bot-card-bottom { display: flex; justify-content: space-between; margin-top: 13px; color: #8e9fa5; font: 10px var(--mono); }

.detail-head { display: flex; align-items: start; justify-content: space-between; gap: 15px; margin-bottom: 16px; }
.detail-head h3 { margin: 0 0 5px; font-size: 18px; }
.detail-head small { color: var(--muted); font: 11px var(--mono); }
.detail-metrics { display: grid; grid-template-columns: repeat(4,1fr); gap: 8px; margin-bottom: 15px; }
.detail-metrics > div { border: 1px solid #2d383f; background: #172027; border-radius: 7px; padding: 10px; }
.detail-metrics label { display: block; font: 9px var(--mono); color: #819299; letter-spacing: .5px; }
.detail-metrics strong { display: block; font: 14px var(--mono); margin-top: 7px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tick-row, .bot-actions { display: flex; gap: 8px; align-items: end; margin-bottom: 14px; }
.tick-row label { flex: 1; }
.bot-actions { flex-wrap: wrap; }
.mini-title { font: 10px var(--mono); color: #a2b1ad; letter-spacing: 1px; margin: 16px 0 8px; }
.wallet-row { display: grid; grid-template-columns: repeat(4,1fr); gap: 7px; }
.wallet-row > div { background: #151d22; padding: 8px; border-radius: 5px; color: #8da0a5; font: 9px var(--mono); }
.wallet-row strong { display: block; color: #dbe5e3; margin-top: 5px; font: 12px var(--mono); }
.history { max-height: 160px; overflow: auto; border: 1px solid #2b363f; border-radius: 6px; padding: 1px 10px; }
.history-line { display: flex; justify-content: space-between; gap: 10px; border-bottom: 1px solid #263038; padding: 8px 0; font: 11px var(--mono); }
.history-line:last-child { border: 0; }
.history-line span:last-child { color: var(--muted); }
.warning { color: #efb091; font-size: 11px; margin-top: 9px; line-height: 1.4; }
footer { display: flex; justify-content: space-between; gap: 20px; border-top: 1px solid var(--line); margin-top: 24px; padding: 18px 0; color: #718088; font: 9px var(--mono); letter-spacing: .8px; }
footer span { text-align: right; letter-spacing: 0; }

@media (max-width: 1180px) {
  .main-grid { grid-template-columns: 1fr; }
  .preview { min-height: 0; }
}
@media (max-width: 800px) {
  .bottom-grid { grid-template-columns: 1fr; }
  .field-row.three { grid-template-columns: repeat(2, minmax(0,1fr)); }
  .detail-metrics, .wallet-row { grid-template-columns: repeat(2,1fr); }
}
</style>
