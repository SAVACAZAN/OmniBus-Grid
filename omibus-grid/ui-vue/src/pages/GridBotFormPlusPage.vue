<template>
  <div class="gbfp-root">

  <!-- RIGHT PANEL: OrderBook + Chart -->
  <div class="gbfp-right">

    <!-- TradingView Chart -->
    <div class="gbfp-chart-wrap">
      <iframe :src="chartUrl" frameborder="0" allowtransparency="true" scrolling="no"
        style="width:100%;height:100%;border:none;"></iframe>
    </div>

    <!-- OrderBook -->
    <div class="gbfp-ob">
      <div class="gbfp-ob-header">
        <span>📖 ORDER BOOK</span>
        <span style="font-size:9px;color:#888;">{{ currentExchange }} / {{ currentSymbol }}</span>
        <span v-if="obSpread" style="font-size:9px;color:#facc15;">spread {{ obSpread }}</span>
      </div>
      <div class="gbfp-ob-body">
        <div class="gbfp-ob-col">
          <div class="gbfp-ob-col-hdr" style="color:#f87171;">ASKS</div>
          <div v-for="(row,i) in asksDisplay" :key="'a'+i" class="gbfp-ob-row ask-row"
            :style="{background:`rgba(248,113,113,${row.depth*0.18})`}">
            <span class="ob-price" style="color:#f87171;">{{ row.price }}</span>
            <span class="ob-qty">{{ row.qty }}</span>
          </div>
        </div>
        <div class="gbfp-ob-mid">
          <div class="ob-mid-bid">{{ bestBid?.toFixed(5) || '--' }}</div>
          <div style="font-size:8px;color:#888;margin:2px 0;">MID</div>
          <div class="ob-mid-ask">{{ bestAsk?.toFixed(5) || '--' }}</div>
        </div>
        <div class="gbfp-ob-col">
          <div class="gbfp-ob-col-hdr" style="color:#4ade80;">BIDS</div>
          <div v-for="(row,i) in bidsDisplay" :key="'b'+i" class="gbfp-ob-row bid-row"
            :style="{background:`rgba(74,222,128,${row.depth*0.18})`}">
            <span class="ob-price" style="color:#4ade80;">{{ row.price }}</span>
            <span class="ob-qty">{{ row.qty }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- LEFT PANEL: Form -->
  <div class="gridbot-form-container">
    <!-- Compact Header -->
    <div class="form-header">
      <span class="header-icon">⚙️</span>
      <span class="header-title">Grid Bot Config</span>
      <span style="margin-left:auto; font-size:10px; color:#888;">{{ currentExchange }} / {{ currentSymbol }}</span>
    </div>

    <!-- Current Prices Display -->
    <div class="prices-display">
      <div class="price-item bid">
        <span class="price-label">BID</span>
        <span class="price-value">{{ bestBid || '-' }}</span>
      </div>
      <div class="price-item ask">
        <span class="price-label">ASK</span>
        <span class="price-value">{{ bestAsk || '-' }}</span>
      </div>
    </div>

    <!-- PYRAMID GRID - TOP FIRST -->
    <div class="config-section pyramid-section">
      <div class="section-header" @click="showPyramidConfig = !showPyramidConfig">
        <span>🔺 PYRAMID GRID</span>
        <span class="collapse-icon">{{ showPyramidConfig ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showPyramidConfig" class="section-content pyramid-content">
        <div style="margin-bottom:12px;">
          <label class="custom-checkbox">
            <input type="checkbox" v-model="pyramidEnabled" />
            ✓ Enable Pyramid Grid
          </label>
        </div>
        <div v-if="pyramidEnabled" style="display:grid; gap:10px;">
          <div class="form-row">
            <label>Type</label>
            <select v-model="pyramidType" class="custom-input">
              <option v-for="o in pyramidTypeOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
            </select>
          </div>
          <div class="form-row">
            <label>Multiplier</label>
            <input type="number" v-model.number="pyramidMultiplier" step="0.1" min="1" max="5"
              class="custom-input" placeholder="1.5" :disabled="pyramidType === 'linear'" />
          </div>
        </div>
        <div v-else class="pyramid-disabled-hint">Enable to configure pyramid strategy</div>
      </div>
    </div>

    <!-- Basic Configuration -->
    <div class="config-section">
      <div class="section-header" @click="showBasicConfig = !showBasicConfig">
        <span>📋 Basic</span>
        <span class="collapse-icon">{{ showBasicConfig ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showBasicConfig" class="section-content">
        <div class="form-row">
          <label>Bot Name</label>
          <input v-model="name" class="custom-input" placeholder="Bot name" />
        </div>
        <div class="form-row">
          <label>Lower Price</label>
          <div class="input-wrapper">
            <input v-model="lowerPrice" class="custom-input" :placeholder="quote" style="padding-right:36px;" />
            <span class="suffix-label">{{ quote }}</span>
          </div>
        </div>
        <div class="form-row">
          <label>Upper Price</label>
          <div class="input-wrapper">
            <input v-model="upperPrice" class="custom-input" :placeholder="quote" style="padding-right:36px;" />
            <span class="suffix-label">{{ quote }}</span>
          </div>
        </div>
        <div class="form-row">
          <label>Amount Type</label>
          <select v-model="amountType" class="custom-input">
            <option v-for="o in amountTypeOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
          </select>
        </div>
        <div class="form-row">
          <label>Amount Unit</label>
          <div class="amount-unit-toggle">
            <button type="button" class="unit-btn" :class="{ active: amountUnit === 'base' }" @click="amountUnit = 'base'">
              💰 {{ base }} (Base)
            </button>
            <button type="button" class="unit-btn" :class="{ active: amountUnit === 'quote' }" @click="amountUnit = 'quote'">
              💵 {{ quote }} (Quote)
            </button>
          </div>
        </div>
        <div class="form-row">
          <label>Amount</label>
          <div class="input-wrapper">
            <input v-model="amount" class="custom-input" :placeholder="amountUnit === 'base' ? base : quote" style="padding-right:40px;" />
            <span class="suffix-label">{{ amountUnit === 'base' ? base : quote }}</span>
          </div>
          <div v-if="amountAnalysis" class="amount-analysis" :class="{ 'has-warning': amountAnalysis.warning }">
            <div class="row">
              <span>Total:</span>
              <span class="mono">{{ amountAnalysis.totalBase.toFixed(4) }} {{ base }} / {{ amountAnalysis.totalQuote.toFixed(2) }} {{ quote }}</span>
            </div>
            <div class="row">
              <span>Per order:</span>
              <span class="mono">{{ amountAnalysis.perOrderBase.toFixed(4) }} {{ base }} / {{ amountAnalysis.perOrderQuote.toFixed(2) }} {{ quote }}</span>
            </div>
            <div v-if="currentLimits" class="row hint">
              <span>Min/order:</span>
              <span class="mono">{{ currentLimits.minBase }} {{ base }} / {{ currentLimits.minQuote }} {{ quote }}</span>
            </div>
            <div v-if="amountAnalysis.warning" class="warning-line">⚠ {{ amountAnalysis.warning }}</div>
            <div v-if="amountAnalysis.minRequired" class="warning-line fix">👉 {{ amountAnalysis.minRequired }}</div>
          </div>
        </div>
        <div class="form-row">
          <label>Nr of Grids</label>
          <input v-model="nrOfGrids" class="custom-input" placeholder="10" type="number" />
        </div>
        <div class="form-row">
          <label>Orders Side</label>
          <select v-model="ordersSide" class="custom-input">
            <option v-for="o in ordersSideOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
          </select>
        </div>
      </div>
    </div>

    <!-- Advanced Configuration -->
    <div class="config-section">
      <div class="section-header" @click="showAdvancedConfig = !showAdvancedConfig">
        <span>🔧 Advanced (Deviation, Incremental)</span>
        <span class="collapse-icon">{{ showAdvancedConfig ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showAdvancedConfig" class="section-content">
        <div class="form-row">
          <label>Inc % Buy</label>
          <div class="input-wrapper">
            <input v-model="incrementalPercentAmountBuy" class="custom-input" placeholder="1.0" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Inc % Sell</label>
          <div class="input-wrapper">
            <input v-model="incrementalPercentAmountSell" class="custom-input" placeholder="1.0" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Dev Price Buy</label>
          <div class="input-wrapper">
            <input v-model="deviationPriceBuy" class="custom-input" placeholder="1.0" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Dev Price Sell</label>
          <div class="input-wrapper">
            <input v-model="deviationPriceSell" class="custom-input" placeholder="1.0" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Dev Amt Buy</label>
          <div class="input-wrapper">
            <input v-model="deviationAmountBuy" class="custom-input" placeholder="0.9" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Dev Amt Sell</label>
          <div class="input-wrapper">
            <input v-model="deviationAmountSell" class="custom-input" placeholder="0.9" style="padding-right:24px;" />
            <span class="suffix-label">%</span>
          </div>
        </div>
        <div class="form-row">
          <label>Price Group Buy</label>
          <div class="input-wrapper">
            <input v-model="priceGroupBuy" class="custom-input" :placeholder="quote" style="padding-right:40px;" />
            <span class="suffix-label">{{ quote }}</span>
          </div>
        </div>
        <div class="form-row">
          <label>Price Group Sell</label>
          <div class="input-wrapper">
            <input v-model="priceGroupSell" class="custom-input" :placeholder="quote" style="padding-right:40px;" />
            <span class="suffix-label">{{ quote }}</span>
          </div>
        </div>
        <div class="form-row">
          <label class="custom-checkbox">
            <input type="checkbox" v-model="usePriceGroup" />
            Use Price Group
          </label>
        </div>
      </div>
    </div>

    <!-- Price Actions -->
    <div class="config-section">
      <div class="section-header" @click="showPriceActions = !showPriceActions">
        <span>💰 Quick Price</span>
        <span class="collapse-icon">{{ showPriceActions ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showPriceActions" class="section-content">
        <div class="price-actions">
          <div class="action-group">
            <span class="action-label">Lower (-)</span>
            <div class="action-buttons">
              <button class="action-btn buy" @click="updateLowerPrice(0.01)">1%</button>
              <button class="action-btn buy" @click="updateLowerPrice(0.02)">2%</button>
              <button class="action-btn buy" @click="updateLowerPrice(0.05)">5%</button>
              <button class="action-btn buy" @click="updateLowerPrice(0.1)">10%</button>
              <button class="action-btn buy" @click="updateLowerPrice(0.2)">20%</button>
              <button class="action-btn buy" @click="updateLowerPrice(0.5)">50%</button>
            </div>
          </div>
          <div class="action-group">
            <span class="action-label">Upper (+)</span>
            <div class="action-buttons">
              <button class="action-btn sell" @click="updateUpperPrice(0.01)">1%</button>
              <button class="action-btn sell" @click="updateUpperPrice(0.02)">2%</button>
              <button class="action-btn sell" @click="updateUpperPrice(0.05)">5%</button>
              <button class="action-btn sell" @click="updateUpperPrice(0.1)">10%</button>
              <button class="action-btn sell" @click="updateUpperPrice(0.2)">20%</button>
              <button class="action-btn sell" @click="updateUpperPrice(0.5)">50%</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Strategies Management -->
    <div class="config-section">
      <div class="section-header" @click="showStrategies = !showStrategies">
        <span>💾 Strategies</span>
        <span class="collapse-icon">{{ showStrategies ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showStrategies" class="section-content">
        <div class="form-row">
          <select v-model="strategyPicker" @change="selectStrategy" class="custom-input">
            <option value="">Select strategy…</option>
            <option v-for="o in strategyPickerOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
          </select>
        </div>
        <div class="strategy-buttons">
          <button class="strategy-btn primary" @click="addStrategy">Add</button>
          <button class="strategy-btn" @click="editStrategy">Edit</button>
          <button class="strategy-btn danger" @click="deleteStrategy">Del</button>
          <button class="strategy-btn danger" @click="deleteAllStrategies">Clear</button>
        </div>
      </div>
    </div>

    <!-- Balance Statistics -->
    <div class="config-section">
      <div class="section-header" style="background:rgba(16,235,4,0.1); border-color:rgba(16,235,4,0.3);">
        <span>💰 Bot Statistics</span>
      </div>
      <div class="section-content" style="display:grid; grid-template-columns:repeat(3,1fr); gap:8px; padding:8px;">
        <div style="background:rgba(16,235,4,0.1); border:1px solid rgba(16,235,4,0.2); border-radius:4px; padding:8px;">
          <div style="font-size:9px; color:rgba(16,235,4,0.6); font-weight:700;">Base Balance</div>
          <div style="font-size:14px; font-weight:800; color:#10eb04;">{{ BalanceBase.toFixed(4) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">USD: ${{ BalanceBaseInUSD.toFixed(2) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">Profit: {{ BalanceBaseProfit.toFixed(4) }}</div>
        </div>
        <div style="background:rgba(250,204,21,0.1); border:1px solid rgba(250,204,21,0.2); border-radius:4px; padding:8px;">
          <div style="font-size:9px; color:rgba(250,204,21,0.6); font-weight:700;">Quote Balance</div>
          <div style="font-size:14px; font-weight:800; color:#facc15;">{{ BalanceQuote.toFixed(4) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">USD: ${{ BalanceQuoteInUSD.toFixed(2) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">Profit: {{ BalanceQuoteProfit.toFixed(4) }}</div>
        </div>
        <div style="background:rgba(100,200,100,0.1); border:1px solid rgba(100,200,100,0.2); border-radius:4px; padding:8px;">
          <div style="font-size:9px; color:rgba(100,200,100,0.6); font-weight:700;">Bot Profit</div>
          <div :style="{fontSize:'14px',fontWeight:'800',color:BalanceBotProfit>=0?'#10eb04':'#eb0404'}">{{ BalanceBotProfit.toFixed(4) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">Initial: ${{ BalanceBotValInitiala.toFixed(2) }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">TP1: {{ TakeProfitBotSTR1 || '-' }}</div>
          <div style="font-size:8px; color:rgba(255,255,255,0.4);">TP2: {{ TakeProfitBotSTR2 || '-' }}</div>
        </div>
      </div>
    </div>

    <!-- RSI at Creation -->
    <div class="config-section">
      <div class="section-header" @click="showRSIInfo = !showRSIInfo">
        <span>📊 RSI at Creation</span>
        <span class="collapse-icon">{{ showRSIInfo ? '▼' : '▶' }}</span>
      </div>
      <div v-show="showRSIInfo" class="section-content">
        <div class="rsi-grid">
          <div v-for="(value, timeframe) in rsiValues" :key="timeframe"
            class="rsi-item" :class="getRSIClass(value)">
            <span class="rsi-timeframe">{{ timeframe }}</span>
            <span class="rsi-value">{{ value !== null ? Number(value).toFixed(2) : '-' }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Button -->
    <div style="margin-top:4px;">
      <button class="create-btn" @click="createGridBot">🚀 Create Grid Bot</button>
    </div>

    <!-- Status message -->
    <div v-if="statusMsg" :style="{padding:'6px 8px',borderRadius:'4px',fontSize:'10px',fontWeight:600,
      background: statusType==='error'?'rgba(235,4,4,0.12)':'rgba(16,235,4,0.1)',
      color: statusType==='error'?'#eb0404':'#10eb04',
      border: `1px solid ${statusType==='error'?'rgba(235,4,4,0.3)':'rgba(16,235,4,0.25)'}`}">
      {{ statusMsg }}
    </div>
  </div><!-- end gridbot-form-container -->

  </div><!-- end gbfp-root -->
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMarket } from '../composables/useMarket'

const { exchange: mktExchange, symbol: mktSymbol } = useMarket()

const currentExchange = ref(mktExchange.value || 'kraken')
const currentSymbol   = ref(mktSymbol.value  || 'XBT/USD')

const base  = computed(() => currentSymbol.value.split('/')[0] || currentSymbol.value.split('-')[0] || 'BASE')
const quote = computed(() => currentSymbol.value.split('/')[1] || currentSymbol.value.split('-')[1] || 'QUOTE')

const bestBid = ref<number | null>(null)
const bestAsk = ref<number | null>(null)
const currentPrice = ref<number | null>(null)

// OrderBook raw data
const obBids = ref<[number,number][]>([])
const obAsks = ref<[number,number][]>([])
const exchangeReady = ref(false)

const obSpread = computed(() => {
  if (!bestBid.value || !bestAsk.value) return ''
  return (bestAsk.value - bestBid.value).toFixed(5)
})

function obRows(levels: [number,number][], reverse = false) {
  if (!levels.length) return []
  const maxQty = Math.max(...levels.map(l => l[1]))
  const rows = levels.slice(0, 12).map(([p, q]) => ({
    price: p.toFixed(5), qty: q.toFixed(4), depth: maxQty > 0 ? q / maxQty : 0
  }))
  return reverse ? rows.reverse() : rows
}
const bidsDisplay = computed(() => obRows(obBids.value))
const asksDisplay = computed(() => obRows(obAsks.value, true))

const chartUrl = computed(() => {
  const sym = currentSymbol.value.replace('/', '').replace('-', '')
  const ex  = currentExchange.value.toLowerCase()
  let tvSym = sym
  if (ex === 'kraken') tvSym = `KRAKEN:${sym}`
  else if (ex === 'coinbase' || ex === 'coinbaseadvanced') tvSym = `COINBASE:${sym}`
  else if (ex === 'hyperliquid') tvSym = `HYPERLIQUID:${sym}`
  return `https://www.tradingview.com/widgetembed/?frameElementId=gbfp_chart&symbol=${encodeURIComponent(tvSym)}&interval=15&theme=dark&style=1&locale=en&toolbar_bg=%230f1419&hide_top_toolbar=0&hide_legend=0&save_image=0&calendar=0&studies=RSI%40tv-basicstudies&show_popup_button=1`
})

const name = ref(`ScalpPlus+Classic_${Date.now().toString().slice(-4)}`)
const strategyPicker = ref('')
const strategyPickerOptions = ref<{value:string;label:string}[]>([])
const lowerPrice = ref('')
const upperPrice = ref('')
const amountType = ref('incrementalPercent')
const amountTypeOptions = [
  { value: 'quantityPerGrid', label: 'Qty Per Grid' },
  { value: 'totalAmount',     label: 'Total Amount' },
  { value: 'incrementalPercent', label: 'Incremental Amount' },
]
const amount    = ref('')
const amountUnit = ref('quote')
const nrOfGrids = ref('')

// LCX pair constraints
const lcxPairLimits = ref<Record<string, {minBase:number;minQuote:number;tickSize:number;lotSize:number}>>({})

async function fetchLcxPairLimits() {
  try {
    const pairs = await invoke<string[]>('get_pairs', { exchange: 'lcx' })
    if (Array.isArray(pairs)) {
      const map: typeof lcxPairLimits.value = {}
      for (const sym of pairs) {
        try {
          const info = await invoke<any>('get_pair_precision', { exchange: 'lcx', symbol: sym })
          map[sym] = {
            minBase:  info.min_base  || 0,
            minQuote: info.min_quote || 0,
            tickSize: info.price_decimals  ? Math.pow(10, -info.price_decimals)  : 0.0001,
            lotSize:  info.amount_decimals ? Math.pow(10, -info.amount_decimals) : 1,
          }
        } catch {}
      }
      lcxPairLimits.value = map
    }
  } catch {
    lcxPairLimits.value = {
      'LCX/EUR':  { minBase:1, minQuote:0.1, tickSize:0.0001, lotSize:1 },
      'LCX/USDC': { minBase:1, minQuote:0.1, tickSize:0.0001, lotSize:1 },
    }
  }
}

const currentLimits = computed(() => {
  if ((currentExchange.value || '').toLowerCase() !== 'lcx') return null
  return lcxPairLimits.value[currentSymbol.value] || null
})

const amountAnalysis = computed(() => {
  const a     = parseFloat(amount.value)    || 0
  const grids = parseFloat(nrOfGrids.value) || 0
  const price = currentPrice.value || ((bestBid.value && bestAsk.value) ? (bestBid.value + bestAsk.value) / 2 : 0)
  if (!a || !grids || !price) return null

  const totalBase  = amountUnit.value === 'base'  ? a : a / price
  const totalQuote = amountUnit.value === 'quote' ? a : a * price
  const perOrderBase  = totalBase  / grids
  const perOrderQuote = totalQuote / grids

  const lim = currentLimits.value
  let warning: string | null = null
  let minRequired: string | null = null
  if (lim) {
    const violatesBase  = lim.minBase  > 0 && perOrderBase  < lim.minBase
    const violatesQuote = lim.minQuote > 0 && perOrderQuote < lim.minQuote
    if (violatesBase || violatesQuote) {
      const neededBase  = (lim.minBase  || 0) * grids
      const neededQuote = (lim.minQuote || 0) * grids
      if (amountUnit.value === 'base') {
        const need = Math.max(neededBase, neededQuote / price)
        minRequired = `Need >= ${need.toFixed(4)} ${base.value}`
      } else {
        const need = Math.max(neededQuote, neededBase * price)
        minRequired = `Need >= ${need.toFixed(2)} ${quote.value}`
      }
      warning = `Per-order ${perOrderBase.toFixed(4)} ${base.value} / ${perOrderQuote.toFixed(2)} ${quote.value} < exchange min (${lim.minBase} ${base.value}, ${lim.minQuote} ${quote.value})`
    }
  }
  return { totalBase, totalQuote, perOrderBase, perOrderQuote, warning, minRequired }
})

const ordersSide = ref('buyOrSell')
const ordersSideOptions = [
  { value: 'buyOrSell', label: 'Buy & Sell' },
  { value: 'buyOnly',   label: 'Buy Only'   },
  { value: 'sellOnly',  label: 'Sell Only'  },
]
const incrementalPercentAmountBuy  = ref('')
const incrementalPercentAmountSell = ref('')
const deviationPriceBuy    = ref('')
const deviationPriceSell   = ref('')
const deviationAmountBuy   = ref('')
const deviationAmountSell  = ref('')
const usePriceGroup  = ref(false)
const priceGroupBuy  = ref('')
const priceGroupSell = ref('')

const pyramidEnabled    = ref(false)
const pyramidType       = ref('linear')
const pyramidMultiplier = ref(1.5)
const pyramidTypeOptions = [
  { value: 'linear',      label: 'Linear (1%, 2%, 3%, …)'            },
  { value: 'exponential', label: 'Exponential (1%, 1.5%, 2.25%, …)'  },
]

const BalanceBase          = ref(0)
const BalanceQuote         = ref(0)
const BalanceBaseInUSD     = ref(0)
const BalanceQuoteInUSD    = ref(0)
const BalanceBaseProfit    = ref(0)
const BalanceQuoteProfit   = ref(0)
const BalanceBotProfit     = ref(0)
const BalanceBotValInitiala = ref(0)
const TakeProfitBotSTR1    = ref('')
const TakeProfitBotSTR2    = ref('')

const rsiValues = ref<Record<string, number | null>>({
  '1m':null,'5m':null,'15m':null,'30m':null,'1h':null,'2h':null,'6h':null,'1d':null
})

const showBasicConfig   = ref(true)
const showAdvancedConfig = ref(false)
const showPyramidConfig  = ref(true)
const showPriceActions   = ref(true)
const showStrategies     = ref(false)
const showRSIInfo        = ref(true)

const statusMsg  = ref('')
const statusType = ref('success')
let initialDeviationApplied = false
let orderBookTimer: ReturnType<typeof setInterval> | null = null

function showStatus(msg: string, type = 'success') {
  statusMsg.value = msg; statusType.value = type
  setTimeout(() => { statusMsg.value = '' }, 3500)
}

async function ensureExchangeReady() {
  if (exchangeReady.value) return
  try {
    await invoke('exchange_register_public', { exchange: currentExchange.value })
    exchangeReady.value = true
  } catch {}
}

async function fetchOrderBookPolling() {
  await ensureExchangeReady()
  if (!exchangeReady.value) return
  try {
    const ob = await invoke<any>('exchange_orderbook', {
      exchange: currentExchange.value,
      symbol:   currentSymbol.value,
      depth:    20,
    })
    // exchange_orderbook returns { bids: [[price,qty],...], asks: [[price,qty],...] }
    if (ob?.bids?.length) {
      obBids.value = ob.bids
      bestBid.value = ob.bids[0][0]
    }
    if (ob?.asks?.length) {
      obAsks.value = ob.asks
      bestAsk.value = ob.asks[0][0]
    }
    if (bestBid.value && bestAsk.value)
      currentPrice.value = (bestBid.value + bestAsk.value) / 2
  } catch {
    // fallback: try ticker
    try {
      const t = await invoke<any>('exchange_ticker', {
        exchange: currentExchange.value,
        symbol:   currentSymbol.value,
      })
      if (t?.bid) bestBid.value = t.bid
      if (t?.ask) bestAsk.value = t.ask
      if (t?.last) currentPrice.value = t.last
    } catch {}
  }
}

async function fetchRSIValues() {
  const timeframes = ['1m','5m','15m','30m','1h','2h','6h','1d']
  await Promise.all(timeframes.map(async tf => {
    try {
      const r = await invoke<any>('calculate_indicators', {
        exchange:   currentExchange.value,
        symbol:     currentSymbol.value,
        timeframe:  tf,
      })
      rsiValues.value[tf] = (r?.success && r?.data?.currentRSI != null)
        ? parseFloat(r.data.currentRSI) || 0
        : 0
    } catch { rsiValues.value[tf] = 0 }
  }))
}

async function fetchBalanceForSymbol() {
  try {
    const balances = await invoke<any[]>('get_balances', {
      exchange: currentExchange.value,
      apiKeyId: null,
    })
    if (Array.isArray(balances)) {
      BalanceBase.value  = balances.find(b => b.coin === base.value)?.free  || 0
      BalanceQuote.value = balances.find(b => b.coin === quote.value)?.free || 0
    }
  } catch {}
}

function getRSIClass(rsi: number | null) {
  if (rsi === null) return ''
  if (rsi >= 70) return 'overbought'
  if (rsi <= 30) return 'oversold'
  if (rsi >= 50) return 'bullish'
  return 'bearish'
}

function updateLowerPrice(dev = 0.01) {
  if (bestBid.value) lowerPrice.value = (bestBid.value * (1 - dev)).toFixed(6)
}
function updateUpperPrice(dev = 0.01) {
  if (bestAsk.value) upperPrice.value = (bestAsk.value * (1 + dev)).toFixed(6)
}
function applyInitialDeviation() {
  if (!initialDeviationApplied) {
    updateLowerPrice(); updateUpperPrice()
    initialDeviationApplied = true
  }
}

function selectStrategy() {
  const store = JSON.parse(localStorage.getItem('strategiesStore') || '[]') as any[]
  const s = store.find(x => x.name === strategyPicker.value)
  if (!s) return
  name.value                       = s.name
  lowerPrice.value                 = s.lowerPrice
  upperPrice.value                 = s.upperPrice
  amountType.value                 = s.amountType
  amount.value                     = s.amount
  amountUnit.value                 = s.amountUnit || 'quote'
  nrOfGrids.value                  = s.nrOfGrids
  ordersSide.value                 = s.ordersSide
  incrementalPercentAmountBuy.value  = s.incrementalPercentAmountBuy
  incrementalPercentAmountSell.value = s.incrementalPercentAmountSell
  deviationPriceBuy.value          = s.deviationPriceBuy
  deviationPriceSell.value         = s.deviationPriceSell
  deviationAmountBuy.value         = s.deviationAmountBuy
  deviationAmountSell.value        = s.deviationAmountSell
  usePriceGroup.value              = s.usePriceGroup
  priceGroupBuy.value              = s.priceGroupBuy
  priceGroupSell.value             = s.priceGroupSell
}

function strategyFromForm() {
  return {
    name: name.value, exchange: currentExchange.value, symbol: currentSymbol.value,
    lowerPrice: lowerPrice.value, upperPrice: upperPrice.value,
    amountType: amountType.value, amount: amount.value, amountUnit: amountUnit.value,
    nrOfGrids: nrOfGrids.value, ordersSide: ordersSide.value,
    incrementalPercentAmountBuy: incrementalPercentAmountBuy.value,
    incrementalPercentAmountSell: incrementalPercentAmountSell.value,
    deviationPriceBuy: deviationPriceBuy.value, deviationPriceSell: deviationPriceSell.value,
    deviationAmountBuy: deviationAmountBuy.value, deviationAmountSell: deviationAmountSell.value,
    usePriceGroup: usePriceGroup.value, priceGroupBuy: priceGroupBuy.value, priceGroupSell: priceGroupSell.value,
  }
}

function addStrategy() {
  const store: any[] = JSON.parse(localStorage.getItem('strategiesStore') || '[]')
  const s = strategyFromForm()
  if (!store.some(x => x.name === s.name)) {
    store.push(s)
    localStorage.setItem('strategiesStore', JSON.stringify(store))
    strategyPickerOptions.value.push({ value: s.name, label: s.name })
    strategyPicker.value = s.name
  }
}

function editStrategy() {
  const store: any[] = JSON.parse(localStorage.getItem('strategiesStore') || '[]')
  const idx = store.findIndex(x => x.name === strategyPicker.value)
  if (idx !== -1) { Object.assign(store[idx], strategyFromForm()); localStorage.setItem('strategiesStore', JSON.stringify(store)) }
}

function deleteStrategy() {
  let store: any[] = JSON.parse(localStorage.getItem('strategiesStore') || '[]')
  store = store.filter(x => x.name !== strategyPicker.value)
  strategyPickerOptions.value = strategyPickerOptions.value.filter(o => o.value !== strategyPicker.value)
  strategyPicker.value = ''
  localStorage.setItem('strategiesStore', JSON.stringify(store))
  lowerPrice.value=''; upperPrice.value=''; amount.value=''; nrOfGrids.value=''
}

function deleteAllStrategies() {
  localStorage.setItem('strategiesStore', '[]')
  strategyPickerOptions.value = []
  strategyPicker.value = ''
}

async function createGridBot() {
  const price   = parseFloat(String(currentPrice.value)) || 0
  const baseBal = BalanceBase.value
  const quotBal = BalanceQuote.value
  const initVal = quotBal + baseBal * price

  const data = {
    name: name.value,
    exchange: currentExchange.value,
    symbol: currentSymbol.value,
    lowerPrice: lowerPrice.value,
    upperPrice: upperPrice.value,
    amountType: amountType.value,
    amount: amount.value,
    amountUnit: amountUnit.value,
    nrOfGrids: nrOfGrids.value,
    ordersSide: ordersSide.value,
    incrementalPercentAmountBuy:  incrementalPercentAmountBuy.value,
    incrementalPercentAmountSell: incrementalPercentAmountSell.value,
    apiKeyId: null,
    priceAtCreation:    price,
    timestampCreation:  Math.floor(Date.now() / 1000),
    rsiAtCreation:      rsiValues.value,
    initialBalanceBase:      baseBal.toString(),
    initialBalanceQuote:     quotBal.toString(),
    initialBalanceBaseUSD:   (baseBal * price).toString(),
    initialBalanceQuoteUSD:  quotBal.toString(),
    initialBotValue:         initVal.toString(),
    initialBotValueUSD:      initVal.toString(),
    gridLowerPrice:  parseFloat(lowerPrice.value) || 0,
    gridUpperPrice:  parseFloat(upperPrice.value) || 0,
    maxDrawdown: price ? ((price - parseFloat(lowerPrice.value)) / price * 100).toString() : '0',
    maxUpside:   price ? ((parseFloat(upperPrice.value) - price) / price * 100).toString() : '0',
    dev_price_buy:   deviationPriceBuy.value,
    dev_price_sell:  deviationPriceSell.value,
    dev_amount_buy:  deviationAmountBuy.value,
    dev_amount_sell: deviationAmountSell.value,
    BalanceBot: {
      BalanceBase:      baseBal.toString(),
      BalanceQuote:     quotBal.toString(),
      BalanceBaseInUSD: (baseBal * price).toString(),
      BalanceQuoteInUSD: quotBal.toString(),
      BalanceBaseProfit: '0', BalanceQuoteProfit: '0', BalanceBotProfit: '0',
      BalanceBotValInitiala: initVal.toString(),
    },
    TakeProfitBot: { TakeProfitBotSTR1: TakeProfitBotSTR1.value || '-', TakeProfitBotSTR2: TakeProfitBotSTR2.value || '-' },
    PyramidGrid: {
      enabled:    pyramidEnabled.value,
      type:       pyramidType.value,
      multiplier: pyramidMultiplier.value,
    },
  }

  try {
    await invoke('create_bot', data)
    showStatus('✅ GridBot created')
  } catch (e: any) {
    showStatus('❌ ' + (e?.message || String(e)), 'error')
  }
}

onMounted(async () => {
  orderBookTimer = setInterval(fetchOrderBookPolling, 500)
  applyInitialDeviation()
  await Promise.allSettled([fetchRSIValues(), fetchBalanceForSymbol(), fetchLcxPairLimits()])
  const store: any[] = JSON.parse(localStorage.getItem('strategiesStore') || '[]')
  strategyPickerOptions.value = store.map(s => ({ value: s.name, label: s.name }))
})

onUnmounted(() => {
  if (orderBookTimer) clearInterval(orderBookTimer)
})
</script>

<style scoped>
* { box-sizing:border-box; margin:0; padding:0; }

/* ── Root 2-column layout ── */
.gbfp-root {
  display:grid;
  grid-template-columns: 480px 1fr;
  grid-template-rows: 100%;
  height:100%;
  gap:4px;
  overflow:hidden;
  background:#0b0e14;
}

/* ── RIGHT: chart + orderbook ── */
.gbfp-right {
  display:grid;
  grid-template-rows: 1fr 240px;
  gap:4px;
  overflow:hidden;
  min-width:0;
}
.gbfp-chart-wrap {
  background:#0f1419;
  border:1px solid #2a3441;
  border-radius:4px;
  overflow:hidden;
  min-height:0;
}

/* OrderBook */
.gbfp-ob {
  background:#0f1419;
  border:1px solid #2a3441;
  border-radius:4px;
  display:flex;
  flex-direction:column;
  overflow:hidden;
  font-family:'Courier New',monospace;
}
.gbfp-ob-header {
  display:flex;
  align-items:center;
  gap:8px;
  padding:4px 8px;
  background:#1a1f2e;
  border-bottom:1px solid #2a3441;
  font-size:10px;
  font-weight:700;
  color:#e0e0e0;
  flex-shrink:0;
}
.gbfp-ob-body {
  display:grid;
  grid-template-columns:1fr 60px 1fr;
  flex:1;
  overflow:hidden;
}
.gbfp-ob-col {
  display:flex;
  flex-direction:column;
  overflow:hidden;
}
.gbfp-ob-col-hdr {
  font-size:9px;
  font-weight:700;
  text-align:center;
  padding:3px 0;
  border-bottom:1px solid #2a3441;
  flex-shrink:0;
}
.gbfp-ob-row {
  display:flex;
  justify-content:space-between;
  align-items:center;
  padding:1px 6px;
  font-size:9px;
  transition:background .1s;
}
.gbfp-ob-row:hover { opacity:.85; }
.ob-price { font-weight:700; }
.ob-qty   { color:rgba(255,255,255,0.45); }
.gbfp-ob-mid {
  display:flex;
  flex-direction:column;
  align-items:center;
  justify-content:center;
  border-left:1px solid #2a3441;
  border-right:1px solid #2a3441;
  padding:4px 2px;
}
.ob-mid-bid { font-size:11px; font-weight:800; color:#4ade80; }
.ob-mid-ask { font-size:11px; font-weight:800; color:#f87171; }

/* ── LEFT: form ── */
.gridbot-form-container {
  width:480px; height:100%;
  background:#1a1f2e; border:1px solid #2a3441; border-radius:8px;
  padding:10px; display:flex; flex-direction:column; gap:8px;
  font-size:11px; overflow-y:auto; overflow-x:hidden; color:#e0e0e0;
  font-family:'Segoe UI',system-ui,-apple-system,sans-serif;
  box-shadow:0 8px 32px rgba(0,0,0,0.6);
}
.gridbot-form-container::-webkit-scrollbar { width:6px; }
.gridbot-form-container::-webkit-scrollbar-thumb { background:rgba(128,128,128,0.4); border-radius:3px; }
.gridbot-form-container::-webkit-scrollbar-thumb:hover { background:rgba(128,128,128,0.6); }
.gridbot-form-container::-webkit-scrollbar-track { background:transparent; }

.form-header { display:flex; align-items:center; gap:8px; padding:8px 10px; background:#0f1419; border-radius:4px; border:1px solid #2a3441; flex-shrink:0; }
.header-icon  { font-size:16px; }
.header-title { font-size:13px; font-weight:700; color:#e0e0e0; letter-spacing:0.3px; }

.prices-display { display:grid; grid-template-columns:1fr 1fr; gap:6px; flex-shrink:0; }
.price-item { padding:8px 6px; background:#0f1419; border-radius:4px; border:1px solid #2a3441; display:flex; flex-direction:column; align-items:center; gap:2px; }
.price-label { font-size:9px; font-weight:700; color:#888; text-transform:uppercase; letter-spacing:0.5px; }
.price-value { font-size:13px; font-weight:700; font-family:'Courier New',monospace; }
.price-item.bid .price-value { color:#4ade80; }
.price-item.ask .price-value { color:#f87171; }

.config-section { background:#0f1419; border:1px solid #2a3441; border-radius:4px; overflow:hidden; }
.section-header { padding:8px 10px; background:#1a1f2e; border-bottom:1px solid #2a3441; cursor:pointer; user-select:none; display:flex; justify-content:space-between; align-items:center; font-size:10px; font-weight:700; color:#999; text-transform:uppercase; transition:background 0.2s; letter-spacing:0.3px; }
.section-header:hover { background:#242936; }
.collapse-icon { font-size:10px; color:#666; }
.section-content { padding:10px; display:flex; flex-direction:column; gap:8px; }

.form-row { display:flex; flex-direction:column; gap:3px; }
.form-row > label { font-size:9px; color:#888; font-weight:700; letter-spacing:0.3px; }

.custom-input { background:#0b0e14; border:1px solid #2a3441; border-radius:4px; padding:6px 8px; color:#e0e0e0; font-size:10px; font-family:inherit; outline:none; transition:border-color 0.2s; width:100%; }
.custom-input:focus { border-color:#ff6b00; }
.custom-input::placeholder { color:#555; }
.custom-input:disabled { opacity:0.45; cursor:not-allowed; }

.input-wrapper { position:relative; display:flex; align-items:center; }
.input-wrapper .suffix-label { position:absolute; right:8px; font-size:9px; color:#888; font-weight:600; pointer-events:none; }

.custom-checkbox { display:flex; align-items:center; gap:6px; cursor:pointer; font-size:10px; color:#ccc; }
.custom-checkbox input[type="checkbox"] { width:14px; height:14px; accent-color:#ff6b00; cursor:pointer; flex-shrink:0; }

.amount-unit-toggle { display:grid; grid-template-columns:1fr 1fr; gap:4px; margin-top:2px; }
.unit-btn { font-size:9px; font-weight:700; padding:5px 8px; border-radius:4px; border:1px solid #2a3441; background:#0b0e14; color:#888; cursor:pointer; transition:all .15s; text-align:center; }
.unit-btn:hover { border-color:#ff6b00; color:#ccc; }
.unit-btn.active { background:rgba(255,107,0,0.15); border-color:#ff6b00; color:#ff6b00; }

.amount-analysis { margin-top:4px; padding:6px 8px; background:rgba(0,0,0,0.35); border-left:2px solid #2a3441; border-radius:3px; font-size:9px; display:flex; flex-direction:column; gap:2px; }
.amount-analysis.has-warning { border-left-color:#e90a15; background:rgba(233,10,21,0.08); }
.amount-analysis .row { display:flex; justify-content:space-between; color:#aaa; }
.amount-analysis .row.hint { color:#666; }
.amount-analysis .mono { font-family:'Courier New',monospace; color:#ccc; }
.amount-analysis .warning-line { color:#ff5252; font-weight:600; margin-top:2px; }
.amount-analysis .warning-line.fix { color:#10eb04; }

.price-actions { display:flex; flex-direction:column; gap:8px; }
.action-group  { display:flex; flex-direction:column; gap:4px; }
.action-label  { font-size:9px; font-weight:700; color:#888; text-transform:uppercase; letter-spacing:0.3px; }
.action-buttons { display:grid; grid-template-columns:repeat(3,1fr); gap:4px; }
.action-btn { padding:4px 0; border-radius:4px; border:1px solid #2a3441; background:#0b0e14; color:#ccc; font-size:9px; font-weight:600; cursor:pointer; transition:all .15s; text-align:center; }
.action-btn:hover { border-color:#ff6b00; background:rgba(255,107,0,0.1); }
.action-btn.buy  { color:#4ade80; }
.action-btn.buy:hover  { border-color:#4ade80; background:rgba(74,222,128,0.1); }
.action-btn.sell { color:#f87171; }
.action-btn.sell:hover { border-color:#f87171; background:rgba(248,113,113,0.1); }

.strategy-buttons { display:grid; grid-template-columns:repeat(4,1fr); gap:4px; }
.strategy-btn { padding:5px 0; border-radius:4px; border:1px solid #2a3441; background:#0b0e14; color:#ccc; font-size:9px; font-weight:600; cursor:pointer; transition:all .15s; text-align:center; }
.strategy-btn:hover { border-color:#ff6b00; background:rgba(255,107,0,0.1); }
.strategy-btn.primary { border-color:#ff6b00; color:#ff6b00; }
.strategy-btn.primary:hover { background:rgba(255,107,0,0.15); }
.strategy-btn.danger  { border-color:#e90a15; color:#e90a15; }
.strategy-btn.danger:hover  { background:rgba(233,10,21,0.12); }

.rsi-grid { display:grid; grid-template-columns:repeat(2,1fr); gap:4px; }
.rsi-item { display:flex; flex-direction:column; align-items:center; padding:6px 4px; border-radius:4px; border:1px solid rgba(255,255,255,0.08); background:rgba(0,0,0,0.3); transition:all .2s; }
.rsi-item:hover { transform:scale(1.03); border-color:rgba(0,255,255,0.2); }
.rsi-timeframe { font-size:8px; font-weight:700; text-transform:uppercase; color:rgba(255,255,255,0.4); margin-bottom:2px; }
.rsi-value { font-size:12px; font-weight:800; }
.rsi-item.bullish   { border-color:rgba(16,235,4,0.5);   background:rgba(16,235,4,0.08);   } .rsi-item.bullish .rsi-value   { color:#10eb04; }
.rsi-item.bearish   { border-color:rgba(233,10,21,0.5);  background:rgba(233,10,21,0.08);  } .rsi-item.bearish .rsi-value   { color:#e90a15; }
.rsi-item.overbought{ border-color:rgba(255,107,0,0.5);  background:rgba(255,107,0,0.08);  } .rsi-item.overbought .rsi-value{ color:#ff6b00; }
.rsi-item.oversold  { border-color:rgba(0,149,255,0.5);  background:rgba(0,149,255,0.08);  } .rsi-item.oversold .rsi-value  { color:#0095ff; }

.pyramid-section { border:2px solid rgba(16,235,4,0.4) !important; background:rgba(16,235,4,0.02) !important; }
.pyramid-section .section-header { background:rgba(16,235,4,0.12) !important; border-color:rgba(16,235,4,0.2) !important; }
.pyramid-section .section-header span:first-child { color:#10eb04 !important; }
.pyramid-content { padding:12px !important; background:rgba(16,235,4,0.03) !important; }
.pyramid-disabled-hint { font-size:10px; color:rgba(255,255,255,0.3); text-align:center; padding:6px; }

.create-btn { width:100%; padding:10px 0; border-radius:4px; border:none; background:#ff6b00; color:#fff; font-size:12px; font-weight:700; cursor:pointer; transition:all .2s; text-transform:uppercase; letter-spacing:0.5px; }
.create-btn:hover  { background:#e05f00; transform:scale(1.01); box-shadow:0 4px 20px rgba(255,107,0,0.3); }
.create-btn:active { transform:scale(0.98); }
</style>
