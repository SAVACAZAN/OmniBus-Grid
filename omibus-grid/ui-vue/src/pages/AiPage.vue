<template>
  <div class="page-content">
    <div class="ai-heading">
      <div>
        <div class="eyebrow">AI RESEARCH 3 / LIVE CHART</div>
        <h1>Read the market<span>.</span></h1>
        <p style="color:var(--muted);margin:0">Your indicators, formations and triggers. One live research chart.</p>
      </div>
      <span class="ai-badge">RESEARCH 3 · PAPER</span>
    </div>

    <p class="ai-intro">Choose exactly what the model learns. BUY opens a simulated long; SELL opens a simulated short. TP closes either at its profit target.</p>

    <div class="ai-status" :class="{ error: noticeError }" role="status" aria-live="polite">{{ noticeText }}</div>

    <form ref="formEl" class="ai-controls" @submit.prevent="startLearning">
      <label>Binance symbol<input name="symbol" maxlength="20" required pattern="[A-Z0-9]{5,20}" autocomplete="off" @change="onFormChange"></label>
      <label>Candle timeframe
        <select name="interval" @change="onFormChange">
          <option v-for="iv in (catalog?.intervals ?? [])" :key="iv.id" :value="iv.id">{{ iv.label }}</option>
        </select>
      </label>
      <label>Maximum holding period (bars)<input name="horizon" type="number" value="4" min="1" max="24" required @change="onFormChange"></label>
      <label>Take profit after costs (%)<input name="take_profit_pct" type="number" value="1" min="0.1" max="50" step="0.1" required @change="onFormChange"></label>

      <details class="ai-picker">
        <summary>Simulation &amp; training settings</summary>
        <div class="ai-simulation-fields">
          <label>Initial history (bars)<input name="history_bars" type="number" value="3000" min="1500" max="10000" required @change="onFormChange"></label>
          <label>Fee per side (bps)<input name="fee_bps" type="number" value="10" min="0" max="100" step="0.1" required @change="onFormChange"></label>
          <label>Slippage per side (bps)<input name="slippage_bps" type="number" value="5" min="0" max="100" step="0.1" required @change="onFormChange"></label>
          <label>Entry score threshold<input name="threshold" type="number" value="0.60" min="0.50" max="0.95" step="0.01" required @change="onFormChange"></label>
        </div>
      </details>

      <details class="ai-picker">
        <summary>Indicators &amp; 20 chart formations · <span>{{ selectedInputs.length }} selected</span></summary>
        <fieldset class="ai-inputs">
          <legend>Learning inputs and chart studies</legend>
          <p>Only checked inputs enter the model. RSI only really means RSI only.</p>
          <div class="ai-presets">
            <button type="button" @click="applyPreset('rsi')">RSI only</button>
            <button type="button" @click="applyPreset('ema')">EMA only</button>
            <button type="button" @click="applyPreset('combined')">RSI + EMA + formations</button>
            <button type="button" @click="applyPreset('patterns')">All 20 formations</button>
            <button type="button" @click="applyPreset('all')">Select all</button>
            <button type="button" @click="applyPreset('none')">Clear</button>
          </div>
          <div v-if="catalog">
            <div v-for="[kind, title] in inputGroups" :key="kind" class="ai-input-group">
              <h3>{{ title }}</h3>
              <div class="ai-checkbox-grid">
                <label v-for="item in catalog.inputs.filter((i: any) => i.kind === kind)" :key="item.id" class="ai-checkbox">
                  <input type="checkbox" :value="item.id" v-model="selectedInputs" @change="onFormChange">
                  <span>{{ item.label }}</span>
                </label>
              </div>
            </div>
          </div>
          <p aria-live="polite">{{ selectedInputs.length }} inputs selected</p>
        </fieldset>
      </details>

      <div class="ai-actions">
        <button class="small-btn" type="submit" :disabled="busy || !initialized || (lastStatus?.running && !configChanged)">
          {{ lastStatus?.running ? 'Apply &amp; restart learning' : 'Start daily learning' }}
        </button>
        <button class="small-btn" type="button" :disabled="busy || !lastStatus?.running" @click="stopLearning">Stop</button>
      </div>
      <p class="ai-control-note">The chart follows your selections immediately. Apply &amp; restart to train a running model on new settings.</p>
    </form>

    <p class="ai-note">100 bps = 1%. Entry follows a one-bar delay. One paper position at a time. No exchange orders are sent.</p>

    <section class="ai-panel ai-live-panel">
      <h2>Live research chart</h2>
      <p :class="['ai-chart-status', { error: chartStatusError }]" role="status">{{ chartStatus }}</p>
      <p class="ai-note">{{ chartContext }}</p>
      <div ref="liveChartEl" class="ai-live-chart-host"></div>
      <div class="ai-live-formations">
        <p v-for="(item, i) in liveFormations" :key="i">
          {{ item.status === 'forming' ? 'FORMING' : 'CONFIRMED' }} · {{ item.name }} · {{ item.direction }}
          {{ item.bull_trigger !== null ? ' · break above ' + num(item.bull_trigger, 4) : '' }}
          {{ item.bear_trigger !== null ? ' · break below ' + num(item.bear_trigger, 4) : '' }}
          {{ item.confirmed_at ? ' · ' + fmtDate(item.confirmed_at) + ' UTC' : '' }}
        </p>
        <p v-if="!liveFormations.length">No matching formations in the recent chart window.</p>
      </div>
    </section>

    <div class="ai-metrics" ref="metricsEl"></div>

    <div class="ai-columns">
      <section class="ai-panel">
        <h2>Latest model signal</h2>
        <div v-if="latestSignal">
          <strong :class="['ai-signal', latestSignal.signal === 'BUY' ? 'long' : latestSignal.signal === 'SELL' ? 'short' : '']">
            {{ latestSignal.signal }}
          </strong>
          <p>Long score {{ num(latestSignal.p) }} · Short score {{ num(latestSignal.short_p) }}</p>
          <p>{{ fmtDate(latestSignal.issued_at) }} UTC · {{ latestSignal.reason }}</p>
        </div>
        <div v-else>Start learning to receive model signals.</div>
        <div>{{ positionText }}</div>
      </section>

      <section class="ai-panel">
        <h2>Evidence &amp; model quality</h2>
        <p>{{ evidenceSummary }}</p>
        <p>{{ lastDecision }}</p>
        <div v-if="evalTable.length" class="ai-eval-table">
          <table>
            <thead><tr><th>Evaluation</th><th>Brier ↓</th><th>Trades</th><th>Net return</th><th>Drawdown</th></tr></thead>
            <tbody>
              <tr v-for="row in evalTable" :key="row.label">
                <td>{{ row.label }}</td><td>{{ num(row.brier) }}</td><td>{{ row.trades }}</td>
                <td>{{ num(row.return_pct, 2) }}%</td><td>{{ num(row.drawdown_pct, 2) }}%</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <section class="ai-panel" style="margin-top:14px">
      <h2>Recent paper trades</h2>
      <table>
        <thead><tr><th>Entry</th><th>Side</th><th>Reason</th><th>Entry px</th><th>Exit px</th><th>Net</th></tr></thead>
        <tbody>
          <tr v-for="(t, i) in recentTrades" :key="i">
            <td>{{ fmtDate(t.entry_time) }}</td><td>{{ t.side }}</td><td>{{ t.reason || t.status }}</td>
            <td>{{ num(t.entry_price, 4) }}</td><td>{{ num(t.exit_price, 4) }}</td>
            <td>{{ t.net === undefined ? 'Pending' : num(t.net * 100, 2) + '%' }}</td>
          </tr>
          <tr v-if="!recentTrades.length"><td colspan="6">No paper positions yet.</td></tr>
        </tbody>
      </table>
    </section>

    <p class="ai-note" style="margin-top:12px">Research files: {{ dataDirectory }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onDeactivated, onUnmounted } from 'vue'
import { useInvoke } from '../composables/useTauri'

const invoke = useInvoke()

// ── Live chart (vanilla JS port — SVG renderer, no Vue needed) ─────────
function createLiveChart(root: HTMLElement) {
  const ns = 'http://www.w3.org/2000/svg'
  let snapshot: any, report: any, visible = 120, offset = 0
  const colors = ['#b9f05a','#69b9ff','#ffb768','#d39bff','#54ddc3','#ee91b7']
  const fmt4 = (n: any) => Number.isFinite(n) ? Number(n).toLocaleString('en-US', { maximumFractionDigits: 4 }) : '—'
  const clock = (t: number) => new Date(t).toISOString().slice(5, 16).replace('T', ' ') + ' UTC'

  const controls = document.createElement('div'); controls.className = 'ai-chart-tools'
  const body = document.createElement('div'); body.className = 'ai-live-canvas'
  const hover = document.createElement('p'); hover.className = 'ai-chart-readout'

  const choices: Record<string, boolean> = { formations: true, rules: true, trades: true }

  function btn(label: string, action: () => void) {
    const b = document.createElement('button'); b.type = 'button'; b.textContent = label
    b.addEventListener('click', action); controls.append(b)
  }
  btn('Zoom +', () => { visible = Math.max(40, Math.round(visible / 1.5)); render() })
  btn('Zoom −', () => { visible = Math.min(500, Math.round(visible * 1.5)); render() })
  btn('← Earlier', () => { offset = Math.min(Math.max(0, (snapshot?.candles.length ?? 600) - 100 - visible), offset + Math.floor(visible / 2)); render() })
  btn('Later →', () => { offset = Math.max(0, offset - Math.floor(visible / 2)); render() })
  btn('Live', () => { offset = 0; render() })

  for (const [key, label] of [['formations','Formations'],['rules','Rule triggers'],['trades','BUY / SELL / TP']] as [string,string][]) {
    const holder = document.createElement('label'), input = document.createElement('input')
    input.type = 'checkbox'; input.checked = true
    input.addEventListener('change', () => { choices[key] = input.checked; render() })
    holder.append(input, document.createTextNode(label)); controls.append(holder)
  }

  root.replaceChildren(controls, hover, body)

  function svgEl(height: number, ariaLabel: string) {
    const e = document.createElementNS(ns, 'svg')
    e.setAttribute('viewBox', `0 0 1200 ${height}`)
    e.setAttribute('role', 'img'); e.setAttribute('aria-label', ariaLabel)
    return e
  }
  function add(parent: Element, tag: string, attrs: Record<string, string | number> = {}, text?: string) {
    const e = document.createElementNS(ns, tag)
    for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v))
    if (text !== undefined) e.textContent = text
    parent.append(e); return e
  }
  function lineSeries(parent: Element, series: any, scaleY: (v: number) => number, color: string) {
    let points: string[] = []
    for (let i = 0; i < series.values.length; i++) {
      const v = series.values[i]
      if (Number.isFinite(v)) { points.push(`${0},${scaleY(v)}`) } // placeholder x — handled in render
      else if (points.length) { add(parent, 'polyline', { points: points.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 }); points = [] }
    }
    if (points.length) add(parent, 'polyline', { points: points.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 })
  }

  function render() {
    if (!snapshot?.candles.length) return
    body.replaceChildren()
    const all = snapshot.candles
    const end = Math.max(101, all.length - offset), start = Math.max(100, end - visible)
    const data = all.slice(start, end)
    if (!data.length) return
    const first = data[0][0], last = data.at(-1)[0], step = all[1][0] - all[0][0]
    const x = (time: number) => 24 + (time - first) / Math.max(step, last - first) * 1088
    const chartEl = svgEl(390, `Live ${snapshot.config.symbol} ${snapshot.config.interval} chart`)
    const overlays = snapshot.panels.filter((p: any) => p.overlay)
    const oscillators = snapshot.panels.filter((p: any) => !p.overlay)
    const bounds: number[] = data.flatMap((c: number[]) => [c[2], c[3]])
    for (const p of overlays) for (const s of p.series) bounds.push(...s.values.slice(start, end).filter(Number.isFinite))
    let low = Math.min(...bounds), high = Math.max(...bounds)
    const padding = (high - low || high * .01) * .12; low -= padding; high += padding
    const y = (v: number) => 330 - (v - low) / (high - low) * 290
    const defs = add(chartEl, 'defs'); const clip = add(defs, 'clipPath', { id: 'ai-live-price-clip' })
    add(clip, 'rect', { x: 20, y: 22, width: 1100, height: 315 })
    const plot = add(chartEl, 'g', { 'clip-path': 'url(#ai-live-price-clip)' })
    for (let i = 0; i < 5; i++) {
      const p = low + (high - low) * i / 4
      add(chartEl, 'line', { x1: 20, x2: 1115, y1: y(p), y2: y(p), stroke: '#28343d' })
      add(chartEl, 'text', { x: 1122, y: y(p) + 4, fill: '#9daebb', 'font-size': 11 }, fmt4(p))
    }
    const w = Math.max(1.5, Math.min(11, 800 / data.length))
    for (const c of data) {
      const color = c[4] >= c[1] ? '#65d4a0' : '#ef8585'
      add(plot, 'line', { x1: x(c[0]), x2: x(c[0]), y1: y(c[2]), y2: y(c[3]), stroke: color })
      add(plot, 'rect', { x: x(c[0]) - w / 2, y: Math.min(y(c[1]), y(c[4])), width: w, height: Math.max(1, Math.abs(y(c[1]) - y(c[4]))), fill: color, opacity: (snapshot.provisional && c === all.at(-1)) ? 0.6 : 1 })
    }
    let ci = 0; const legend: [string, string][] = []
    for (const p of overlays) for (const s of p.series) {
      const color = colors[ci++ % colors.length]
      let pts: string[] = []
      for (let i = start; i < end; i++) {
        const v = s.values[i]
        if (Number.isFinite(v)) pts.push(`${x(all[i][0])},${y(v)}`)
        else if (pts.length) { add(plot, 'polyline', { points: pts.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 }); pts = [] }
      }
      if (pts.length) add(plot, 'polyline', { points: pts.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 })
      legend.push([s.name, color])
    }

    function marker(time: number, price: number, label: string, color: string) {
      const candle = data.find((c: any) => c[0] <= time && c[6] >= time)
      if (!candle || !Number.isFinite(price)) return
      const cx = x(candle[0]), cy = y(price)
      add(plot, 'circle', { cx, cy, r: 4, fill: color, stroke: '#0c1218' })
      const above = /SELL|TP|EXIT|↓/.test(label)
      add(plot, 'text', { x: cx, y: Math.max(36, Math.min(323, cy + (above ? -12 : 20))), fill: color, 'text-anchor': 'middle', 'font-size': 11, 'font-weight': 'bold' }, label)
    }

    const formations = choices.formations ? [
      ...snapshot.confirmed.filter((p: any) => p.confirmed_at >= first && p.confirmed_at < last + step).slice(-6),
      ...snapshot.forming.slice(-6)
    ] : []
    for (const pattern of formations) {
      const provisional = pattern.status === 'forming'
      const color = provisional ? '#eabb69' : pattern.direction === 'bullish' ? '#81dabe' : '#afaeff'
      add(plot, 'polyline', { points: pattern.points.map((p: any) => `${x(p.time)},${y(p.price)}`).join(' '), fill: 'none', stroke: color, 'stroke-width': 2, 'stroke-dasharray': provisional ? '5 5' : 'none' })
      for (const boundary of pattern.lines ?? []) {
        const [a, b] = boundary.points
        add(plot, 'line', { x1: x(a.time), x2: x(b.time), y1: y(a.price), y2: y(b.price), stroke: color, 'stroke-dasharray': '5 4', 'stroke-width': 1.4 })
      }
      const anchor = pattern.points.at(-1)
      add(plot, 'text', { x: x(anchor.time), y: Math.max(38, Math.min(320, y(anchor.price) - 14)), fill: color, 'font-size': 11, 'text-anchor': 'middle' }, pattern.name + (provisional ? ' · forming' : ''))
      if (!provisional) marker(pattern.confirmed_at, pattern.close, pattern.direction === 'bullish' ? 'BREAK ↑' : 'BREAK ↓', color)
    }
    if (choices.rules) for (const m of snapshot.custom_triggers) marker(m.time, m.price, m.kind, '#ffc875')
    if (choices.trades && report) {
      for (const trade of report.trades) {
        if (trade.entry_price !== undefined) marker(trade.entry_time, trade.entry_price, trade.side === 'LONG' ? 'BUY' : 'SELL', trade.side === 'LONG' ? '#81edac' : '#ff9898')
        if (trade.status === 'CLOSED') marker(trade.exit_time, trade.exit_price, trade.reason, trade.reason === 'TP' ? '#66d8ff' : '#edb66e')
      }
      if (report.position?.target_price) {
        const p = report.position.target_price
        add(plot, 'line', { x1: 24, x2: 1112, y1: y(p), y2: y(p), stroke: '#66d8ff', 'stroke-dasharray': '7 5' })
        add(plot, 'text', { x: 35, y: y(p) - 5, fill: '#66d8ff', 'font-size': 12 }, `TP ${report.position.side} ${fmt4(p)}`)
      }
    }
    for (let i = 0; i < 5; i++) {
      const c = data[Math.floor((data.length - 1) * i / 4)]
      add(chartEl, 'text', { x: x(c[0]), y: 367, fill: '#9daebb', 'font-size': 10, 'text-anchor': i === 0 ? 'start' : i === 4 ? 'end' : 'middle' }, clock(c[0]))
    }
    const lastBar = all.at(-1)
    hover.textContent = `${snapshot.config.symbol} · ${snapshot.config.interval} · Last ${fmt4(lastBar[4])} · ${offset ? 'Historical view' : 'Latest candles'}`
    const cross = add(chartEl, 'line', { y1: 25, y2: 335, x1: 0, x2: 0, stroke: '#cad5de', 'stroke-dasharray': '2 4', visibility: 'hidden' })
    chartEl.addEventListener('pointermove', event => {
      const rect = chartEl.getBoundingClientRect()
      const coord = (event.clientX - rect.left) / rect.width * 1200
      const index = Math.max(0, Math.min(data.length - 1, Math.round((coord - 24) / 1088 * (data.length - 1))))
      const c = data[index]
      cross.setAttribute('x1', String(x(c[0]))); cross.setAttribute('x2', String(x(c[0]))); cross.setAttribute('visibility', 'visible')
      hover.textContent = `${clock(c[0])} · O ${fmt4(c[1])} H ${fmt4(c[2])} L ${fmt4(c[3])} C ${fmt4(c[4])}`
    })
    chartEl.addEventListener('pointerleave', () => cross.setAttribute('visibility', 'hidden'))
    body.append(chartEl)
    if (legend.length) {
      const labels = document.createElement('div'); labels.className = 'ai-chart-legend'
      for (const [name, color] of legend) { const lbl = document.createElement('span'); lbl.textContent = name; lbl.style.color = color; labels.append(lbl) }
      body.append(labels)
    }
    for (const [pi, panel] of oscillators.entries()) {
      const pane = svgEl(150, panel.label + ' indicator panel')
      const vals = panel.series.flatMap((s: any) => s.values.slice(start, end).filter(Number.isFinite))
      const rules = [...snapshot.config.triggers.buy, ...snapshot.config.triggers.sell].filter((r: any) => r.right === 'number' && panel.series.some((s: any) => s.name === r.left))
      vals.push(...rules.map((r: any) => r.value))
      if (!vals.length) continue
      let min = Math.min(...vals), max = Math.max(...vals)
      if (['rsi','stochastic','mfi','aroon','adx'].includes(panel.id)) { min = Math.min(min, 0); max = Math.max(max, 100) }
      const span = max - min || 1; min -= span * .1; max += span * .1
      const py = (v: number) => 115 - (v - min) / (max - min) * 80
      add(pane, 'text', { x: 24, y: 19, fill: '#c8d8e3', 'font-size': 12 }, panel.label)
      for (let i = 0; i < 3; i++) {
        const p = min + (max - min) * i / 2
        add(pane, 'line', { x1: 20, x2: 1115, y1: py(p), y2: py(p), stroke: '#26323a' })
        add(pane, 'text', { x: 1122, y: py(p) + 3, fill: '#92a3b1', 'font-size': 10 }, fmt4(p))
      }
      for (const [i, s] of panel.series.entries()) {
        const color = colors[(pi + i) % colors.length]
        let pts: string[] = []
        for (let k = start; k < end; k++) {
          const v = s.values[k]; const ax = x(all[k][0])
          if (Number.isFinite(v)) pts.push(`${ax},${py(v)}`)
          else if (pts.length) { add(pane, 'polyline', { points: pts.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 }); pts = [] }
        }
        if (pts.length) add(pane, 'polyline', { points: pts.join(' '), fill: 'none', stroke: color, 'stroke-width': 1.6 })
        add(pane, 'text', { x: 230 + i * 220, y: 19, fill: color, 'font-size': 11 }, s.name + ' ' + fmt4(s.values[end - 1]))
      }
      for (const rule of rules) {
        add(pane, 'line', { x1: 24, x2: 1112, y1: py(rule.value), y2: py(rule.value), stroke: '#ffc875', 'stroke-dasharray': '5 4' })
        add(pane, 'text', { x: 35, y: py(rule.value) - 4, fill: '#ffc875', 'font-size': 10 }, 'Rule level ' + rule.value)
      }
      body.append(pane)
    }
  }
  return {
    update(next: any, modelReport: any) { snapshot = next; report = modelReport; render() },
    dispose() { root.replaceChildren() },
  }
}

// ── State ──────────────────────────────────────────────────────────────
const formEl = ref<HTMLFormElement | null>(null)
const liveChartEl = ref<HTMLElement | null>(null)
const metricsEl = ref<HTMLElement | null>(null)

const catalog = ref<any>(null)
const selectedInputs = ref<string[]>([])
const initialized = ref(false)
const busy = ref(false)
const noticeText = ref('Loading worker status…')
const noticeError = ref(false)
const chartStatus = ref('Loading chart settings…')
const chartStatusError = ref(false)
const chartContext = ref('')
const liveFormations = ref<any[]>([])
const latestSignal = ref<any>(null)
const positionText = ref('No pending or open paper position.')
const evidenceSummary = ref('')
const lastDecision = ref('')
const evalTable = ref<any[]>([])
const recentTrades = ref<any[]>([])
const dataDirectory = ref('')
const configChanged = ref(false)

const lastStatus = ref<any>(null)
const lastSnapshot = ref<any>(null)
const lastReport = ref<any>(null)

const inputGroups: [string, string][] = [
  ['indicator', 'Indicators'],
  ['pattern', 'Chart formations'],
  ['price', 'Additional price data'],
]

let liveChart: ReturnType<typeof createLiveChart> | null = null
let pollTimer: ReturnType<typeof setTimeout> | null = null
let chartTimer: ReturnType<typeof setTimeout> | null = null
let chartRevision = 0
let chartBusy = false
let pollGeneration = 0
let active = false

// ── Helpers ────────────────────────────────────────────────────────────
const num = (v: any, d = 3) => typeof v === 'number' ? v.toFixed(d) : '—'
const fmtDate = (v: any) => v ? new Date(v).toISOString().replace('T', ' ').slice(0, 19) : '—'

function getConfig() {
  if (!formEl.value) throw new Error('Form not ready')
  const data = new FormData(formEl.value)
  const cfg: any = Object.fromEntries(data)
  cfg.inputs = selectedInputs.value.slice().sort()
  if (!cfg.inputs.length) throw new Error('Select at least one indicator or pattern to learn.')
  cfg.symbol = (cfg.symbol ?? '').trim().toUpperCase()
  cfg.triggers = { buy: [], sell: [] }
  for (const k of ['horizon','history_bars','fee_bps','slippage_bps','threshold','take_profit_pct'])
    cfg[k] = Number(cfg[k])
  return cfg
}

function sameProfile(a: any, b: any) {
  if (!a || !b) return false
  return Object.keys(a).length === Object.keys(b).length && Object.keys(a).every(key => {
    if (key === 'inputs') return JSON.stringify([...a.inputs].sort()) === JSON.stringify([...b.inputs].sort())
    if (key === 'triggers') return JSON.stringify(a.triggers) === JSON.stringify(b.triggers)
    return a[key] === b[key]
  })
}

function applyPreset(preset: string) {
  if (!catalog.value) return
  const all = catalog.value.inputs.map((i: any) => i.id)
  const presets: Record<string, string[]> = {
    rsi: ['rsi'], ema: ['ema'],
    combined: ['rsi','ema','head_shoulders','inverse_head_shoulders','double_top','double_bottom'],
    patterns: catalog.value.inputs.filter((i: any) => i.kind === 'pattern').map((i: any) => i.id),
    all, none: [],
  }
  selectedInputs.value = presets[preset] ?? []
  onFormChange()
}

function onFormChange() {
  try { configChanged.value = !sameProfile(getConfig(), lastStatus.value?.config) } catch { configChanged.value = true }
  scheduleChart()
}

// ── Chart refresh ──────────────────────────────────────────────────────
function scheduleChart() {
  chartRevision++
  if (chartTimer) clearTimeout(chartTimer)
  if (active && initialized.value) chartTimer = setTimeout(refreshChart, 350)
}

async function refreshChart() {
  if (!active) return
  if (chartBusy) { chartTimer = setTimeout(refreshChart, 500); return }
  let cfg: any
  try { cfg = { ...getConfig() }; delete cfg.history_bars; delete cfg.fee_bps; delete cfg.slippage_bps; delete cfg.threshold; delete cfg.horizon; delete cfg.take_profit_pct } catch (e) { chartStatus.value = String(e); return }
  const rev = chartRevision; chartBusy = true; let delay = 5000
  chartStatusError.value = false
  chartStatus.value = `Updating ${cfg.symbol} ${cfg.interval}…${lastSnapshot.value ? ' Previous snapshot remains visible.' : ''}`
  try {
    const snapshot = await invoke<any>('ai_chart_snapshot', { config: cfg })
    if (rev !== chartRevision) return
    if (snapshot.error) { delay = Math.max(15000, Math.min(3600000, (snapshot.retry_seconds ?? 15) * 1000)); throw new Error(snapshot.error) }
    lastSnapshot.value = snapshot
    displayChart()
    chartStatusError.value = false
    chartStatus.value = `${snapshot.preview ? 'SAVED TEST SNAPSHOT' : snapshot.delayed ? 'DELAYED DATA' : 'LIVE · 5-second refresh'} · ${cfg.symbol} ${cfg.interval} · updated ${fmtDate(snapshot.updated_at)} UTC`
  } catch (e) {
    if (rev === chartRevision) { chartStatusError.value = true; chartStatus.value = `Chart unavailable: ${e}. ${lastSnapshot.value ? 'Last snapshot: ' + fmtDate(lastSnapshot.value.updated_at) + ' UTC.' : ''}`; delay = Math.max(delay, 15000) }
  } finally {
    chartBusy = false
    if (active) chartTimer = setTimeout(refreshChart, rev === chartRevision ? delay : 0)
  }
}

function displayChart() {
  if (!lastSnapshot.value || !liveChart) return
  const matching = lastReport.value && sameProfile(
    { symbol: lastSnapshot.value.config.symbol, interval: lastSnapshot.value.config.interval, inputs: lastSnapshot.value.config.inputs, triggers: lastSnapshot.value.config.triggers },
    { symbol: lastReport.value.config.symbol, interval: lastReport.value.config.interval, inputs: lastReport.value.config.inputs, triggers: lastReport.value.config.triggers }
  )
  liveChart.update(lastSnapshot.value, matching ? lastReport.value : null)
  chartContext.value = matching
    ? `Paper markers: saved model · Custom BUY rules: ${lastSnapshot.value.rule_status.buy ? 'met' : 'not met'}; SELL rules: ${lastSnapshot.value.rule_status.sell ? 'met' : 'not met'}.`
    : 'Chart preview follows selected inputs. Start learning to see BUY/SELL/TP markers.'
  const items = [...lastSnapshot.value.forming, ...lastSnapshot.value.confirmed.slice(-5).reverse()]
  liveFormations.value = items
}

// ── Status polling ─────────────────────────────────────────────────────
function renderStatus(status: any) {
  lastStatus.value = status
  if (!initialized.value) {
    catalog.value = status.catalog
    selectedInputs.value = (status.config.inputs ?? []).slice()
    // Apply saved config values to form
    if (formEl.value) {
      for (const [k, v] of Object.entries(status.config)) {
        if (['inputs','triggers'].includes(k)) continue
        const el = formEl.value.elements.namedItem(k) as HTMLInputElement | HTMLSelectElement | null
        if (el) el.value = String(v)
      }
    }
    initialized.value = true
    scheduleChart()
  }

  const worker = status.worker ?? {}
  const error = status.error || (status.running && worker.error)
  const stale = status.running && worker.heartbeat && Date.now() - worker.heartbeat > 420000
  noticeError.value = Boolean(error || stale)
  noticeText.value = error
    ? `Worker needs attention: ${error}`
    : stale ? 'Worker heartbeat is stale. Stop and restart if it does not recover.'
    : status.running
      ? ({ fetching: 'Downloading closed public candles…', evaluating: 'Evaluating outcomes and training when due…', waiting: 'Running · checks every 15 seconds · learns every 24 hours' }[worker.phase as string] ?? 'Starting the AI worker…')
      : 'Learning stopped. The live chart still works. Select inputs, then start learning when ready.'

  dataDirectory.value = status.data_directory ?? ''

  const r = worker.report
  if (!r || r.schema !== 3 || !sameProfile(status.config, r.config)) {
    if (lastReport.value) { lastReport.value = null; displayChart() }
    latestSignal.value = null; positionText.value = 'No pending or open paper position.'
    evidenceSummary.value = ''; lastDecision.value = ''; evalTable.value = []; recentTrades.value = []
    return
  }

  if (lastReport.value?.updated_at !== r.updated_at) { lastReport.value = r; displayChart() }
  else lastReport.value = r

  dataDirectory.value = r.profile ?? status.data_directory

  latestSignal.value = r.signals[0] ?? null

  const pos = r.position
  positionText.value = pos
    ? `${pos.status} ${pos.side} · entry ${fmtDate(pos.entry_time)} UTC · TP ${num(pos.target_price, 4)} · time exit ${fmtDate(pos.deadline)} UTC`
    : 'No pending or open paper position.'
  const lastExit = r.trades.find((t: any) => t.status === 'CLOSED')
  if (lastExit) positionText.value += ` | Latest exit: ${lastExit.reason} · ${num(lastExit.net * 100, 2)}% after costs`

  evidenceSummary.value = `Model v${r.champion_version} · candidate ${r.challenger_generation} · ${r.evidence_count} / ${r.evidence_required} forward outcomes.`
  lastDecision.value = r.last_decision ?? ''
  evalTable.value = [
    { label: 'Historical validation', ...r.bootstrap.validation },
    { label: 'Historical holdout', ...r.bootstrap.holdout },
    { label: 'Fixed base-rate baseline', ...r.bootstrap.holdout_baseline },
    { label: 'Forward paper record', ...r.forward },
  ]
  recentTrades.value = r.trades.slice(0, 20)
}

async function refresh() {
  try { renderStatus(await invoke<any>('ai_status')) }
  catch (e) { if (active) { noticeError.value = true; noticeText.value = String(e) } }
}

async function poll(gen: number) {
  await refresh()
  if (active && gen === pollGeneration) pollTimer = setTimeout(() => poll(gen), 5000)
}

// ── Actions ────────────────────────────────────────────────────────────
async function startLearning() {
  if (busy.value) return
  busy.value = true
  try {
    const cfg = getConfig()
    if (lastStatus.value?.running) await invoke('ai_stop')
    await invoke('ai_start', { config: cfg })
    busy.value = false
    await refresh()
  } catch (e) {
    busy.value = false; await refresh()
    noticeError.value = true; noticeText.value = String(e)
  }
}

async function stopLearning() {
  if (busy.value) return
  busy.value = true
  try { await invoke('ai_stop'); busy.value = false; await refresh() }
  catch (e) { busy.value = false; await refresh(); noticeError.value = true; noticeText.value = String(e) }
}

// ── Lifecycle ──────────────────────────────────────────────────────────
onMounted(() => {
  if (liveChartEl.value) liveChart = createLiveChart(liveChartEl.value)
})

onActivated(() => {
  if (!active) { active = true; void poll(++pollGeneration); scheduleChart() }
})

onDeactivated(() => {
  active = false; pollGeneration++; chartRevision++
  if (pollTimer) clearTimeout(pollTimer)
  if (chartTimer) clearTimeout(chartTimer)
})

onUnmounted(() => {
  active = false; pollGeneration++; chartRevision++
  if (pollTimer) clearTimeout(pollTimer)
  if (chartTimer) clearTimeout(chartTimer)
  liveChart?.dispose()
})
</script>

<style scoped>
.ai-heading { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; }
.ai-badge { font: 10px var(--mono); letter-spacing: 1px; color: var(--lime); border: 1px solid #3d5430; background: #1a2b18; border-radius: 6px; padding: 5px 10px; white-space: nowrap; margin-top: 8px; }
.ai-intro { color: var(--muted); font-size: 13px; margin: 0 0 14px; line-height: 1.5; }
.ai-status { padding: 10px 14px; border-radius: 7px; border: 1px solid #3b5030; background: #172318; color: #cde5c0; font-size: 12px; margin-bottom: 16px; }
.ai-status.error { border-color: #6b3d38; background: #281a19; color: #ffd0c8; }
.ai-controls { display: flex; flex-direction: column; gap: 10px; margin-bottom: 18px; }
.ai-controls label { color: var(--muted); font-size: 11px; font-weight: 600; }
.ai-simulation-fields { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; padding: 12px 0 4px; }
.ai-picker { border: 1px solid var(--line); border-radius: 8px; padding: 10px 14px; }
.ai-picker summary { cursor: pointer; font-size: 12px; color: #b0c4b8; user-select: none; }
.ai-inputs { border: none; padding: 0; margin: 0; }
.ai-inputs legend { color: var(--lime); font-size: 11px; letter-spacing: 1px; }
.ai-presets { display: flex; flex-wrap: wrap; gap: 6px; margin: 8px 0 12px; }
.ai-presets button { font-size: 11px; padding: 4px 9px; background: #1e2c22; color: #b0cfa8; border: 1px solid #3a5040; border-radius: 5px; }
.ai-input-group h3 { font-size: 11px; color: #8a9d95; letter-spacing: .5px; margin: 12px 0 6px; font-weight: 600; }
.ai-checkbox-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 4px; }
.ai-checkbox { display: flex; align-items: center; gap: 6px; font-size: 12px; color: #c4d8ce; cursor: pointer; padding: 3px 0; }
.ai-checkbox input { width: auto; height: auto; margin: 0; flex-shrink: 0; }
.ai-actions { display: flex; gap: 8px; }
.ai-control-note { color: var(--muted); font-size: 11px; margin: 4px 0 0; }
.ai-note { color: var(--muted); font-size: 11px; line-height: 1.5; margin: 8px 0; }
.ai-panel { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 18px 20px; margin-bottom: 14px; }
.ai-panel h2 { margin: 0 0 12px; font-size: 15px; font-weight: 650; }
.ai-live-panel { }
.ai-chart-status { font-size: 11px; color: var(--muted); margin: 0 0 6px; }
.ai-chart-status.error { color: var(--red); }
.ai-live-chart-host :deep(.ai-chart-tools) { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.ai-live-chart-host :deep(.ai-chart-tools button), .ai-live-chart-host :deep(.ai-chart-tools label) { font-size: 11px; padding: 3px 8px; background: #1c2820; color: #b0cfb8; border: 1px solid #3a5040; border-radius: 5px; cursor: pointer; display: flex; align-items: center; gap: 5px; }
.ai-live-chart-host :deep(.ai-chart-readout) { font: 11px var(--mono); color: #9daebb; margin: 4px 0 6px; min-height: 16px; }
.ai-live-chart-host :deep(.ai-live-canvas svg) { width: 100%; display: block; }
.ai-live-chart-host :deep(.ai-chart-legend) { display: flex; flex-wrap: wrap; gap: 12px; margin-top: 6px; font: 11px var(--mono); }
.ai-live-formations { margin-top: 10px; font-size: 11px; color: #a0b8b0; line-height: 1.6; }
.ai-signal { display: inline-block; font: 700 18px var(--mono); margin-bottom: 6px; }
.ai-signal.long { color: var(--green); }
.ai-signal.short { color: var(--red); }
.ai-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.ai-eval-table { overflow-x: auto; margin-top: 10px; }
table { border-collapse: collapse; width: 100%; font: 12px var(--mono); }
th { background: #182128; color: #7e9198; text-align: left; font-size: 9px; letter-spacing: .7px; padding: 9px 8px; }
td { border-top: 1px solid #263038; padding: 8px; color: #d6e0e3; }
tr:hover td { background: #1a2529; }
@media (max-width: 900px) { .ai-columns { grid-template-columns: 1fr; } }
</style>
