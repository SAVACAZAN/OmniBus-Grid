<template>
  <div class="shell">
    <!-- Click-outside overlay for key pickers -->
    <div v-if="showKeyPicker1 || showKeyPicker2"
      @click="showKeyPicker1=false; showKeyPicker2=false"
      style="position:fixed;inset:0;z-index:1999;"></div>

    <!-- Rail sidebar -->
    <aside v-if="!isAuthRoute" class="rail" aria-label="Navigation">
      <div class="brand-mark">O<span>.</span></div>
      <div class="rail-line"></div>
      <nav class="module-nav">
        <RouterLink v-for="item in navItems" :key="item.to" :to="item.to" custom
          v-slot="{ isActive, navigate }">
          <button type="button" class="rail-tab" :class="{ active: isActive }"
            :aria-label="item.label" :title="item.label" @click="navigate">{{ item.icon }}</button>
        </RouterLink>
      </nav>
      <div class="rail-bottom">OMIBUS</div>
    </aside>

    <div class="workspace">

      <!-- ── Topbar ── -->
      <header v-if="!isAuthRoute" class="topbar">

        <div v-if="cfg.market_selector" class="tb-left">

          <!-- ── EX1 (blue) ── -->
          <select v-model="ex1" @change="onEx1Change" class="tb-sel ex1-sel">
            <option v-for="ex in EXCHANGES.filter(e => e.id !== market.exchange2.value)" :key="ex.id" :value="ex.id">{{ ex.name }}</option>
          </select>

          <select v-model="sym1" @change="market.setSymbol(sym1)" class="tb-sel tb-pair">
            <option v-for="p in pairList1" :key="p" :value="p">{{ p }}</option>
          </select>

          <!-- EX1 key picker -->
          <div class="kp-wrap" @mouseleave="hoverKey1=false">
            <button class="kp-btn ex1-kp" @click="showKeyPicker1=!showKeyPicker1; showKeyPicker2=false"
              @mouseenter="hoverKey1=true">
              <span v-for="(kid,ki) in market.selectedKeyIds.value.slice(0,4)" :key="kid"
                class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
              <span class="kp-label">{{ keyLabel1 }}</span>
              <span class="kp-arrow" style="color:rgba(59,130,246,0.5)">▾</span>
            </button>
            <div v-if="hoverKey1 && market.selectedKeyIds.value.length > 0 && !showKeyPicker1" class="kp-tip">
              <div v-for="(kid,ki) in market.selectedKeyIds.value" :key="kid" class="kp-tip-row">
                <span class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
                <span>{{ keyNameEx1(kid) }}</span>
              </div>
            </div>
            <div v-if="showKeyPicker1" class="kp-drop ex1-drop">
              <label v-for="(k,ki) in market.exchangeKeys.value" :key="k.id" class="kp-item"
                :class="{ sel: market.selectedKeyIds.value.includes(k.id) }">
                <span class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
                <input type="checkbox" :checked="market.selectedKeyIds.value.includes(k.id)"
                  @change="market.toggleKey(k.id)" />
                <span>{{ k.label }}</span>
              </label>
              <div v-if="!market.exchangeKeys.value.length" class="kp-empty">No keys</div>
              <RouterLink to="/profile" @click="showKeyPicker1=false" class="kp-add">+ Add key…</RouterLink>
            </div>
          </div>

          <!-- EX1/EX2 divider -->
          <div class="ex-div"></div>

          <!-- ── EX2 (purple) ── -->
          <select v-model="ex2" @change="onEx2Change" class="tb-sel ex2-sel">
            <option value="">+ EX2</option>
            <option v-for="ex in EXCHANGES.filter(e => e.id !== market.exchange.value)" :key="ex.id" :value="ex.id">{{ ex.name }}</option>
          </select>

          <template v-if="market.exchange2.value">
            <select v-model="sym2" @change="market.setSymbol2(sym2)" class="tb-sel tb-pair ex2-pair">
              <option v-for="p in pairList2" :key="p" :value="p">{{ p }}</option>
            </select>

            <!-- EX2 key picker -->
            <div class="kp-wrap" @mouseleave="hoverKey2=false">
              <button class="kp-btn ex2-kp" @click="showKeyPicker2=!showKeyPicker2; showKeyPicker1=false"
                @mouseenter="hoverKey2=true">
                <span v-for="(kid,ki) in market.selectedKeyIds2.value.slice(0,4)" :key="kid"
                  class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
                <span class="kp-label">{{ keyLabel2 }}</span>
                <span class="kp-arrow" style="color:rgba(167,139,250,0.5)">▾</span>
              </button>
              <div v-if="hoverKey2 && market.selectedKeyIds2.value.length > 0 && !showKeyPicker2" class="kp-tip ex2-tip">
                <div v-for="(kid,ki) in market.selectedKeyIds2.value" :key="kid" class="kp-tip-row">
                  <span class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
                  <span>{{ keyNameEx2(kid) }}</span>
                </div>
              </div>
              <div v-if="showKeyPicker2" class="kp-drop ex2-drop">
                <label v-for="(k,ki) in market.exchangeKeys2.value" :key="k.id" class="kp-item"
                  :class="{ sel: market.selectedKeyIds2.value.includes(k.id) }">
                  <span class="kdot" :style="{background:KEY_COLORS[ki]}"></span>
                  <input type="checkbox" :checked="market.selectedKeyIds2.value.includes(k.id)"
                    @change="market.toggleKey2(k.id)" />
                  <span>{{ k.label }}</span>
                </label>
                <div v-if="!market.exchangeKeys2.value.length" class="kp-empty">No keys</div>
              </div>
            </div>
          </template>

          <!-- Quick pairs EX1 -->
          <template v-if="cfg.market_selector && market.quickPairs.value.length">
            <div class="tb-sep"></div>
            <span v-for="qp in market.quickPairs.value" :key="'1-'+qp"
              class="qp-chip" :class="{ active: market.symbol.value === qp }"
              @click="market.setSymbol(qp); sym1=qp">{{ qp }}</span>
          </template>

          <!-- Quick pairs EX2 -->
          <template v-if="cfg.market_selector && market.exchange2.value && market.quickPairs2.value.length">
            <div class="qp-ex-div"></div>
            <span v-for="qp in market.quickPairs2.value" :key="'2-'+qp"
              class="qp-chip qp2" :class="{ active2: market.symbol2.value === qp }"
              @click="market.setSymbol2(qp); sym2=qp">{{ qp }}</span>
          </template>

          <!-- Layout picker (Charts page only) -->
          <template v-if="isChartsPage && cfg.charts">
            <div class="tb-sep"></div>
            <div class="qp-layout-picker">
              <button v-for="l in LAYOUTS" :key="l.id"
                :class="['qp-layout-btn', { active: charts.layout.value === l.id }]"
                :title="l.label" @click="charts.setLayout(l.id)">{{ l.icon }}</button>
            </div>
          </template>

          <div class="tb-flex"></div>
          <div class="brand-name">OMIBUS <span>/ {{ activeLabel }}</span></div>
        </div>

        <!-- Right -->
        <div class="topbar-right">
          <span class="pulse"></span>
          PAPER ENGINE
          <span class="divider"></span>
          <span>{{ clock }}</span>
          <span class="divider"></span>
          <RouterLink to="/profile" class="user-pill" title="Profile & API Keys">
            <span class="user-avatar">{{ userInitial }}</span>
            <span class="user-name">{{ displayName }}</span>
          </RouterLink>
        </div>
      </header>

      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useInvoke } from './composables/useTauri'
import { useSession } from './composables/useSession'
import { useMarket, KEY_COLORS, DEFAULT_PAIRS } from './composables/useMarket'
import { useCharts, LAYOUTS } from './composables/useCharts'

const route   = useRoute()
const session = useSession()
const invoke  = useInvoke()
const market  = useMarket()

const isAuthRoute  = computed(() => route.path === '/auth')
const isChartsPage = computed(() => route.path === '/charts')
const charts = useCharts()

// ── Module config ─────────────────────────────────────────────────────────────
interface Cfg { grid: boolean; charts: boolean; orderbook: boolean; trade: boolean; ai: boolean; auth: boolean; profile: boolean; market_selector: boolean; gridbotplus: boolean }
const cfg = ref<Cfg>({ grid:true, charts:true, orderbook:true, trade:true, ai:true, auth:true, profile:true, market_selector:true, gridbotplus:true })

const ALL_NAV = [
  { to: '/grid',      icon: '▦', label: 'Grid workspace', module: 'grid'      },
  { to: '/charts',    icon: '⌁', label: 'Market charts',  module: 'charts'    },
  { to: '/orderbook', icon: '≡', label: 'Order Book',     module: 'orderbook' },
  { to: '/trade',     icon: '⇅', label: 'Trade',          module: 'trade'     },
  { to: '/gridbotplus', icon: '🤖', label: 'GridBot+',      module: 'gridbotplus' },
  { to: '/gridbot',        icon: '⊞', label: 'GridBot',      module: 'grid'       },
  { to: '/gridbotformplus', icon: '⚙', label: 'GridBot V2',  module: 'gridbotplus' },
  { to: '/ai',        icon: '✧', label: 'AI Research',    module: 'ai'        },
  { to: '/profile',   icon: '◉', label: 'Profile',        module: 'profile'   },
]
const navItems    = computed(() => ALL_NAV.filter(n => cfg.value[n.module as keyof Cfg]))
const activeLabel = computed(() => ALL_NAV.find(n => route.path.startsWith(n.to))?.label ?? 'WORKSPACE')

// ── Exchange lists ────────────────────────────────────────────────────────────
const EXCHANGES = [
  { id: 'kraken',      name: 'Kraken'      },
  { id: 'coinbase',    name: 'Coinbase'    },
  { id: 'hyperliquid', name: 'Hyperliquid' },
]

// Local refs synced with market singleton
const ex1  = ref(market.exchange.value)
const sym1 = ref(market.symbol.value)
const ex2  = ref(market.exchange2.value)
const sym2 = ref(market.symbol2.value)

watch(market.exchange,  v => { ex1.value  = v })
watch(market.symbol,    v => { sym1.value = v })
watch(market.exchange2, v => { ex2.value  = v })
watch(market.symbol2,   v => { sym2.value = v })

function onEx1Change() { market.setExchange(ex1.value) }
function onEx2Change() { market.setExchange2(ex2.value) }

const pairList1 = computed(() => {
  const saved = localStorage.getItem(`quickPairs_${ex1.value}`)
  return saved ? JSON.parse(saved) as string[] : DEFAULT_PAIRS[ex1.value] ?? []
})
const pairList2 = computed(() => {
  if (!ex2.value) return []
  const saved = localStorage.getItem(`quickPairs_${ex2.value}`)
  return saved ? JSON.parse(saved) as string[] : DEFAULT_PAIRS[ex2.value] ?? []
})

// ── Key pickers ───────────────────────────────────────────────────────────────
const showKeyPicker1 = ref(false)
const showKeyPicker2 = ref(false)
const hoverKey1 = ref(false)
const hoverKey2 = ref(false)

const keyLabel1 = computed(() => {
  const ids = market.selectedKeyIds.value
  if (!ids.length) return 'No key'
  const first = market.exchangeKeys.value.find(k => k.id === ids[0])?.label ?? 'Key'
  return ids.length > 1 ? `${first} +${ids.length-1}` : first
})
const keyLabel2 = computed(() => {
  const ids = market.selectedKeyIds2.value
  if (!ids.length) return 'No key'
  const first = market.exchangeKeys2.value.find(k => k.id === ids[0])?.label ?? 'Key'
  return ids.length > 1 ? `${first} +${ids.length-1}` : first
})

function keyNameEx1(id: number) { return market.exchangeKeys.value.find(k=>k.id===id)?.label ?? `Key #${id}` }
function keyNameEx2(id: number) { return market.exchangeKeys2.value.find(k=>k.id===id)?.label ?? `Key #${id}` }

// ── Session + keys ────────────────────────────────────────────────────────────
async function loadKeys() {
  if (!session.user.value) return
  try { market.setKeys(await invoke<any[]>('get_api_keys', {})) } catch {}
}
watch(() => session.user.value, u => { if (u) loadKeys() })

// ── User pill ─────────────────────────────────────────────────────────────────
const displayName = computed(() => session.user.value?.displayName ?? '')
const userInitial = computed(() => displayName.value.charAt(0).toUpperCase() || '?')

// ── Clock ─────────────────────────────────────────────────────────────────────
const clock = ref('')
let timer: ReturnType<typeof setInterval>
function tick() { clock.value = new Date().toISOString().slice(11,19) + ' UTC' }

onMounted(async () => {
  tick(); timer = setInterval(tick, 1000)
  try { cfg.value = await invoke<Cfg>('get_modules') } catch {}
  loadKeys()
})
onUnmounted(() => clearInterval(timer))
</script>

<style scoped>
.tb-left { display:flex; align-items:center; gap:5px; flex:1; min-width:0; overflow-x:auto; scrollbar-width:none; }
.tb-left::-webkit-scrollbar { display:none; }
.tb-sep  { width:1px; height:18px; background:rgba(255,255,255,0.08); margin:0 4px; flex-shrink:0; }
.tb-flex { flex:1; min-width:8px; flex-shrink:0; }
.ex-div  { width:1px; height:18px; background:rgba(167,139,250,0.25); margin:0 2px; flex-shrink:0; }

/* selects */
.tb-sel {
  background:rgba(26,31,46,0.9); border-radius:4px;
  padding:3px 5px; font-size:10px; color:#e0e0e0; outline:none; cursor:pointer;
}
.ex1-sel { border:1px solid rgba(59,130,246,0.3); color:#3b82f6; font-weight:700; width:82px; }
.ex2-sel { border:1px solid rgba(167,139,250,0.3); color:#a78bfa; font-weight:700; width:82px; }
.tb-pair { border:1px solid rgba(59,130,246,0.25); width:90px; }
.ex2-pair { border-color:rgba(167,139,250,0.25); }

/* key picker shared */
.kp-wrap  { position:relative; }
.kp-btn   { display:flex; align-items:center; gap:3px; border-radius:4px; padding:3px 6px; font-size:10px; color:#e0e0e0; cursor:pointer; width:110px; overflow:hidden; }
.ex1-kp   { background:rgba(26,31,46,0.9); border:1px solid rgba(59,130,246,0.3); }
.ex2-kp   { background:rgba(26,22,46,0.9); border:1px solid rgba(167,139,250,0.3); }
.kp-btn:hover { filter:brightness(1.2); }
.kp-label { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.kp-arrow { font-size:8px; flex-shrink:0; }
.kdot     { width:7px; height:7px; border-radius:50%; flex-shrink:0; display:inline-block; }

.kp-tip   { position:absolute; top:calc(100% + 6px); left:0; z-index:2002; background:#0d1220; border:1px solid rgba(59,130,246,0.35); border-radius:6px; padding:6px 10px; min-width:140px; pointer-events:none; box-shadow:0 4px 14px rgba(0,0,0,.6); }
.ex2-tip  { border-color:rgba(167,139,250,0.35); background:#0d1020; }
.kp-tip-row { display:flex; align-items:center; gap:7px; padding:2px 0; font-size:10px; color:#e0e0e0; }

.kp-drop  { position:absolute; top:calc(100% + 3px); left:0; z-index:2001; border-radius:6px; padding:4px 0; min-width:160px; box-shadow:0 6px 20px rgba(0,0,0,.7); }
.ex1-drop { background:#0d1220; border:1px solid rgba(59,130,246,0.4); }
.ex2-drop { background:#0d1020; border:1px solid rgba(167,139,250,0.4); }

.kp-item  { display:flex; align-items:center; gap:6px; padding:5px 10px; cursor:pointer; font-size:10px; color:#e0e0e0; user-select:none; transition:background .12s; }
.kp-item:hover, .kp-item.sel { background:rgba(59,130,246,0.1); }
.kp-item input { cursor:pointer; margin:0; }
.kp-empty { padding:6px 10px; color:rgba(255,255,255,0.3); font-size:10px; }
.kp-add   { display:block; border-top:1px solid rgba(59,130,246,0.15); padding:5px 10px; font-size:10px; color:#3b82f6; text-decoration:none; margin-top:2px; }
.kp-add:hover { background:rgba(59,130,246,0.08); }

.qp-ex-div { width:1px; height:18px; background:rgba(167,139,250,0.25); margin:0 4px; flex-shrink:0; }
.qp-layout-picker { display:flex; gap:2px; flex-shrink:0; }
.qp-layout-btn {
  padding:2px 6px; border-radius:3px; font-size:11px;
  background:transparent; border:1px solid rgba(255,255,255,0.08);
  color:rgba(255,255,255,0.3); cursor:pointer; transition:all .1s;
}
.qp-layout-btn:hover { border-color:rgba(59,130,246,0.4); color:rgba(255,255,255,0.7); }
.qp-layout-btn.active { background:rgba(59,130,246,0.18); border-color:#3b82f6; color:#3b82f6; }

.qp-chip  { padding:2px 8px; border-radius:4px; font-size:10px; font-weight:600; cursor:pointer; white-space:nowrap; flex-shrink:0; border:1px solid rgba(59,130,246,0.2); color:rgba(255,255,255,0.35); transition:all .12s; }
.qp-chip:hover { border-color:rgba(59,130,246,0.5); color:rgba(255,255,255,0.7); }
.qp-chip.active { border-color:#3b82f6; background:rgba(59,130,246,0.2); color:#3b82f6; }

.qp2      { border-color:rgba(167,139,250,0.2); }
.qp2:hover { border-color:rgba(167,139,250,0.5); color:rgba(220,200,255,0.7); }
.qp2.active2 { border-color:#a78bfa; background:rgba(167,139,250,0.2); color:#a78bfa; }

/* user pill */
.user-pill { display:flex; align-items:center; gap:6px; padding:3px 8px 3px 3px; border-radius:20px; background:var(--surface); border:1px solid var(--border); text-decoration:none; color:var(--text); transition:border-color .15s; cursor:pointer; }
.user-pill:hover { border-color:var(--accent); }
.user-avatar { width:22px; height:22px; border-radius:50%; background:var(--accent); color:#000; font-size:.75rem; font-weight:700; display:flex; align-items:center; justify-content:center; }
.user-name { font-size:.8rem; max-width:100px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
</style>
