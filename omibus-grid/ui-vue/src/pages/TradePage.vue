<template>
  <div class="tr-shell">

    <!-- ── Left: Orderbook ── -->
    <div class="tr-ob">
      <div class="panel-head">ORDER BOOK
        <div class="depth-picker">
          <button v-for="d in DEPTHS" :key="d" :class="['depth-btn',{active:depth===d}]" @click="depth=d">{{ d }}</button>
        </div>
        <span class="ob-live" :class="{live:isLive}">{{ isLive ? '● LIVE' : '○ REST' }}</span>
      </div>

      <!-- metrics mini bar -->
      <div v-if="ob" class="ob-metrics">
        <span class="ob-m"><span class="ob-ml">MID</span>{{ fmt(midPrice,priceDp) }}</span>
        <span class="ob-m"><span class="ob-ml">SPR</span><span :class="spreadPct<0.05?'g':'y'">{{ spreadPct.toFixed(3) }}%</span></span>
        <span class="ob-m"><span class="ob-ml">IMB</span><span :style="{color:imbalance>0?'#4ade80':'#f87171'}">{{ (imbalance*100).toFixed(1) }}%</span></span>
      </div>

      <div class="ob-book" v-if="ob">
        <!-- Asks reversed -->
        <div class="ob-asks">
          <div v-for="lvl in askLevels" :key="'a'+lvl.price"
            class="ob-row ask-row" @click="fillPrice(lvl.price)">
            <span class="ob-price ask">{{ fmt(lvl.price,priceDp) }}</span>
            <span class="ob-size">{{ fmtVol(lvl.size) }}</span>
            <span class="ob-total muted">{{ fmtVol(lvl.total) }}</span>
            <div class="ob-bar ask-bar" :style="{width:barPct(lvl.size)+'%'}"></div>
          </div>
        </div>
        <div class="ob-spread-row">
          <span class="ob-spread-val">{{ fmt(spread,4) }}</span>
          <span class="ob-spread-pct">{{ spreadPct.toFixed(3) }}%</span>
        </div>
        <div class="ob-bids">
          <div v-for="lvl in bidLevels" :key="'b'+lvl.price"
            class="ob-row bid-row" @click="fillPrice(lvl.price)">
            <span class="ob-price bid">{{ fmt(lvl.price,priceDp) }}</span>
            <span class="ob-size">{{ fmtVol(lvl.size) }}</span>
            <span class="ob-total muted">{{ fmtVol(lvl.total) }}</span>
            <div class="ob-bar bid-bar" :style="{width:barPct(lvl.size)+'%'}"></div>
          </div>
        </div>
      </div>
      <div v-else class="ob-empty">{{ loading ? 'Loading…' : 'Select pair above' }}</div>
    </div>

    <!-- ── Center: Chart + Order Form ── -->
    <div class="tr-center">

      <!-- TradingView chart -->
      <div class="tr-chart">
        <div class="chart-topbar">
          <span class="chart-sym">{{ tvSymbol }}</span>
          <div class="chart-ivs">
            <button v-for="iv in INTERVALS" :key="iv"
              :class="['iv-btn',{active:chartInterval===iv}]"
              @click="chartInterval=iv; loadChart()">{{ iv }}</button>
          </div>
          <button class="chart-reload" @click="loadChart" title="Reload chart">↗</button>
        </div>
        <div class="chart-host" ref="chartHost"></div>
      </div>

      <!-- Order Form -->
      <div class="tr-form-col">

      <!-- pair / exchange banner -->
      <div class="form-banner">
        <span class="form-ex">{{ market.exchange.value.toUpperCase() }}</span>
        <span class="form-sym">{{ market.symbol.value }}</span>
        <span v-if="midPrice" class="form-mid">{{ fmt(midPrice,priceDp) }}</span>
      </div>

      <!-- buy / sell tabs -->
      <div class="side-tabs">
        <button :class="['side-tab buy-tab',{active:side==='buy'}]"  @click="side='buy'">BUY</button>
        <button :class="['side-tab sell-tab',{active:side==='sell'}]" @click="side='sell'">SELL</button>
      </div>

      <!-- order type tabs -->
      <div class="otype-tabs">
        <button v-for="t in ORDER_TYPES" :key="t.id"
          :class="['otype-tab',{active:orderType===t.id}]"
          @click="orderType = t.id as 'limit'|'market'">{{ t.label }}</button>
      </div>

      <!-- balances -->
      <div class="bal-row" v-if="balances.length">
        <span class="bal-lbl">Available</span>
        <span v-for="b in balances.slice(0,4)" :key="b.coin" class="bal-val">
          <span class="bal-coin">{{ b.coin }}</span>{{ fmtBal(b.available) }}
        </span>
        <button class="bal-refresh" @click="loadBalances" title="Refresh balances">⟳</button>
      </div>
      <div class="bal-row" v-else-if="noKeys">
        <span class="bal-warn">No API key for {{ market.exchange.value }}. Add one in Profile.</span>
      </div>
      <div class="bal-row" v-else>
        <span class="bal-lbl">Balances</span>
        <button class="bal-refresh" @click="loadBalances">Load ⟳</button>
      </div>

      <!-- price field (limit only) -->
      <label class="f-label" v-if="orderType==='limit'">
        Price
        <div class="f-input-wrap">
          <input v-model.number="price" type="number" step="any" min="0" class="f-input"
            placeholder="0.00" :disabled="submitting" />
          <button class="f-fill-mid" @click="price=midPrice" :disabled="!midPrice" title="Fill mid price">MID</button>
          <button class="f-fill-bid" @click="price=bestBid" :disabled="!bestBid" title="Fill best bid">BID</button>
          <button class="f-fill-ask" @click="price=bestAsk" :disabled="!bestAsk" title="Fill best ask">ASK</button>
        </div>
      </label>

      <!-- quantity -->
      <label class="f-label">
        Quantity
        <div class="f-input-wrap">
          <input v-model.number="qty" type="number" step="any" min="0" class="f-input"
            placeholder="0.00000000" :disabled="submitting" />
        </div>
      </label>

      <!-- pct shortcuts -->
      <div class="pct-row">
        <button v-for="p in [25,50,75,100]" :key="p" class="pct-btn" @click="setPct(p)">{{ p }}%</button>
      </div>

      <!-- order total -->
      <div class="order-total" v-if="qty && (orderType==='market' || price)">
        <span class="ot-lbl">EST. TOTAL</span>
        <span class="ot-val">{{ fmtBal(orderTotal) }} {{ quoteCoin }}</span>
      </div>

      <!-- submit -->
      <button :class="['submit-btn', side==='buy'?'submit-buy':'submit-sell']"
        :disabled="submitting || !canSubmit"
        @click="placeOrder">
        {{ submitting ? '…' : `${side.toUpperCase()} ${market.symbol.value}` }}
      </button>

      <p v-if="formError" class="form-error">{{ formError }}</p>
      <p v-if="formOk"    class="form-ok">{{ formOk }}</p>

      </div><!-- /tr-form-col -->
    </div><!-- /tr-center -->

    <!-- ── Right: Orders + History ── -->
    <div class="tr-right">

      <!-- tab bar -->
      <div class="r-tabs">
        <button v-for="t in RIGHT_TABS" :key="t.id"
          :class="['r-tab',{active:rightTab===t.id}]"
          @click="rightTab=t.id; if(t.id==='open') loadOpenOrders(); if(t.id==='history') loadOrderHistory(); if(t.id==='fills') loadFills();">
          {{ t.label }}
          <span v-if="t.id==='open' && openOrders.length" class="r-badge">{{ openOrders.length }}</span>
        </button>
        <button class="r-refresh" @click="refreshRight">⟳</button>
      </div>

      <!-- Open orders -->
      <div v-if="rightTab==='open'" class="r-panel">
        <div v-if="!openOrders.length" class="r-empty">{{ loadingRight ? 'Loading…' : 'No open orders' }}</div>
        <div v-for="o in openOrders" :key="o.id" class="ord-row">
          <div class="ord-top">
            <span :class="['ord-side',o.side==='Buy'?'buy':'sell']">{{ o.side.toUpperCase() }}</span>
            <span class="ord-sym">{{ o.symbol }}</span>
            <span class="ord-type">{{ o.order_type }}</span>
            <button class="ord-cancel" @click="cancelOrder(o.id)" title="Cancel">✕</button>
          </div>
          <div class="ord-bot">
            <span class="ord-qty">{{ fmtBal(o.quantity) }}</span>
            <span v-if="o.price" class="ord-price">@ {{ fmt(o.price,priceDp) }}</span>
            <span class="ord-fill muted">filled {{ fmtBal(o.filled_qty) }}</span>
          </div>
        </div>
      </div>

      <!-- Order history -->
      <div v-if="rightTab==='history'" class="r-panel">
        <div v-if="!orderHistory.length" class="r-empty">{{ loadingRight ? 'Loading…' : 'No history' }}</div>
        <div v-for="o in orderHistory" :key="o.id" class="ord-row">
          <div class="ord-top">
            <span :class="['ord-side',o.side==='Buy'?'buy':'sell']">{{ o.side.toUpperCase() }}</span>
            <span class="ord-sym">{{ o.symbol }}</span>
            <span :class="['ord-status',o.status.toLowerCase()]">{{ o.status }}</span>
          </div>
          <div class="ord-bot">
            <span class="ord-qty">{{ fmtBal(o.quantity) }}</span>
            <span v-if="o.price" class="ord-price">@ {{ fmt(o.price,priceDp) }}</span>
            <span class="ord-time muted">{{ fmtTime(o.created_at) }}</span>
          </div>
        </div>
      </div>

      <!-- Trade fills -->
      <div v-if="rightTab==='fills'" class="r-panel">
        <div v-if="!fills.length" class="r-empty">{{ loadingRight ? 'Loading…' : 'No fills' }}</div>
        <div v-for="f in fills" :key="f.id" class="ord-row">
          <div class="ord-top">
            <span :class="['ord-side',f.side==='Buy'?'buy':'sell']">{{ f.side.toUpperCase() }}</span>
            <span class="ord-sym">{{ f.symbol }}</span>
            <span v-if="f.fee" class="ord-fee muted">fee {{ fmtBal(f.fee) }}</span>
          </div>
          <div class="ord-bot">
            <span class="ord-qty">{{ fmtBal(f.quantity) }}</span>
            <span class="ord-price">@ {{ fmt(f.price,priceDp) }}</span>
            <span class="ord-time muted">{{ fmtTime(f.timestamp) }}</span>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useMarket } from '../composables/useMarket'
import { useInvoke } from '../composables/useTauri'

const market = useMarket()
const invoke = useInvoke()

// ── Constants ─────────────────────────────────────────────────────────────────
const DEPTHS    = [10, 15, 20, 30, 50]
const REST_MS   = 5000
const INTERVALS = ['1', '5', '15', '30', '60', '240', 'D', 'W']

// ── TradingView chart ─────────────────────────────────────────────────────────
const chartHost     = ref<HTMLElement | null>(null)
const chartInterval = ref('D')

const VALID_SYMBOL = /^[A-Z0-9_]{1,20}:[A-Z0-9_.!/-]{1,35}$/

function toTvSymbol(exchange: string, symbol: string): string {
  const sym = symbol.replace(/[/-]/g, '').toUpperCase()
  const ex  = exchange.toUpperCase()
  const exMap: Record<string, string> = {
    KRAKEN: 'KRAKEN', COINBASE: 'COINBASE', BINANCE: 'BINANCE',
    BYBIT: 'BYBIT', OKX: 'OKX', KUCOIN: 'KUCOIN',
    HYPERLIQUID: 'BINANCE',
  }
  return `${exMap[ex] ?? ex}:${sym}`
}

const tvSymbol = computed(() => toTvSymbol(market.exchange.value, market.symbol.value))

function loadChart() {
  const sym = tvSymbol.value
  if (!VALID_SYMBOL.test(sym) || !chartHost.value) return
  const settings = {
    autosize: true, symbol: sym, interval: chartInterval.value,
    timezone: 'Etc/UTC', theme: 'dark', style: '1', locale: 'en',
    allow_symbol_change: false, hide_side_toolbar: true,
    hide_top_toolbar: false, hide_legend: false, hide_volume: false,
    withdateranges: true, save_image: false, details: false,
    calendar: false, hotlist: false,
    support_host: 'https://www.tradingview.com',
  }
  const url = new URL('https://www.tradingview-widget.com/embed-widget/advanced-chart/')
  url.searchParams.set('locale', 'en')
  url.hash = encodeURIComponent(JSON.stringify(settings))
  const frame = document.createElement('iframe')
  frame.title = `Chart ${sym}`
  frame.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads')
  frame.setAttribute('allow', 'fullscreen')
  frame.referrerPolicy = 'strict-origin-when-cross-origin'
  frame.style.cssText = 'width:100%;height:100%;border:none;display:block;'
  chartHost.value.replaceChildren(frame)
  frame.src = url.href
}

watch([() => market.exchange.value, () => market.symbol.value], loadChart)
const ORDER_TYPES = [{ id: 'limit', label: 'Limit' }, { id: 'market', label: 'Market' }]
const RIGHT_TABS  = [{ id: 'open', label: 'Open' }, { id: 'history', label: 'History' }, { id: 'fills', label: 'Fills' }]

// ── Orderbook state ───────────────────────────────────────────────────────────
const depth   = ref(20)
const ob      = ref<any>(null)
const loading = ref(false)
const isLive  = ref(false)
let ws: WebSocket | null = null
let restTimer: ReturnType<typeof setInterval> | null = null
const bidsMap = new Map<string, number>()
const asksMap = new Map<string, number>()

// ── Order form state ──────────────────────────────────────────────────────────
const side       = ref<'buy' | 'sell'>('buy')
const orderType  = ref<'limit' | 'market'>('limit')
const price      = ref<number | ''>('')
const qty        = ref<number | ''>('')
const submitting = ref(false)
const formError  = ref('')
const formOk     = ref('')

// ── Right panel ───────────────────────────────────────────────────────────────
const rightTab    = ref('open')
const loadingRight = ref(false)
const openOrders   = ref<any[]>([])
const orderHistory = ref<any[]>([])
const fills        = ref<any[]>([])
const balances     = ref<any[]>([])
const noKeys       = ref(false)

// ── Symbol helpers ────────────────────────────────────────────────────────────
function normRest(exchange: string, symbol: string) {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return symbol.replace('/', '')
  if (ex === 'coinbase')    return symbol.replace('/', '-')
  if (ex === 'hyperliquid') return symbol.replace(/\/.*/, '')
  return symbol
}
function normWs(exchange: string, symbol: string) {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return symbol
  if (ex === 'coinbase')    return symbol.replace('/', '-')
  if (ex === 'hyperliquid') return symbol.replace(/\/.*/, '')
  return symbol
}

// ── REST fetchers ─────────────────────────────────────────────────────────────
async function restFetch(exchange: string, symbol: string, d: number) {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken') {
    const pair = normRest(exchange, symbol)
    const j = await (await fetch(`https://api.kraken.com/0/public/Depth?pair=${pair}&count=${d}`)).json()
    if (j.error?.length) throw new Error(j.error[0])
    const data = Object.values(j.result as Record<string, any>)[0] as any
    return { bids: data.bids.map(([p, s]: string[]) => ({ price: +p, size: +s })), asks: data.asks.map(([p, s]: string[]) => ({ price: +p, size: +s })) }
  }
  if (ex === 'coinbase') {
    const pair = normRest(exchange, symbol)
    const j = await (await fetch(`https://api.exchange.coinbase.com/products/${pair}/book?level=2`)).json()
    return { bids: (j.bids as string[][]).slice(0, d).map(([p, s]) => ({ price: +p, size: +s })), asks: (j.asks as string[][]).slice(0, d).map(([p, s]) => ({ price: +p, size: +s })) }
  }
  if (ex === 'hyperliquid') {
    const coin = normRest(exchange, symbol)
    const j = await (await fetch('https://api.hyperliquid.xyz/info', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ type: 'l2Book', coin }) })).json()
    const lvls = j.levels as [{ px: string; sz: string }[], { px: string; sz: string }[]]
    return { bids: lvls[0].slice(0, d).map(l => ({ price: +l.px, size: +l.sz })), asks: lvls[1].slice(0, d).map(l => ({ price: +l.px, size: +l.sz })) }
  }
  throw new Error(`No REST for ${exchange}`)
}

function withTotals(levels: { price: number; size: number }[]) {
  let cum = 0; return levels.map(l => { cum += l.size; return { ...l, total: cum } })
}
function snapFromMap() {
  const bids = [...bidsMap.entries()].filter(([, s]) => s > 0).map(([p, s]) => ({ price: +p, size: s })).sort((a, b) => b.price - a.price).slice(0, depth.value)
  const asks = [...asksMap.entries()].filter(([, s]) => s > 0).map(([p, s]) => ({ price: +p, size: s })).sort((a, b) => a.price - b.price).slice(0, depth.value)
  return { bids: withTotals(bids), asks: withTotals(asks) }
}

async function restSnapshot() {
  const ex = market.exchange.value; const sym = market.symbol.value
  if (!ex || !sym) return
  loading.value = true
  try {
    const raw = await restFetch(ex, sym, depth.value)
    bidsMap.clear(); asksMap.clear()
    raw.bids.forEach((l: { price: number; size: number }) => bidsMap.set(l.price.toString(), l.size))
    raw.asks.forEach((l: { price: number; size: number }) => asksMap.set(l.price.toString(), l.size))
    ob.value = snapFromMap(); isLive.value = true
  } catch (e) { console.warn('OB REST:', e) } finally { loading.value = false }
}

// ── WebSocket ─────────────────────────────────────────────────────────────────
function openWs(exchange: string, symbol: string) {
  closeWs()
  const ex = exchange.toLowerCase()
  let sock: WebSocket

  if (ex === 'kraken') {
    sock = new WebSocket('wss://ws.kraken.com')
    sock.onopen = () => sock.send(JSON.stringify({ event: 'subscribe', pair: [normWs(exchange, symbol)], subscription: { name: 'book', depth: Math.min(depth.value, 500) } }))
    sock.onmessage = (e) => {
      const msg = JSON.parse(e.data); if (!Array.isArray(msg)) return
      const data = msg[1] as any
      const apply = (arr: string[][], m: Map<string, number>) => arr?.forEach(([p, s]) => { const sz = +s; if (sz === 0) m.delete(p); else m.set(p, sz) })
      apply(data.as ?? data.a, asksMap); apply(data.bs ?? data.b, bidsMap)
      ob.value = snapFromMap(); isLive.value = true
    }
  } else if (ex === 'coinbase') {
    sock = new WebSocket('wss://ws-feed.exchange.coinbase.com')
    sock.onopen = () => sock.send(JSON.stringify({ type: 'subscribe', product_ids: [normWs(exchange, symbol)], channels: ['level2'] }))
    sock.onmessage = (e) => {
      const msg = JSON.parse(e.data)
      if (msg.type === 'snapshot') {
        bidsMap.clear(); asksMap.clear()
        ;(msg.bids as string[][]).forEach(([p, s]) => bidsMap.set(p, +s))
        ;(msg.asks as string[][]).forEach(([p, s]) => asksMap.set(p, +s))
      } else if (msg.type === 'l2update') {
        ;(msg.changes as string[][]).forEach(([side, p, s]) => { const m = side === 'buy' ? bidsMap : asksMap; const sz = +s; if (sz === 0) m.delete(p); else m.set(p, sz) })
      } else return
      ob.value = snapFromMap(); isLive.value = true
    }
  } else if (ex === 'hyperliquid') {
    sock = new WebSocket('wss://api.hyperliquid.xyz/ws')
    sock.onopen = () => sock.send(JSON.stringify({ method: 'subscribe', subscription: { type: 'l2Book', coin: normWs(exchange, symbol) } }))
    sock.onmessage = (e) => {
      const msg = JSON.parse(e.data); const data = msg.data; if (!data?.levels) return
      bidsMap.clear(); asksMap.clear()
      ;(data.levels[0] as { px: string; sz: string }[]).forEach(l => bidsMap.set(l.px, +l.sz))
      ;(data.levels[1] as { px: string; sz: string }[]).forEach(l => asksMap.set(l.px, +l.sz))
      ob.value = snapFromMap(); isLive.value = true
    }
  } else return

  sock.onerror = () => isLive.value = false
  sock.onclose = () => isLive.value = false
  ws = sock
}

function closeWs() { if (ws && ws.readyState < 2) ws.close(); ws = null }

async function connect() {
  closeWs(); bidsMap.clear(); asksMap.clear(); ob.value = null; isLive.value = false
  const ex = market.exchange.value; const sym = market.symbol.value
  if (!ex || !sym) return
  await restSnapshot()
  openWs(ex, sym)
}

// ── Computed orderbook ────────────────────────────────────────────────────────
const askLevels  = computed(() => [...(ob.value?.asks ?? [])].reverse())
const bidLevels  = computed(() => ob.value?.bids ?? [])
const bestBid    = computed(() => ob.value?.bids?.[0]?.price ?? 0)
const bestAsk    = computed(() => ob.value?.asks?.[0]?.price ?? 0)
const spread     = computed(() => bestAsk.value - bestBid.value)
const midPrice   = computed(() => (bestBid.value + bestAsk.value) / 2)
const spreadPct  = computed(() => midPrice.value > 0 ? spread.value / midPrice.value * 100 : 0)
const bidDepth5  = computed(() => (ob.value?.bids ?? []).slice(0, 5).reduce((s: number, l: any) => s + l.size, 0))
const askDepth5  = computed(() => (ob.value?.asks ?? []).slice(0, 5).reduce((s: number, l: any) => s + l.size, 0))
const imbalance  = computed(() => { const b = bidDepth5.value, a = askDepth5.value; return (b + a) ? (b - a) / (b + a) : 0 })
const maxSize    = computed(() => Math.max(...(ob.value?.bids ?? []).map((l: any) => l.size), ...(ob.value?.asks ?? []).map((l: any) => l.size), 1))
function barPct(size: number) { return Math.min(100, (size / maxSize.value) * 100) }

function inferDp(p: number) { return p >= 1000 ? 2 : p >= 1 ? 4 : 6 }
const priceDp = computed(() => inferDp(midPrice.value))

// ── Formatters ────────────────────────────────────────────────────────────────
function fmt(n: number, dp = 2) { return n ? n.toFixed(dp) : '—' }
function fmtVol(n: number) {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M'
  if (n >= 1_000)     return (n / 1_000).toFixed(2) + 'K'
  return n.toFixed(4)
}
function fmtBal(n: number) { return n ? n.toFixed(8).replace(/\.?0+$/, '') : '0' }
function fmtTime(ts: number) {
  if (!ts) return ''
  const d = new Date(ts < 1e12 ? ts * 1000 : ts)
  return d.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })
}

// ── Quote coin ────────────────────────────────────────────────────────────────
const quoteCoin = computed(() => {
  const sym = market.symbol.value
  const parts = sym.split(/[\/\-]/)
  return parts.length >= 2 ? parts[1] : 'USD'
})

// ── Order form helpers ────────────────────────────────────────────────────────
const orderTotal = computed(() => {
  const q = +qty.value || 0
  const p = orderType.value === 'market' ? midPrice.value : (+price.value || 0)
  return q * p
})

const canSubmit = computed(() =>
  !!qty.value && +qty.value > 0 &&
  (orderType.value === 'market' || (!!price.value && +price.value > 0))
)

function fillPrice(p: number) {
  price.value = p
  if (side.value === 'buy' && p <= bestBid.value) side.value = 'buy'
  if (side.value === 'sell' && p >= bestAsk.value) side.value = 'sell'
}

function setPct(pct: number) {
  const base = balances.value.find(b => {
    const sym = market.symbol.value
    const coin = sym.split(/[\/\-]/)[0]
    // for buy use quote coin; for sell use base coin
    if (side.value === 'buy')  return b.coin === (sym.split(/[\/\-]/)[1] ?? 'USD')
    if (side.value === 'sell') return b.coin === coin
    return false
  })
  if (!base) return
  const p = +price.value || midPrice.value
  if (side.value === 'buy' && p > 0) {
    qty.value = parseFloat(((base.available * pct / 100) / p).toFixed(8))
  } else if (side.value === 'sell') {
    qty.value = parseFloat((base.available * pct / 100).toFixed(8))
  }
}

// ── API calls ─────────────────────────────────────────────────────────────────
async function loadBalances() {
  const ex = market.exchange.value
  try {
    const list = await invoke<any[]>('exchange_balances', { exchange: ex })
    balances.value = list.filter(b => b.available > 0 || b.total > 0)
    noKeys.value = false
  } catch (e: any) {
    if (String(e).includes('no private API')) noKeys.value = true
    balances.value = []
  }
}

async function placeOrder() {
  formError.value = ''; formOk.value = ''
  if (!canSubmit.value) return
  submitting.value = true
  try {
    const order = await invoke<any>('exchange_place_order', {
      exchange:  market.exchange.value,
      symbol:    market.symbol.value,
      side:      side.value,
      orderType: orderType.value,
      qty:       +qty.value,
      price:     orderType.value === 'limit' ? +price.value : null,
    })
    formOk.value = `Order placed: ${order.id ?? 'OK'}`
    qty.value = ''; price.value = ''
    await loadOpenOrders()
    await loadBalances()
  } catch (e: any) {
    formError.value = String(e)
  } finally { submitting.value = false }
}

async function cancelOrder(id: string) {
  try {
    await invoke('exchange_cancel_order', { exchange: market.exchange.value, orderId: id })
    await loadOpenOrders()
    await loadBalances()
  } catch (e: any) { formError.value = String(e) }
}

async function loadOpenOrders() {
  loadingRight.value = true
  try { openOrders.value = await invoke<any[]>('exchange_open_orders', { exchange: market.exchange.value }) }
  catch { openOrders.value = [] } finally { loadingRight.value = false }
}

async function loadOrderHistory() {
  loadingRight.value = true
  try { orderHistory.value = await invoke<any[]>('exchange_order_history', { exchange: market.exchange.value, limit: 50 }) }
  catch { orderHistory.value = [] } finally { loadingRight.value = false }
}

async function loadFills() {
  loadingRight.value = true
  try { fills.value = await invoke<any[]>('exchange_trade_history', { exchange: market.exchange.value, limit: 50 }) }
  catch { fills.value = [] } finally { loadingRight.value = false }
}

function refreshRight() {
  if (rightTab.value === 'open')    loadOpenOrders()
  if (rightTab.value === 'history') loadOrderHistory()
  if (rightTab.value === 'fills')   loadFills()
  loadBalances()
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────
watch([() => market.exchange.value, () => market.symbol.value, depth], connect)
watch(() => market.exchange.value, () => { loadBalances(); loadOpenOrders() })

onMounted(() => {
  connect()
  loadChart()
  restTimer = setInterval(() => { if (!isLive.value) restSnapshot() }, REST_MS)
  loadBalances()
  loadOpenOrders()
})
onUnmounted(() => { closeWs(); if (restTimer) clearInterval(restTimer) })
</script>

<style scoped>
.tr-shell {
  display: grid;
  grid-template-columns: 220px 1fr 280px;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
  font-family: var(--mono, monospace);
  gap: 1px;
  background-color: rgba(255,255,255,0.04);
}

/* ── Center column: chart on top, form below ── */
.tr-center {
  display: grid;
  grid-template-rows: 1fr 340px;
  overflow: hidden;
  background: var(--bg);
  min-height: 0;
}

/* ── TradingView chart ── */
.tr-chart {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
  background: #0a0e1a;
}
.chart-topbar {
  display: flex; align-items: center; gap: 6px; flex-shrink: 0;
  padding: 4px 10px;
  background: rgba(14,22,40,0.95);
  border-bottom: 1px solid var(--border);
}
.chart-sym {
  font-size: 10px; font-weight: 700; color: #3b82f6; letter-spacing: .06em;
}
.chart-ivs { display: flex; gap: 2px; margin-left: 6px; }
.iv-btn {
  padding: 2px 6px; border-radius: 3px; font-size: 9px; font-weight: 700;
  background: transparent; border: 1px solid rgba(255,255,255,0.07);
  color: rgba(255,255,255,0.3); cursor: pointer; transition: all .1s;
}
.iv-btn:hover  { border-color: rgba(59,130,246,0.4); color: rgba(255,255,255,0.7); }
.iv-btn.active { background: rgba(59,130,246,0.18); border-color: #3b82f6; color: #3b82f6; }
.chart-reload {
  margin-left: auto; padding: 2px 7px; background: transparent;
  border: 1px solid rgba(255,255,255,0.08); border-radius: 3px;
  color: rgba(255,255,255,0.35); cursor: pointer; font-size: 11px;
}
.chart-reload:hover { color: #e0e0e0; border-color: rgba(255,255,255,0.25); }
.chart-host {
  flex: 1; min-height: 0; overflow: hidden;
}

/* ── Panel header ── */
.panel-head {
  display: flex; align-items: center; gap: 6px; flex-shrink: 0;
  padding: 5px 10px;
  font-size: 9px; font-weight: 700; letter-spacing: .1em; color: var(--text-muted);
  background: rgba(14,22,40,0.9);
  border-bottom: 1px solid var(--border);
}
.panel-head .depth-picker { display: flex; gap: 2px; margin-left: 4px; }
.depth-btn { padding: 1px 5px; border-radius: 3px; font-size: 9px; font-weight: 600; background: transparent; border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.3); cursor: pointer; }
.depth-btn.active { background: rgba(59,130,246,0.18); border-color: #3b82f6; color: #3b82f6; }
.ob-live { margin-left: auto; font-size: 9px; font-weight: 700; color: rgba(255,255,255,0.25); }
.ob-live.live { color: #4ade80; }

/* ── Left: Orderbook ── */
.tr-ob {
  display: flex; flex-direction: column; overflow: hidden;
  background: var(--bg);
}
.ob-metrics {
  display: flex; gap: 0; flex-shrink: 0; padding: 3px 8px;
  background: rgba(10,16,28,0.7); border-bottom: 1px solid rgba(59,130,246,0.06);
}
.ob-m { display: flex; flex-direction: column; gap: 1px; padding: 0 8px; border-right: 1px solid rgba(255,255,255,0.04); flex-shrink: 0; }
.ob-m:last-child { border-right: none; }
.ob-ml { font-size: 7px; font-weight: 700; letter-spacing: .06em; color: rgba(255,255,255,0.25); }
.ob-m > span:last-child { font-size: 10px; font-weight: 700; color: #e0e0e0; }
.g { color: #4ade80 !important; }
.y { color: #fbbf24 !important; }

.ob-book { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.ob-asks, .ob-bids { flex: 1; overflow-y: auto; scrollbar-width: none; }
.ob-asks::-webkit-scrollbar, .ob-bids::-webkit-scrollbar { display: none; }
.ob-asks { display: flex; flex-direction: column; justify-content: flex-end; }

.ob-row {
  display: grid; grid-template-columns: 1fr 1fr 1fr;
  align-items: center; position: relative;
  padding: 1px 8px; font-size: 9px; font-weight: 600;
  min-height: 16px; cursor: pointer;
}
.ob-row:hover { background: rgba(255,255,255,0.06); }
.ob-price { z-index: 1; }
.ob-price.ask { color: #f87171; }
.ob-price.bid { color: #4ade80; }
.ob-size { z-index: 1; text-align: center; color: rgba(255,255,255,0.6); }
.ob-total { z-index: 1; text-align: right; }
.muted { color: rgba(255,255,255,0.3); font-weight: 400; }
.ob-bar { position: absolute; top: 0; bottom: 0; right: 0; opacity: 0.1; }
.ask-bar { background: #f87171; }
.bid-bar { background: #4ade80; }
.ob-spread-row { display: flex; align-items: center; justify-content: center; gap: 4px; padding: 2px 8px; flex-shrink: 0; background: rgba(255,255,255,0.02); border-top: 1px solid rgba(255,255,255,0.04); border-bottom: 1px solid rgba(255,255,255,0.04); }
.ob-spread-val { font-size: 9px; font-weight: 700; color: #e0e0e0; }
.ob-spread-pct { font-size: 8px; color: rgba(255,255,255,0.35); }
.ob-empty { display: flex; align-items: center; justify-content: center; flex: 1; color: rgba(255,255,255,0.2); font-size: 11px; }

/* ── Center: Order form ── */
.tr-form-col {
  display: flex; flex-direction: column; gap: 8px;
  padding: 12px 14px; overflow-y: auto; scrollbar-width: thin;
  background: var(--surface);
}
.form-banner { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.form-ex  { font-size: 9px; font-weight: 700; letter-spacing: .1em; color: #3b82f6; }
.form-sym { font-size: 14px; font-weight: 700; color: #e0e0e0; }
.form-mid { font-size: 12px; font-weight: 700; color: rgba(255,255,255,0.5); margin-left: auto; }

.side-tabs { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; flex-shrink: 0; }
.side-tab { padding: 8px; border-radius: 5px; font-size: 12px; font-weight: 800; cursor: pointer; border: 1px solid transparent; transition: all .15s; letter-spacing: .06em; }
.buy-tab  { background: rgba(74,222,128,0.06); border-color: rgba(74,222,128,0.2); color: rgba(74,222,128,0.5); }
.buy-tab.active  { background: rgba(74,222,128,0.18); border-color: #4ade80; color: #4ade80; }
.sell-tab { background: rgba(248,113,113,0.06); border-color: rgba(248,113,113,0.2); color: rgba(248,113,113,0.5); }
.sell-tab.active { background: rgba(248,113,113,0.18); border-color: #f87171; color: #f87171; }

.otype-tabs { display: flex; gap: 3px; flex-shrink: 0; }
.otype-tab { flex: 1; padding: 4px 6px; border-radius: 4px; font-size: 10px; font-weight: 700; cursor: pointer; background: transparent; border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.3); transition: all .12s; }
.otype-tab:hover { border-color: rgba(59,130,246,0.4); color: rgba(255,255,255,0.7); }
.otype-tab.active { background: rgba(59,130,246,0.12); border-color: #3b82f6; color: #60a5fa; }

.bal-row { display: flex; align-items: center; gap: 8px; flex-shrink: 0; padding: 5px 8px; background: rgba(10,16,28,0.6); border-radius: 5px; flex-wrap: wrap; }
.bal-lbl  { font-size: 9px; font-weight: 700; letter-spacing: .08em; color: rgba(255,255,255,0.3); }
.bal-val  { font-size: 10px; font-weight: 700; color: #e0e0e0; display: flex; align-items: center; gap: 3px; }
.bal-coin { font-size: 8px; color: rgba(255,255,255,0.4); }
.bal-refresh { margin-left: auto; background: transparent; border: 1px solid rgba(255,255,255,0.1); border-radius: 3px; color: rgba(255,255,255,0.4); cursor: pointer; padding: 1px 5px; font-size: 11px; }
.bal-refresh:hover { color: #e0e0e0; border-color: rgba(255,255,255,0.3); }
.bal-warn { font-size: 10px; color: #fbbf24; }

.f-label { display: flex; flex-direction: column; gap: 4px; font-size: 9px; font-weight: 700; letter-spacing: .06em; color: rgba(255,255,255,0.35); flex-shrink: 0; }
.f-input-wrap { display: flex; align-items: center; gap: 3px; }
.f-input { flex: 1; padding: 7px 9px; background: rgba(10,14,26,0.8); border: 1px solid rgba(59,130,246,0.25); border-radius: 5px; color: #e0e0e0; font-size: 12px; font-family: var(--mono, monospace); outline: none; min-width: 0; }
.f-input:focus { border-color: #3b82f6; }
.f-input:disabled { opacity: .5; }
.f-fill-mid, .f-fill-bid, .f-fill-ask { padding: 3px 6px; border-radius: 3px; font-size: 9px; font-weight: 700; cursor: pointer; border: 1px solid; flex-shrink: 0; }
.f-fill-mid { background: rgba(59,130,246,0.1); border-color: rgba(59,130,246,0.3); color: #60a5fa; }
.f-fill-bid { background: rgba(74,222,128,0.1); border-color: rgba(74,222,128,0.3); color: #4ade80; }
.f-fill-ask { background: rgba(248,113,113,0.1); border-color: rgba(248,113,113,0.3); color: #f87171; }
.f-fill-mid:disabled, .f-fill-bid:disabled, .f-fill-ask:disabled { opacity: .3; cursor: not-allowed; }

.pct-row { display: flex; gap: 4px; flex-shrink: 0; }
.pct-btn { flex: 1; padding: 4px; border-radius: 4px; font-size: 10px; font-weight: 700; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.4); cursor: pointer; transition: all .1s; }
.pct-btn:hover { border-color: rgba(59,130,246,0.4); color: #60a5fa; }

.order-total { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: rgba(10,14,26,0.6); border-radius: 5px; flex-shrink: 0; }
.ot-lbl { font-size: 9px; font-weight: 700; letter-spacing: .08em; color: rgba(255,255,255,0.3); }
.ot-val { font-size: 13px; font-weight: 700; color: #e0e0e0; }

.submit-btn { padding: 11px; border-radius: 6px; font-size: 13px; font-weight: 800; cursor: pointer; border: none; letter-spacing: .04em; transition: opacity .12s, transform .08s; flex-shrink: 0; }
.submit-btn:active { transform: scale(0.98); }
.submit-btn:disabled { opacity: .4; cursor: not-allowed; }
.submit-buy  { background: #4ade80; color: #0a1a0a; }
.submit-sell { background: #f87171; color: #1a0a0a; }
.submit-buy:hover:not(:disabled)  { background: #22c55e; }
.submit-sell:hover:not(:disabled) { background: #ef4444; }

.form-error { font-size: 11px; color: #f87171; margin: 0; }
.form-ok    { font-size: 11px; color: #4ade80;  margin: 0; }

/* ── Right panel ── */
.tr-right {
  display: flex; flex-direction: column; overflow: hidden;
  background: var(--bg);
}
.r-tabs { display: flex; align-items: center; flex-shrink: 0; background: rgba(14,22,40,0.9); border-bottom: 1px solid var(--border); }
.r-tab { padding: 7px 12px; font-size: 10px; font-weight: 700; cursor: pointer; background: transparent; border: none; border-bottom: 2px solid transparent; color: rgba(255,255,255,0.3); transition: all .12s; display: flex; align-items: center; gap: 4px; }
.r-tab:hover { color: rgba(255,255,255,0.7); }
.r-tab.active { color: #3b82f6; border-bottom-color: #3b82f6; background: rgba(59,130,246,0.04); }
.r-badge { background: rgba(59,130,246,0.25); color: #60a5fa; border-radius: 8px; font-size: 8px; padding: 1px 5px; font-weight: 800; }
.r-refresh { margin-left: auto; padding: 4px 8px; background: transparent; border: none; color: rgba(255,255,255,0.3); cursor: pointer; font-size: 14px; }
.r-refresh:hover { color: #e0e0e0; }

.r-panel { flex: 1; overflow-y: auto; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.1) transparent; }
.r-empty { display: flex; align-items: center; justify-content: center; height: 120px; color: rgba(255,255,255,0.2); font-size: 11px; }

.ord-row { padding: 8px 12px; border-bottom: 1px solid rgba(255,255,255,0.04); }
.ord-top { display: flex; align-items: center; gap: 6px; margin-bottom: 3px; }
.ord-bot { display: flex; align-items: center; gap: 6px; }
.ord-side { font-size: 9px; font-weight: 800; letter-spacing: .08em; }
.ord-side.buy  { color: #4ade80; }
.ord-side.sell { color: #f87171; }
.ord-sym { font-size: 11px; font-weight: 700; color: #e0e0e0; }
.ord-type { font-size: 9px; color: rgba(255,255,255,0.3); margin-left: auto; }
.ord-cancel { margin-left: 6px; background: transparent; border: 1px solid rgba(248,113,113,0.25); color: #f87171; border-radius: 3px; cursor: pointer; font-size: 9px; padding: 1px 5px; }
.ord-cancel:hover { background: rgba(248,113,113,0.15); }
.ord-qty   { font-size: 11px; font-weight: 700; color: #e0e0e0; }
.ord-price { font-size: 10px; color: rgba(255,255,255,0.6); }
.ord-fill  { font-size: 9px; margin-left: auto; }
.ord-time  { font-size: 9px; margin-left: auto; }
.ord-fee   { font-size: 9px; margin-left: auto; }
.ord-status { font-size: 9px; font-weight: 700; margin-left: auto; }
.ord-status.filled    { color: #4ade80; }
.ord-status.cancelled { color: rgba(255,255,255,0.3); }
.ord-status.open      { color: #60a5fa; }
.ord-status.rejected  { color: #f87171; }
</style>
