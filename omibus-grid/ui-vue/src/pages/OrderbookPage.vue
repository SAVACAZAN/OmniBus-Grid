<template>
  <div class="ob-shell">

    <!-- ── Header ── -->
    <div class="ob-header">
      <span class="ob-eyebrow">ORDER BOOK / 003</span>
      <div class="ob-hsep"></div>
      <span class="ob-pair">{{ market.exchange.value.toUpperCase() }} · {{ market.symbol.value }}</span>
      <div class="ob-hsep"></div>

      <!-- Depth selector -->
      <div class="depth-picker">
        <button v-for="d in DEPTHS" :key="d"
          :class="['depth-btn', { active: depth === d }]"
          @click="depth = d">{{ d }}</button>
      </div>

      <div class="ob-hsep"></div>

      <!-- EX2 toggle -->
      <button v-if="market.exchange2.value"
        :class="['ex2-toggle', { active: showEx2 }]"
        @click="showEx2 = !showEx2">
        {{ market.exchange2.value.toUpperCase() }} · {{ market.symbol2.value }}
      </button>

      <div style="flex:1"></div>
      <span class="ob-live" :class="{ live: isLive }">{{ isLive ? '● WS LIVE' : '○ REST' }}</span>
      <span class="ob-latency" v-if="latencyMs > 0">{{ latencyMs }}ms</span>
    </div>

    <!-- ── Metrics bar ── -->
    <div v-if="ob" class="ob-metrics">
      <div class="ob-metric">
        <span class="ob-metric-label">MID</span>
        <span class="ob-metric-val">{{ fmt(midPrice, 4) }}</span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">SPREAD</span>
        <span class="ob-metric-val">{{ fmt(spread, 4) }} <span class="ob-pct">({{ spreadPct.toFixed(3) }}%)</span></span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">BID D5</span>
        <span class="ob-metric-val buy">{{ fmtVol(bidDepth5) }}</span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">ASK D5</span>
        <span class="ob-metric-val sell">{{ fmtVol(askDepth5) }}</span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">IMBALANCE</span>
        <span class="ob-metric-val" :style="{ color: imbalance > 0 ? '#4ade80' : '#f87171' }">
          {{ (imbalance * 100).toFixed(1) }}%
        </span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">BID VOL</span>
        <span class="ob-metric-val buy">{{ fmtVol(bidDepthFull) }}</span>
      </div>
      <div class="ob-metric">
        <span class="ob-metric-label">ASK VOL</span>
        <span class="ob-metric-val sell">{{ fmtVol(askDepthFull) }}</span>
      </div>
    </div>

    <!-- ── Main content ── -->
    <div class="ob-content" :class="{ 'dual': showEx2 && ob2 }">

      <!-- EX1 ladder -->
      <div class="ob-book" v-if="ob">
        <div class="ob-book-label">{{ market.exchange.value.toUpperCase() }}</div>

        <!-- Asks (reversed — lowest ask at bottom near mid) -->
        <div class="ob-asks">
          <div v-for="level in askLevels" :key="'a'+level.price" class="ob-row ask-row">
            <span class="ob-price ask">{{ fmt(level.price, priceDp) }}</span>
            <span class="ob-size">{{ fmtVol(level.size) }}</span>
            <span class="ob-total">{{ fmtVol(level.total) }}</span>
            <div class="ob-bar ask-bar" :style="{ width: barPct(level.size) + '%' }"></div>
          </div>
        </div>

        <!-- Spread row -->
        <div class="ob-spread-row">
          <span class="ob-spread-val">{{ fmt(spread, 4) }}</span>
          <span class="ob-spread-pct">{{ spreadPct.toFixed(3) }}%</span>
        </div>

        <!-- Bids -->
        <div class="ob-bids">
          <div v-for="level in bidLevels" :key="'b'+level.price" class="ob-row bid-row">
            <span class="ob-price bid">{{ fmt(level.price, priceDp) }}</span>
            <span class="ob-size">{{ fmtVol(level.size) }}</span>
            <span class="ob-total">{{ fmtVol(level.total) }}</span>
            <div class="ob-bar bid-bar" :style="{ width: barPct(level.size) + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- EX2 ladder -->
      <div class="ob-book ob-book-ex2" v-if="showEx2 && ob2">
        <div class="ob-book-label ex2-label">{{ market.exchange2.value.toUpperCase() }}</div>

        <div class="ob-asks">
          <div v-for="level in askLevels2" :key="'a2'+level.price" class="ob-row ask-row">
            <span class="ob-price ask">{{ fmt(level.price, priceDp2) }}</span>
            <span class="ob-size">{{ fmtVol(level.size) }}</span>
            <span class="ob-total">{{ fmtVol(level.total) }}</span>
            <div class="ob-bar ask-bar" :style="{ width: barPct2(level.size) + '%' }"></div>
          </div>
        </div>

        <div class="ob-spread-row">
          <span class="ob-spread-val">{{ fmt(spread2, 4) }}</span>
          <span class="ob-spread-pct">{{ spreadPct2.toFixed(3) }}%</span>
        </div>

        <div class="ob-bids">
          <div v-for="level in bidLevels2" :key="'b2'+level.price" class="ob-row bid-row">
            <span class="ob-price bid">{{ fmt(level.price, priceDp2) }}</span>
            <span class="ob-size">{{ fmtVol(level.size) }}</span>
            <span class="ob-total">{{ fmtVol(level.total) }}</span>
            <div class="ob-bar bid-bar" :style="{ width: barPct2(level.size) + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="!ob && !loading" class="ob-empty">
        <span>Select an exchange and pair in the topbar to load the order book.</span>
      </div>
      <div v-if="loading && !ob" class="ob-empty">
        <span>Loading…</span>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useMarket } from '../composables/useMarket'

const market = useMarket()

// ── Constants ─────────────────────────────────────────────────────────────────
const DEPTHS = [10, 15, 20, 30, 50]
const REST_REFRESH_MS = 5000   // fallback poll when WS unavailable

// ── State ─────────────────────────────────────────────────────────────────────
const depth     = ref(20)
const ob        = ref<any>(null)
const ob2       = ref<any>(null)
const loading   = ref(false)
const isLive    = ref(false)
const isLive2   = ref(false)
const latencyMs = ref(0)
const showEx2   = ref(false)
let   restTimer: ReturnType<typeof setInterval> | null = null
let   ws1: WebSocket | null = null
let   ws2: WebSocket | null = null

// ── In-memory order book state (for WS incremental updates) ──────────────────
const bidsMap  = new Map<string, number>()  // price → size  (EX1)
const asksMap  = new Map<string, number>()
const bidsMap2 = new Map<string, number>()  // EX2
const asksMap2 = new Map<string, number>()

// ── Symbol normalisation ──────────────────────────────────────────────────────
function normRest(exchange: string, symbol: string): string {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return symbol.replace('/', '')       // XBT/USD → XBTUSD
  if (ex === 'coinbase')    return symbol.replace('/', '-')      // BTC/USD → BTC-USD
  if (ex === 'hyperliquid') return symbol.replace(/\/.*/,'')    // BTC/USD → BTC
  return symbol
}
function normWs(exchange: string, symbol: string): string {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return symbol                        // keep XBT/USD as-is for WS
  if (ex === 'coinbase')    return symbol.replace('/', '-')
  if (ex === 'hyperliquid') return symbol.replace(/\/.*/,'')
  return symbol
}

// ── REST fetch (snapshot + fallback polling) ──────────────────────────────────
async function restKraken(symbol: string, d: number) {
  const pair = normRest('kraken', symbol)
  const j = await (await fetch(`https://api.kraken.com/0/public/Depth?pair=${pair}&count=${d}`)).json()
  if (j.error?.length) throw new Error(j.error[0])
  const data = Object.values(j.result as Record<string,any>)[0] as any
  return { bids: data.bids.map(([p,s]:string[]) => ({ price:+p, size:+s })),
           asks: data.asks.map(([p,s]:string[]) => ({ price:+p, size:+s })) }
}
async function restCoinbase(symbol: string, d: number) {
  const pair = normRest('coinbase', symbol)
  const j = await (await fetch(`https://api.exchange.coinbase.com/products/${pair}/book?level=2`)).json()
  return { bids: (j.bids as string[][]).slice(0,d).map(([p,s])=>({price:+p,size:+s})),
           asks: (j.asks as string[][]).slice(0,d).map(([p,s])=>({price:+p,size:+s})) }
}
async function restHyperliquid(symbol: string, d: number) {
  const coin = normRest('hyperliquid', symbol)
  const j = await (await fetch('https://api.hyperliquid.xyz/info', {
    method:'POST', headers:{'Content-Type':'application/json'},
    body: JSON.stringify({ type:'l2Book', coin }),
  })).json()
  const lvls = j.levels as [{px:string;sz:string}[],{px:string;sz:string}[]]
  return { bids: lvls[0].slice(0,d).map(l=>({price:+l.px,size:+l.sz})),
           asks: lvls[1].slice(0,d).map(l=>({price:+l.px,size:+l.sz})) }
}
async function restFetch(exchange: string, symbol: string, d: number) {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return restKraken(symbol, d)
  if (ex === 'coinbase')    return restCoinbase(symbol, d)
  if (ex === 'hyperliquid') return restHyperliquid(symbol, d)
  throw new Error(`No REST for ${exchange}`)
}

// ── Cumulative totals ─────────────────────────────────────────────────────────
function withTotals(levels: { price:number; size:number }[]) {
  let cum = 0
  return levels.map(l => { cum += l.size; return { ...l, total: cum } })
}

// ── Book map → sorted snapshot ────────────────────────────────────────────────
function snapFromMap(bm: Map<string,number>, am: Map<string,number>, d: number) {
  const bids = [...bm.entries()]
    .filter(([,s])=>s>0).map(([p,s])=>({price:+p,size:s}))
    .sort((a,b)=>b.price-a.price).slice(0,d)
  const asks = [...am.entries()]
    .filter(([,s])=>s>0).map(([p,s])=>({price:+p,size:s}))
    .sort((a,b)=>a.price-b.price).slice(0,d)
  return { bids: withTotals(bids), asks: withTotals(asks) }
}

// ── WebSocket — Kraken ────────────────────────────────────────────────────────
function openKrakenWs(symbol: string, bm: Map<string,number>, am: Map<string,number>,
                      setOb: (v:any)=>void, setLive: (v:boolean)=>void): WebSocket {
  const sock = new WebSocket('wss://ws.kraken.com')
  sock.onopen = () => {
    sock.send(JSON.stringify({ event:'subscribe', pair:[symbol],
      subscription:{ name:'book', depth:Math.min(depth.value, 500) } }))
  }
  sock.onmessage = (e) => {
    const msg = JSON.parse(e.data)
    if (!Array.isArray(msg)) return
    const data = msg[1] as any
    // snapshot: data.as / data.bs   update: data.a / data.b
    const applyLevels = (arr: string[][], map: Map<string,number>) => {
      arr?.forEach(([p,s]) => { const sz=+s; if(sz===0) map.delete(p); else map.set(p,sz) })
    }
    applyLevels(data.as ?? data.a, am)
    applyLevels(data.bs ?? data.b, bm)
    setOb(snapFromMap(bm, am, depth.value))
    setLive(true)
  }
  sock.onerror = () => setLive(false)
  sock.onclose = () => setLive(false)
  return sock
}

// ── WebSocket — Coinbase ──────────────────────────────────────────────────────
function openCoinbaseWs(symbol: string, bm: Map<string,number>, am: Map<string,number>,
                        setOb: (v:any)=>void, setLive: (v:boolean)=>void): WebSocket {
  const sock = new WebSocket('wss://ws-feed.exchange.coinbase.com')
  const productId = normWs('coinbase', symbol)
  sock.onopen = () => {
    sock.send(JSON.stringify({ type:'subscribe', product_ids:[productId], channels:['level2'] }))
  }
  sock.onmessage = (e) => {
    const msg = JSON.parse(e.data)
    if (msg.type === 'snapshot') {
      bm.clear(); am.clear()
      ;(msg.bids as string[][]).forEach(([p,s]) => bm.set(p,+s))
      ;(msg.asks as string[][]).forEach(([p,s]) => am.set(p,+s))
    } else if (msg.type === 'l2update') {
      ;(msg.changes as string[][]).forEach(([side,p,s]) => {
        const map = side==='buy' ? bm : am
        const sz = +s; if(sz===0) map.delete(p); else map.set(p,sz)
      })
    } else return
    setOb(snapFromMap(bm, am, depth.value))
    setLive(true)
  }
  sock.onerror = () => setLive(false)
  sock.onclose = () => setLive(false)
  return sock
}

// ── WebSocket — Hyperliquid ───────────────────────────────────────────────────
function openHyperliquidWs(symbol: string, bm: Map<string,number>, am: Map<string,number>,
                           setOb: (v:any)=>void, setLive: (v:boolean)=>void): WebSocket {
  const coin = normWs('hyperliquid', symbol)
  const sock = new WebSocket('wss://api.hyperliquid.xyz/ws')
  sock.onopen = () => {
    sock.send(JSON.stringify({ method:'subscribe', subscription:{ type:'l2Book', coin } }))
  }
  sock.onmessage = (e) => {
    const msg = JSON.parse(e.data)
    const data = msg.data
    if (!data?.levels) return
    bm.clear(); am.clear()
    ;(data.levels[0] as {px:string;sz:string}[]).forEach(l => bm.set(l.px,+l.sz))
    ;(data.levels[1] as {px:string;sz:string}[]).forEach(l => am.set(l.px,+l.sz))
    setOb(snapFromMap(bm, am, depth.value))
    setLive(true)
  }
  sock.onerror = () => setLive(false)
  sock.onclose = () => setLive(false)
  return sock
}

// ── Open WS or fallback to REST polling ──────────────────────────────────────
function openWs(exchange: string, symbol: string,
                bm: Map<string,number>, am: Map<string,number>,
                setOb: (v:any)=>void, setLive: (v:boolean)=>void): WebSocket | null {
  const ex = exchange.toLowerCase()
  if (ex === 'kraken')      return openKrakenWs(symbol, bm, am, setOb, setLive)
  if (ex === 'coinbase')    return openCoinbaseWs(symbol, bm, am, setOb, setLive)
  if (ex === 'hyperliquid') return openHyperliquidWs(symbol, bm, am, setOb, setLive)
  return null
}

function closeWs(sock: WebSocket | null) {
  if (sock && sock.readyState < 2) sock.close()
}

// ── REST snapshot on connect + periodic REST fallback ────────────────────────
async function restSnapshot(exchange: string, symbol: string,
                            bm: Map<string,number>, am: Map<string,number>,
                            setOb: (v:any)=>void, setLive: (v:boolean)=>void) {
  const t0 = Date.now()
  loading.value = true
  try {
    const raw = await restFetch(exchange, symbol, depth.value)
    bm.clear(); am.clear()
    raw.bids.forEach((l:{price:number;size:number}) => bm.set(l.price.toString(), l.size))
    raw.asks.forEach((l:{price:number;size:number}) => am.set(l.price.toString(), l.size))
    setOb(snapFromMap(bm, am, depth.value))
    latencyMs.value = Date.now() - t0
    setLive(true)
  } catch(e) {
    console.warn('REST snapshot:', e); setLive(false)
  } finally { loading.value = false }
}

// ── Connect / reconnect ───────────────────────────────────────────────────────
async function connectEx1() {
  closeWs(ws1); ws1 = null
  bidsMap.clear(); asksMap.clear()
  ob.value = null; isLive.value = false
  const ex = market.exchange.value; const sym = market.symbol.value
  if (!ex || !sym) return
  // REST snapshot first so we have data immediately
  await restSnapshot(ex, sym, bidsMap, asksMap, v => { ob.value = v }, v => { isLive.value = v })
  // Then open WS for live updates
  ws1 = openWs(ex, sym, bidsMap, asksMap, v => { ob.value = v }, v => { isLive.value = v })
}

async function connectEx2() {
  closeWs(ws2); ws2 = null
  bidsMap2.clear(); asksMap2.clear()
  ob2.value = null; isLive2.value = false
  if (!showEx2.value || !market.exchange2.value || !market.symbol2.value) return
  const ex = market.exchange2.value; const sym = market.symbol2.value
  await restSnapshot(ex, sym, bidsMap2, asksMap2, v => { ob2.value = v }, v => { isLive2.value = v })
  ws2 = openWs(ex, sym, bidsMap2, asksMap2, v => { ob2.value = v }, v => { isLive2.value = v })
}

// ── Computed — EX1 ───────────────────────────────────────────────────────────
const askLevels = computed(() => [...(ob.value?.asks ?? [])].reverse())
const bidLevels = computed(() => ob.value?.bids ?? [])

const bestBid    = computed(() => ob.value?.bids?.[0]?.price ?? 0)
const bestAsk    = computed(() => ob.value?.asks?.[0]?.price ?? 0)
const spread     = computed(() => bestAsk.value - bestBid.value)
const midPrice   = computed(() => (bestBid.value + bestAsk.value) / 2)
const spreadPct  = computed(() => midPrice.value > 0 ? spread.value / midPrice.value * 100 : 0)
const bidDepth5  = computed(() => (ob.value?.bids ?? []).slice(0,5).reduce((s:number,l:any) => s+l.size, 0))
const askDepth5  = computed(() => (ob.value?.asks ?? []).slice(0,5).reduce((s:number,l:any) => s+l.size, 0))
const imbalance  = computed(() => { const b=bidDepth5.value, a=askDepth5.value; return (b+a) ? (b-a)/(b+a) : 0 })
const bidDepthFull = computed(() => (ob.value?.bids ?? []).reduce((s:number,l:any) => s+l.size, 0))
const askDepthFull = computed(() => (ob.value?.asks ?? []).reduce((s:number,l:any) => s+l.size, 0))
const maxSize    = computed(() => Math.max(...(ob.value?.bids??[]).map((l:any)=>l.size), ...(ob.value?.asks??[]).map((l:any)=>l.size), 1))

// ── Computed — EX2 ───────────────────────────────────────────────────────────
const askLevels2 = computed(() => [...(ob2.value?.asks ?? [])].reverse())
const bidLevels2 = computed(() => ob2.value?.bids ?? [])
const bestBid2   = computed(() => ob2.value?.bids?.[0]?.price ?? 0)
const bestAsk2   = computed(() => ob2.value?.asks?.[0]?.price ?? 0)
const spread2    = computed(() => bestAsk2.value - bestBid2.value)
const midPrice2  = computed(() => (bestBid2.value + bestAsk2.value) / 2)
const spreadPct2 = computed(() => midPrice2.value > 0 ? spread2.value / midPrice2.value * 100 : 0)
const maxSize2   = computed(() => Math.max(...(ob2.value?.bids??[]).map((l:any)=>l.size), ...(ob2.value?.asks??[]).map((l:any)=>l.size), 1))

// ── Decimal places ────────────────────────────────────────────────────────────
function inferDp(price: number) { return price >= 1000 ? 2 : price >= 1 ? 4 : 6 }
const priceDp  = computed(() => inferDp(midPrice.value))
const priceDp2 = computed(() => inferDp(midPrice2.value))

// ── Formatters ────────────────────────────────────────────────────────────────
function fmt(n: number, dp = 2) { return n ? n.toFixed(dp) : '—' }
function fmtVol(n: number) {
  if (n >= 1_000_000) return (n/1_000_000).toFixed(2) + 'M'
  if (n >= 1_000)     return (n/1_000).toFixed(2) + 'K'
  return n.toFixed(4)
}
function barPct(size: number)  { return Math.min(100, (size / maxSize.value) * 100) }
function barPct2(size: number) { return Math.min(100, (size / maxSize2.value) * 100) }

// ── Lifecycle ─────────────────────────────────────────────────────────────────
watch([() => market.exchange.value, () => market.symbol.value, depth], connectEx1)
watch([() => market.exchange2.value, () => market.symbol2.value, showEx2, depth], connectEx2)

// REST fallback poll — refreshes snapshot if WS is not live
function restPoll() {
  if (!isLive.value && market.exchange.value && market.symbol.value)
    restSnapshot(market.exchange.value, market.symbol.value,
      bidsMap, asksMap, v => { ob.value = v }, v => { isLive.value = v })
  if (showEx2.value && !isLive2.value && market.exchange2.value && market.symbol2.value)
    restSnapshot(market.exchange2.value, market.symbol2.value,
      bidsMap2, asksMap2, v => { ob2.value = v }, v => { isLive2.value = v })
}

onMounted(() => {
  connectEx1()
  restTimer = setInterval(restPoll, REST_REFRESH_MS)
})
onUnmounted(() => {
  closeWs(ws1); closeWs(ws2)
  if (restTimer) clearInterval(restTimer)
})
</script>

<style scoped>
.ob-shell {
  display: flex; flex-direction: column;
  height: 100%; overflow: hidden;
  background: var(--bg); font-family: var(--mono, monospace);
}

/* ── Header ── */
.ob-header {
  display: flex; align-items: center; gap: 6px; flex-shrink: 0;
  padding: 5px 12px;
  background: rgba(14,22,40,0.9);
  border-bottom: 1px solid var(--border);
}
.ob-eyebrow { font-size: 9px; font-weight: 700; letter-spacing: .1em; color: var(--text-muted); white-space: nowrap; }
.ob-hsep    { width: 1px; height: 16px; background: rgba(255,255,255,0.08); flex-shrink: 0; }
.ob-pair    { font-size: 11px; font-weight: 700; color: #3b82f6; white-space: nowrap; }

.depth-picker { display: flex; gap: 2px; }
.depth-btn {
  padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 600;
  background: transparent; border: 1px solid rgba(255,255,255,0.08);
  color: rgba(255,255,255,0.3); cursor: pointer; transition: all .1s;
}
.depth-btn:hover  { border-color: rgba(59,130,246,0.4); color: rgba(255,255,255,0.7); }
.depth-btn.active { background: rgba(59,130,246,0.18); border-color: #3b82f6; color: #3b82f6; }

.ex2-toggle {
  padding: 2px 8px; border-radius: 4px; font-size: 10px; font-weight: 700;
  background: transparent; border: 1px solid rgba(167,139,250,0.25);
  color: rgba(167,139,250,0.5); cursor: pointer; transition: all .1s;
}
.ex2-toggle:hover  { border-color: rgba(167,139,250,0.5); color: #a78bfa; }
.ex2-toggle.active { background: rgba(167,139,250,0.15); border-color: #a78bfa; color: #a78bfa; }

.ob-live    { font-size: 9px; font-weight: 700; letter-spacing: .08em; color: rgba(255,255,255,0.25); }
.ob-live.live { color: #4ade80; }
.ob-latency { font-size: 9px; color: rgba(255,255,255,0.25); }

/* ── Metrics bar ── */
.ob-metrics {
  display: flex; align-items: center; gap: 0; flex-shrink: 0;
  padding: 4px 12px;
  background: rgba(10,16,28,0.8);
  border-bottom: 1px solid rgba(59,130,246,0.08);
  overflow-x: auto; scrollbar-width: none;
}
.ob-metrics::-webkit-scrollbar { display: none; }
.ob-metric { display: flex; flex-direction: column; gap: 1px; padding: 0 12px; border-right: 1px solid rgba(255,255,255,0.05); flex-shrink: 0; }
.ob-metric:last-child { border-right: none; }
.ob-metric-label { font-size: 8px; font-weight: 700; letter-spacing: .08em; color: rgba(255,255,255,0.3); }
.ob-metric-val   { font-size: 11px; font-weight: 700; color: #e0e0e0; }
.ob-metric-val.buy  { color: #4ade80; }
.ob-metric-val.sell { color: #f87171; }
.ob-pct { font-size: 9px; color: rgba(255,255,255,0.4); font-weight: 400; }

/* ── Content ── */
.ob-content {
  flex: 1; overflow: hidden;
  display: grid; grid-template-columns: 1fr;
}
.ob-content.dual { grid-template-columns: 1fr 1fr; gap: 1px; }

/* ── Book ── */
.ob-book {
  display: flex; flex-direction: column; overflow: hidden;
  position: relative;
}
.ob-book-label {
  padding: 3px 10px; font-size: 9px; font-weight: 700; letter-spacing: .08em;
  color: #3b82f6; background: rgba(59,130,246,0.06);
  border-bottom: 1px solid rgba(59,130,246,0.1); flex-shrink: 0;
}
.ob-book-ex2 .ob-book-label, .ex2-label { color: #a78bfa; background: rgba(167,139,250,0.06); border-bottom-color: rgba(167,139,250,0.1); }

.ob-asks, .ob-bids { flex: 1; overflow-y: auto; scrollbar-width: none; }
.ob-asks::-webkit-scrollbar, .ob-bids::-webkit-scrollbar { display: none; }
.ob-asks { display: flex; flex-direction: column; justify-content: flex-end; }

/* ── Rows ── */
.ob-row {
  display: grid; grid-template-columns: 1fr 1fr 1fr;
  align-items: center; position: relative;
  padding: 1px 10px; font-size: 10px; font-weight: 600;
  transition: background .08s;
  min-height: 18px;
}
.ob-row:hover { background: rgba(255,255,255,0.04); }

.ob-price { z-index: 1; }
.ob-size  { z-index: 1; text-align: center; color: rgba(255,255,255,0.6); }
.ob-total { z-index: 1; text-align: right;  color: rgba(255,255,255,0.35); font-weight: 400; }

.ask .ob-price, .ob-price.ask { color: #f87171; }
.bid .ob-price, .ob-price.bid { color: #4ade80; }

/* depth bar */
.ob-bar {
  position: absolute; top: 0; bottom: 0; right: 0;
  opacity: 0.12; border-radius: 1px 0 0 1px;
  transition: width .15s;
}
.ask-bar { background: #f87171; }
.bid-bar { background: #4ade80; }

/* ── Spread row ── */
.ob-spread-row {
  display: flex; align-items: center; justify-content: center; gap: 6px;
  padding: 3px 10px; flex-shrink: 0;
  background: rgba(255,255,255,0.025);
  border-top: 1px solid rgba(255,255,255,0.04);
  border-bottom: 1px solid rgba(255,255,255,0.04);
}
.ob-spread-val { font-size: 10px; font-weight: 700; color: #e0e0e0; }
.ob-spread-pct { font-size: 9px; color: rgba(255,255,255,0.4); }

/* ── Empty ── */
.ob-empty {
  display: flex; align-items: center; justify-content: center;
  height: 100%; color: rgba(255,255,255,0.25); font-size: 12px;
}
</style>
