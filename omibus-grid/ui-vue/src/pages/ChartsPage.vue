<template>
  <div class="charts-shell">

    <!-- ── Chart grid ── -->
    <div class="charts-grid" :class="`layout-${charts.layout.value}`">
      <div v-for="(slot, idx) in visibleSlots" :key="slot.id" class="chart-slot">

        <!-- Slot header -->
        <div class="slot-header">
          <input v-model="slot.symbol" class="slot-input"
            placeholder="KRAKEN:XBTUSD"
            @keydown.enter="loadSlot(slot)"
            @blur="loadSlot(slot)"
            spellcheck="false" autocomplete="off" />
          <button class="slot-btn" @click="loadSlot(slot)" title="Reload">↗</button>
          <button v-if="slots.length > 1" class="slot-btn danger" @click="removeSlot(idx)" title="Close">✕</button>
        </div>

        <!-- Chart iframe -->
        <div class="slot-chart" :ref="el => { slotHosts[slot.id] = el as HTMLElement }"></div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onActivated, onUnmounted, reactive, nextTick } from 'vue'
import { useMarket } from '../composables/useMarket'
import { useCharts } from '../composables/useCharts'

const market = useMarket()
const charts = useCharts()

// ── Constants ─────────────────────────────────────────────────────────────────
const VALID_SYMBOL   = /^[A-Z0-9_]{1,20}:[A-Z0-9_.!/-]{1,35}$/
const VALID_INTERVAL = new Set(['1','3','5','15','30','60','120','240','D','W','M'])

// ── State ─────────────────────────────────────────────────────────────────────
let _uid = 0
interface Slot { id: number; symbol: string; interval: string; loaded: boolean }

const globalInterval = ref('D')
const slots          = ref<Slot[]>([{ id: ++_uid, symbol: 'KRAKEN:XBTUSD', interval: 'D', loaded: false }])
const slotHosts      = reactive<Record<number, HTMLElement | null>>({})

// ── Visible slots per layout ──────────────────────────────────────────────────
const layoutCount = computed(() => ({ '1': 1, '2h': 2, '2v': 2, '4': 4 }[charts.layout.value] ?? 1))

const visibleSlots = computed(() => {
  while (slots.value.length < layoutCount.value)
    slots.value.push({ id: ++_uid, symbol: 'KRAKEN:XBTUSD', interval: globalInterval.value, loaded: false })
  return slots.value.slice(0, layoutCount.value)
})

// ── Market selector → slot 1 ──────────────────────────────────────────────────
function toTvSymbol(exchange: string, symbol: string): string {
  // normalise: XBT/USD → XBTUSD, BTC-USD → BTCUSD, BTCUSDT → BTCUSDT
  const sym = symbol.replace(/[/-]/g, '').toUpperCase()
  const ex  = exchange.toUpperCase()
  // TradingView exchange map
  const exMap: Record<string, string> = {
    KRAKEN: 'KRAKEN', COINBASE: 'COINBASE', BINANCE: 'BINANCE',
    BYBIT: 'BYBIT', OKX: 'OKX', KUCOIN: 'KUCOIN',
    GATE: 'GATEIO', MEXC: 'MEXC', LCX: 'LCX',
    HYPERLIQUID: 'BINANCE',   // no native TV feed, fallback to Binance
  }
  return `${exMap[ex] ?? ex}:${sym}`
}

function syncFromMarket() {
  if (!market.exchange.value || !market.symbol.value) return
  const tv = toTvSymbol(market.exchange.value, market.symbol.value)
  if (slots.value.length > 0) {
    slots.value[0].symbol = tv
    slots.value[0].interval = globalInterval.value
    loadSlot(slots.value[0])
  }
}

function syncFromMarket2() {
  if (!market.exchange2.value || !market.symbol2.value) return
  const tv = toTvSymbol(market.exchange2.value, market.symbol2.value)
  if (slots.value.length > 1) {
    slots.value[1].symbol = tv
    slots.value[1].interval = globalInterval.value
    loadSlot(slots.value[1])
  }
}

// Auto-sync slots when market selector changes (active once page is mounted)
let autoSync = false
watch([() => market.exchange.value, () => market.symbol.value], () => {
  if (autoSync) syncFromMarket()
})
watch([() => market.exchange2.value, () => market.symbol2.value], () => {
  if (autoSync) syncFromMarket2()
})

// ── Global interval propagation ───────────────────────────────────────────────
watch(globalInterval, iv => {
  visibleSlots.value.forEach(s => { s.interval = iv; loadSlot(s) })
})

// ── Load / remove slots ───────────────────────────────────────────────────────
function loadSlot(slot: Slot) {
  const sym = slot.symbol.trim().toUpperCase()
  const iv  = slot.interval

  if (!VALID_SYMBOL.test(sym)) return
  slot.symbol = sym

  const settings = {
    autosize: true, symbol: sym, interval: iv,
    timezone: 'Etc/UTC', theme: 'dark', style: '1', locale: 'en',
    allow_symbol_change: true, hide_side_toolbar: false,
    hide_top_toolbar: false, hide_legend: false, hide_volume: false,
    withdateranges: true, save_image: true, details: true,
    calendar: false, hotlist: false,
    support_host: 'https://www.tradingview.com',
  }

  const url = new URL('https://www.tradingview-widget.com/embed-widget/advanced-chart/')
  url.searchParams.set('locale', 'en')
  url.hash = encodeURIComponent(JSON.stringify(settings))

  const frame = document.createElement('iframe')
  frame.title = `TradingView chart — ${sym}`
  frame.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads')
  frame.setAttribute('allow', 'fullscreen')
  frame.referrerPolicy = 'strict-origin-when-cross-origin'
  frame.style.cssText = 'width:100%;height:100%;border:none;'

  const host = slotHosts[slot.id]
  if (host) { host.replaceChildren(frame); frame.src = url.href }
  slot.loaded = true
}

function addSlot() {
  slots.value.push({ id: ++_uid, symbol: 'BINANCE:BTCUSDT', interval: globalInterval.value, loaded: false })
}

function removeSlot(idx: number) {
  slots.value.splice(idx, 1)
}

// ── IndexedDB persistence ─────────────────────────────────────────────────────
const IDB_NAME = 'omibus-charts-db'
const IDB_STORE = 'state'
const IDB_KEY = 'charts-v1'

function openIdb(): Promise<IDBDatabase> {
  return new Promise((res, rej) => {
    const req = indexedDB.open(IDB_NAME, 1)
    req.onupgradeneeded = () => req.result.createObjectStore(IDB_STORE)
    req.onsuccess = () => res(req.result)
    req.onerror = () => rej(req.error)
  })
}

async function saveToIdb() {
  try {
    const db = await openIdb()
    const payload = {
      layout: charts.layout.value,
      interval: globalInterval.value,
      slots: slots.value.map(s => ({ symbol: s.symbol, interval: s.interval })),
    }
    const tx = db.transaction(IDB_STORE, 'readwrite')
    tx.objectStore(IDB_STORE).put(payload, IDB_KEY)
    db.close()
    // keep localStorage in sync as fast fallback
    localStorage.setItem('omibus-charts', JSON.stringify(payload))
  } catch {}
}

async function restoreFromIdb(): Promise<boolean> {
  try {
    const db = await openIdb()
    const d: any = await new Promise((res, rej) => {
      const req = db.transaction(IDB_STORE, 'readonly').objectStore(IDB_STORE).get(IDB_KEY)
      req.onsuccess = () => res(req.result)
      req.onerror = () => rej(req.error)
    })
    db.close()
    if (!d) return false
    if (d.layout && ['1','2h','2v','4'].includes(d.layout)) charts.setLayout(d.layout as import('../composables/useCharts').Layout)
    if (d.interval) globalInterval.value = d.interval
    if (Array.isArray(d.slots) && d.slots.length) {
      slots.value = d.slots.map((s: any) => ({
        id: ++_uid,
        symbol: VALID_SYMBOL.test(s.symbol) ? s.symbol : 'KRAKEN:XBTUSD',
        interval: VALID_INTERVAL.has(s.interval) ? s.interval : 'D',
        loaded: false,
      }))
    }
    return true
  } catch { return false }
}

function restoreFromLocalStorage() {
  try {
    const d = JSON.parse(localStorage.getItem('omibus-charts') ?? 'null')
    if (!d) return
    if (d.layout && ['1','2h','2v','4'].includes(d.layout)) charts.setLayout(d.layout as import('../composables/useCharts').Layout)
    if (d.interval) globalInterval.value = d.interval
    if (Array.isArray(d.slots) && d.slots.length) {
      slots.value = d.slots.map((s: any) => ({
        id: ++_uid,
        symbol: VALID_SYMBOL.test(s.symbol) ? s.symbol : 'KRAKEN:XBTUSD',
        interval: VALID_INTERVAL.has(s.interval) ? s.interval : 'D',
        loaded: false,
      }))
    }
  } catch {}
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────
let saveTimer: ReturnType<typeof setInterval>

onMounted(async () => {
  // Try IDB first (fastest on repeated opens), fallback to localStorage
  const restoredFromIdb = await restoreFromIdb()
  if (!restoredFromIdb) restoreFromLocalStorage()

  // Initial sync from market selector
  if (market.exchange.value && market.symbol.value) {
    slots.value[0].symbol = toTvSymbol(market.exchange.value, market.symbol.value)
    slots.value[0].interval = globalInterval.value
    autoSync = true
  }
  // Wait for DOM, then load all visible slots
  await nextTick()
  visibleSlots.value.forEach(s => loadSlot(s))
  saveTimer = setInterval(saveToIdb, 8000)
})

onActivated(async () => {
  await nextTick()
  visibleSlots.value.forEach(s => { if (!s.loaded) loadSlot(s) })
})

onUnmounted(() => {
  clearInterval(saveTimer)
  saveToIdb()
})
</script>

<style scoped>
.charts-shell {
  display: flex; flex-direction: column;
  height: 100%; overflow: hidden;
  background: var(--bg);
}

/* ── Controls bar ── */
.charts-bar {
  display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
  padding: 5px 10px; flex-shrink: 0;
  background: rgba(14,22,40,0.9);
  border-bottom: 1px solid var(--border);
}

.eyebrow { font-size: 9px; font-weight: 700; letter-spacing: .1em; color: var(--text-muted); white-space: nowrap; }

.bar-sep { width: 1px; height: 18px; background: rgba(255,255,255,0.08); }

.layout-picker { display: flex; gap: 2px; }
.layout-btn {
  padding: 3px 7px; border-radius: 4px; font-size: 11px;
  background: transparent; border: 1px solid rgba(59,130,246,0.2);
  color: rgba(255,255,255,0.3); cursor: pointer; transition: all .12s;
}
.layout-btn:hover { border-color: rgba(59,130,246,0.5); color: rgba(255,255,255,0.7); }
.layout-btn.active { background: rgba(59,130,246,0.2); border-color: #3b82f6; color: #3b82f6; }

/* Timeframe buttons (global bar) */
.tf-picker { display: flex; gap: 2px; flex-wrap: wrap; }
.tf-btn {
  padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 600;
  background: transparent; border: 1px solid rgba(255,255,255,0.08);
  color: rgba(255,255,255,0.35); cursor: pointer; transition: all .1s;
  min-width: 26px;
}
.tf-btn:hover { border-color: rgba(59,130,246,0.4); color: rgba(255,255,255,0.7); }
.tf-btn.active { background: rgba(59,130,246,0.18); border-color: #3b82f6; color: #3b82f6; }

.market-badge {
  display: flex; align-items: center; gap: 5px;
  padding: 2px 8px; border-radius: 5px;
  background: rgba(59,130,246,0.08); border: 1px solid rgba(59,130,246,0.2);
  font-size: 10px;
}
.mb-ex  { color: #3b82f6; font-weight: 700; }
.mb-sym { color: #e0e0e0; }
.mb-sync {
  padding: 1px 6px; border-radius: 3px; font-size: 9px; font-weight: 700;
  background: rgba(59,130,246,0.15); border: 1px solid rgba(59,130,246,0.3);
  color: #3b82f6; cursor: pointer;
}
.mb-sync:hover { background: rgba(59,130,246,0.3); }

.ex2-badge { background: rgba(167,139,250,0.08); border-color: rgba(167,139,250,0.2); }
.ex2-ex    { color: #a78bfa; }
.ex2-sync  {
  background: rgba(167,139,250,0.15); border-color: rgba(167,139,250,0.3);
  color: #a78bfa;
}
.ex2-sync:hover { background: rgba(167,139,250,0.3); }

.bar-btn {
  padding: 3px 10px; border-radius: 4px; font-size: 10px; font-weight: 700;
  background: rgba(16,235,4,0.12); border: 1px solid rgba(16,235,4,0.3);
  color: #10eb04; cursor: pointer;
}
.bar-btn:hover { background: rgba(16,235,4,0.22); }

/* ── Grid layouts ── */
.charts-grid {
  flex: 1; overflow: hidden;
  display: grid; gap: 4px; padding: 4px;
}
.layout-1  { grid-template-columns: 1fr; grid-template-rows: 1fr; }
.layout-2h { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr; }
.layout-2v { grid-template-columns: 1fr; grid-template-rows: 1fr 1fr; }
.layout-4  { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }

/* ── Chart slot ── */
.chart-slot {
  display: flex; flex-direction: column; overflow: hidden;
  border: 1px solid var(--border); border-radius: 7px;
  background: #0d1318;
}

.slot-header {
  display: flex; align-items: center; gap: 4px; flex-wrap: wrap;
  padding: 3px 6px; flex-shrink: 0;
  background: rgba(14,22,40,0.8);
  border-bottom: 1px solid rgba(59,130,246,0.1);
}

.slot-input {
  flex: 0 0 auto; width: 130px; padding: 2px 5px; font-size: 10px; font-family: monospace;
  background: rgba(15,20,35,0.8); border: 1px solid rgba(59,130,246,0.2);
  border-radius: 4px; color: #e0e0e0; outline: none;
}
.slot-input:focus { border-color: rgba(59,130,246,0.6); }

/* Per-slot timeframe buttons */
.slot-tf { display: flex; gap: 2px; flex-wrap: wrap; }
.stf-btn {
  padding: 1px 5px; border-radius: 3px; font-size: 9px; font-weight: 600;
  background: transparent; border: 1px solid rgba(255,255,255,0.07);
  color: rgba(255,255,255,0.3); cursor: pointer; transition: all .1s;
  min-width: 22px;
}
.stf-btn:hover { border-color: rgba(59,130,246,0.35); color: rgba(255,255,255,0.65); }
.stf-btn.active { background: rgba(59,130,246,0.15); border-color: rgba(59,130,246,0.6); color: #60a5fa; }

.slot-btn {
  padding: 2px 7px; border-radius: 3px; font-size: 10px; font-weight: 700;
  background: rgba(59,130,246,0.1); border: 1px solid rgba(59,130,246,0.25);
  color: #60a5fa; cursor: pointer;
}
.slot-btn:hover { background: rgba(59,130,246,0.25); }
.slot-btn.danger { background: rgba(235,4,4,0.1); border-color: rgba(235,4,4,0.25); color: #f87171; }
.slot-btn.danger:hover { background: rgba(235,4,4,0.25); }

.slot-chart { flex: 1; overflow: hidden; }
</style>
