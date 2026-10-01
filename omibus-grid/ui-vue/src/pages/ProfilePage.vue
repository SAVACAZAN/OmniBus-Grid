<template>
<div style="height:100%; display:flex; flex-direction:column; overflow:hidden;">

  <!-- Tab bar -->
  <div style="display:flex; gap:0; flex-shrink:0; background:rgba(14,22,40,0.6); border-bottom:2px solid rgba(59,130,246,0.15); overflow-x:auto;">
    <div v-for="tab in tabs" :key="tab.id" @click="activeTab = tab.id"
      :style="{padding:'10px 18px', cursor:'pointer', fontWeight:700, fontSize:'12px', whiteSpace:'nowrap', display:'flex', alignItems:'center', gap:'6px',
        color: activeTab === tab.id ? '#3b82f6' : 'rgba(255,255,255,0.3)',
        borderBottom: activeTab === tab.id ? '2px solid #3b82f6' : '2px solid transparent', marginBottom:'-2px',
        background: activeTab === tab.id ? 'rgba(59,130,246,0.05)' : 'transparent'}">
      <span>{{ tab.icon }}</span> {{ tab.label }}
    </div>
  </div>

  <!-- Content -->
  <div style="flex:1; overflow-y:auto; padding:16px;">

    <!-- ========== ACCOUNT ========== -->
    <div v-if="activeTab === 'account'">
      <div class="card" style="margin-bottom:16px;">
        <div class="card-title">👤 Account</div>
        <div style="display:grid; grid-template-columns:140px 1fr; gap:8px 12px; font-size:13px; margin-bottom:20px;">
          <span style="color:rgba(255,255,255,0.4);">Display name</span>
          <span style="color:#e0e0e0;">{{ session.user.value?.displayName ?? '—' }}</span>
          <span style="color:rgba(255,255,255,0.4);">User ID</span>
          <span style="color:#e0e0e0; font-family:monospace;">#{{ session.user.value?.id ?? '—' }}</span>
        </div>
        <button @click="logout"
          style="padding:7px 18px; background:rgba(235,4,4,0.15); color:#eb0404; border:1px solid rgba(235,4,4,0.3); border-radius:5px; cursor:pointer; font-size:12px; font-weight:700;">
          Logout
        </button>
      </div>
    </div>

    <!-- ========== EXCHANGES ========== -->
    <div v-if="activeTab === 'exchanges'">
      <div class="card" style="margin-bottom:16px;">
        <div style="display:flex; align-items:center; gap:10px; margin-bottom:8px;">
          <span class="card-title" style="margin-bottom:0;">🔑 API Keys</span>
          <button @click="openExportModal"
            style="margin-left:auto; padding:5px 12px; font-size:11px; font-weight:700; background:rgba(16,235,4,0.15); color:#10eb04; border:1px solid rgba(16,235,4,0.3); border-radius:4px; cursor:pointer;">
            🔒 Export</button>
          <button @click="exportPlain"
            style="padding:5px 12px; font-size:11px; font-weight:700; background:rgba(251,191,36,0.12); color:#fbbf24; border:1px solid rgba(251,191,36,0.3); border-radius:4px; cursor:pointer;">
            📄 Plain</button>
          <button @click="openImportModal"
            style="padding:5px 12px; font-size:11px; font-weight:700; background:rgba(59,130,246,0.15); color:#60a5fa; border:1px solid rgba(59,130,246,0.3); border-radius:4px; cursor:pointer;">
            ⬇ Import</button>
        </div>
        <p style="color:rgba(255,255,255,0.4); font-size:11px; margin-bottom:12px;">🔒 Keys stored encrypted in SQLite · Export = .omnibus file criptat AES-256</p>

        <!-- Existing keys -->
        <div v-if="loadingKeys" style="text-align:center; padding:16px; color:rgba(255,255,255,0.3); font-size:12px;">Loading…</div>
        <div v-for="key in keys" :key="key.id"
          style="margin-bottom:8px; padding:12px; background:rgba(26,31,46,0.4); border:1px solid transparent; border-radius:6px; display:flex; align-items:center; gap:12px;">
          <div style="width:70px; font-weight:700; color:#3b82f6; font-size:12px;">{{ key.exchange }}</div>
          <div style="flex:1; min-width:0;">
            <div v-if="editingKeyId !== key.id" style="display:flex; align-items:center; gap:6px;">
              <span style="font-size:12px; font-weight:600; color:#e0e0e0;">{{ key.label }}</span>
              <button @click="startEditLabel(key)"
                style="font-size:9px; padding:1px 6px; border-radius:3px; cursor:pointer; background:rgba(59,130,246,0.1); border:1px solid rgba(59,130,246,0.25); color:#60a5fa; line-height:1.6;">✏</button>
            </div>
            <div v-else style="display:flex; align-items:center; gap:6px;">
              <input v-model="editingLabel" @keydown.enter="saveEditLabel(key.id)" @keydown.escape="cancelEditLabel"
                style="font-size:12px; font-weight:600; padding:2px 8px; border-radius:4px; border:1px solid rgba(59,130,246,0.5); background:rgba(59,130,246,0.08); color:#e0e0e0; width:180px; outline:none;"
                autofocus />
              <button @click="saveEditLabel(key.id)"
                style="font-size:9px; padding:2px 8px; border-radius:3px; cursor:pointer; background:rgba(16,235,4,0.2); border:1px solid rgba(16,235,4,0.3); color:#10eb04; font-weight:700;">✓</button>
              <button @click="cancelEditLabel"
                style="font-size:9px; padding:2px 8px; border-radius:3px; cursor:pointer; background:rgba(235,4,4,0.1); border:1px solid rgba(235,4,4,0.25); color:#f87171; font-weight:700;">✕</button>
            </div>
            <div style="font-size:11px; color:rgba(255,255,255,0.3); font-family:monospace; margin-top:2px;">{{ key.api_key_masked }}</div>
          </div>
          <button @click="removeApiKey(key.id)"
            style="background:rgba(235,4,4,0.2); color:#eb0404; border:1px solid rgba(235,4,4,0.3); padding:5px 12px; border-radius:4px; cursor:pointer; font-size:10px; font-weight:700;">Delete</button>
        </div>
        <div v-if="!loadingKeys && keys.length === 0"
          style="text-align:center; padding:16px; color:rgba(255,255,255,0.3); font-size:12px;">No API keys configured</div>

        <!-- Add new key -->
        <div style="margin-top:16px; padding:16px; background:rgba(59,130,246,0.05); border:1px solid rgba(59,130,246,0.2); border-radius:8px;">
          <div style="font-size:12px; font-weight:700; color:#3b82f6; margin-bottom:10px;">+ Add New API Key</div>
          <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
            <div class="form-group"><label>Exchange</label>
              <select v-model="newKey.exchange" class="form-input" style="font-size:13px;">
                <option v-for="ex in EXCHANGES" :key="ex.id" :value="ex.id">{{ ex.name }}</option>
              </select>
            </div>
            <div class="form-group"><label>Label</label>
              <input v-model="newKey.label" placeholder="Trading Key, Scalping…" class="form-input" style="font-size:13px;"></div>
            <div class="form-group"><label>API Key</label>
              <input v-model="newKey.api_key" placeholder="Enter API key…" class="form-input" style="font-family:monospace; font-size:13px;"></div>
            <div class="form-group"><label>API Secret / PEM Key</label>
              <textarea v-model="newKey.api_secret" placeholder="Enter secret or paste PEM key…" class="form-input"
                style="font-family:monospace; font-size:11px; min-height:70px; resize:vertical; white-space:pre;"></textarea></div>
          </div>
          <div style="display:flex; gap:8px; margin-top:8px; align-items:center;">
            <button class="btn btn-primary" @click="addApiKey" style="padding:8px 20px; font-size:12px;">🔑 Save Key</button>
            <span v-if="keyMsg" :style="{fontSize:'11px', color: keyMsg.includes('Error') ? '#eb0404' : '#10eb04'}">{{ keyMsg }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- ========== QUICK PAIRS ========== -->
    <div v-if="activeTab === 'quickpairs'">
      <div class="card" style="margin-bottom:16px;">
        <div class="card-title">Quick Pairs (Trading tab bar)</div>
        <p style="color:rgba(255,255,255,0.4); font-size:11px; margin-bottom:12px;">Configure which pairs appear in the quick-switch bar. One pair per line.</p>
        <span v-if="quickPairsMsg" style="font-size:11px; font-weight:700; color:#10eb04;">{{ quickPairsMsg }}</span>
      </div>
      <div v-for="ex in EXCHANGES" :key="'qp-'+ex.id" class="card" style="margin-bottom:12px;">
        <div style="display:flex; align-items:center; gap:8px; margin-bottom:8px;">
          <span style="font-weight:700; color:#3b82f6; font-size:13px;">{{ ex.name }}</span>
          <span style="font-size:10px; color:rgba(255,255,255,0.3);">{{ (quickPairsEdit[ex.id] || '').split('\n').filter((s: string) => s.trim()).length }} pairs</span>
        </div>
        <textarea v-model="quickPairsEdit[ex.id]"
          style="width:100%; min-height:120px; background:rgba(15,20,25,0.8); border:1px solid rgba(59,130,246,0.2); color:#e0e0e0; padding:8px; font-family:monospace; font-size:12px; border-radius:4px; resize:vertical; line-height:1.6;"></textarea>
        <div style="display:flex; gap:6px; margin-top:6px;">
          <button @click="saveQuickPairs(ex.id)"
            style="padding:5px 14px; font-size:11px; font-weight:700; background:#3b82f6; color:white; border:none; border-radius:4px; cursor:pointer;">Save {{ ex.name }}</button>
          <button @click="resetQuickPairs(ex.id)"
            style="padding:5px 14px; font-size:11px; font-weight:700; background:rgba(239,68,68,0.2); color:#ef4444; border:1px solid rgba(239,68,68,0.3); border-radius:4px; cursor:pointer;">Reset</button>
        </div>
      </div>
    </div>

    <!-- ========== AI PROVIDERS ========== -->
    <div v-if="activeTab === 'ai'">
      <div class="card" style="margin-bottom:16px;">
        <div class="card-title">🤖 AI Provider API Keys</div>
        <p style="color:rgba(255,255,255,0.4); font-size:11px; margin-bottom:16px;">Stored locally. Used for AI-powered analysis and bots.</p>
        <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(280px, 1fr)); gap:12px;">
          <div v-for="ai in aiProviders" :key="ai.id"
            :style="{padding:'12px', borderRadius:'8px', border: '1px solid ' + ai.color + '30', background: ai.color + '08'}">
            <div style="display:flex; align-items:center; gap:8px; margin-bottom:8px;">
              <div :style="{width:'8px', height:'8px', borderRadius:'50%', background: aiKeys[ai.field] ? '#10eb04' : '#666'}"></div>
              <span :style="{fontWeight:700, fontSize:'13px', color: ai.color}">{{ ai.name }}</span>
              <span v-if="aiKeys[ai.field]" style="margin-left:auto; font-size:9px; color:#10eb04;">Connected</span>
            </div>
            <input v-model="aiKeys[ai.field]" type="password" :placeholder="ai.placeholder || 'API Key…'"
              class="form-input" style="font-family:monospace; font-size:12px; width:100%;">
          </div>
        </div>
        <div style="display:flex; gap:8px; margin-top:16px; align-items:center;">
          <button class="btn btn-primary" @click="saveAiKeys" style="padding:8px 20px; font-size:12px;">Save AI Keys</button>
          <span v-if="aiMsg" :style="{fontSize:'11px', color: aiMsg.includes('Error') ? '#eb0404' : '#10eb04'}">{{ aiMsg }}</span>
        </div>
      </div>
    </div>

    <!-- ========== SOCIAL ========== -->
    <div v-if="activeTab === 'social'">
      <div class="card" style="margin-bottom:16px;">
        <div class="card-title">🌐 Social & Notifications</div>
        <p style="color:rgba(255,255,255,0.4); font-size:11px; margin-bottom:16px;">Connect social accounts for alerts and bot status updates.</p>
        <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(300px, 1fr)); gap:12px;">
          <div v-for="sp in socialPlatforms" :key="sp.id"
            :style="{padding:'12px', borderRadius:'8px', border: '1px solid ' + sp.color + '30', background: sp.color + '08'}">
            <div style="display:flex; align-items:center; gap:8px; margin-bottom:8px;">
              <span style="font-size:16px;">{{ sp.icon }}</span>
              <span :style="{fontWeight:700, fontSize:'13px', color: sp.color}">{{ sp.name }}</span>
              <span v-if="socialKeys[sp.id]" style="margin-left:auto; font-size:9px; color:#10eb04;">✓ Set</span>
            </div>
            <input v-model="socialKeys[sp.id]"
              :type="sp.type === 'token' ? 'password' : 'text'"
              :placeholder="sp.type === 'handle' ? '@username' : sp.type === 'url' ? 'https://…' : sp.type === 'token' ? 'Token…' : 'ID…'"
              class="form-input" style="font-size:12px; width:100%;">
          </div>
        </div>
        <div style="display:flex; gap:8px; margin-top:16px; align-items:center;">
          <button class="btn btn-primary" @click="saveSocialKeys" style="padding:8px 20px; font-size:12px;">Save Social</button>
          <span v-if="socialMsg" :style="{fontSize:'11px', color: socialMsg.includes('Error') ? '#eb0404' : '#10eb04'}">{{ socialMsg }}</span>
        </div>
      </div>
    </div>

    <!-- ========== SPECIALIZED ========== -->
    <div v-if="activeTab === 'specialized'">
      <div class="card" style="margin-bottom:16px;">
        <div class="card-title">🎯 Specialized Platforms</div>
        <p style="color:rgba(255,255,255,0.4); font-size:11px; margin-bottom:16px;">Additional platform configurations stored locally.</p>

        <div style="padding:16px; border:1px solid rgba(59,130,246,0.2); border-radius:8px; margin-bottom:12px; background:rgba(59,130,246,0.03);">
          <div style="font-weight:700; font-size:14px; color:#3b82f6; margin-bottom:10px;">⚡ Hyperliquid</div>
          <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
            <div class="form-group"><label>Wallet Address</label><input class="form-input" style="font-family:monospace; font-size:12px;" placeholder="0x…"></div>
            <div class="form-group"><label>Private Key</label><input type="password" class="form-input" style="font-family:monospace; font-size:12px;" placeholder="Private key…"></div>
          </div>
          <div style="font-size:10px; color:rgba(235,4,4,0.5); margin-top:4px;">⚠ Private keys stored locally. Never share them.</div>
        </div>

        <div style="padding:16px; border:1px solid rgba(168,85,247,0.2); border-radius:8px; margin-bottom:12px; background:rgba(168,85,247,0.03);">
          <div style="font-weight:700; font-size:14px; color:#a855f7; margin-bottom:10px;">👻 Solana / Phantom</div>
          <div class="form-group"><label>Wallet Address</label><input class="form-input" style="font-family:monospace; font-size:12px;" placeholder="Solana address…"></div>
        </div>

        <div style="padding:16px; border:1px solid rgba(250,204,21,0.2); border-radius:8px; margin-bottom:12px; background:rgba(250,204,21,0.03);">
          <div style="font-weight:700; font-size:14px; color:#facc15; margin-bottom:10px;">🦊 MetaMask / EVM</div>
          <div class="form-group"><label>Wallet Address</label><input class="form-input" style="font-family:monospace; font-size:12px;" placeholder="0x…"></div>
        </div>

        <div style="padding:16px; border:1px solid rgba(100,100,100,0.2); border-radius:8px; background:rgba(100,100,100,0.03);">
          <div style="font-weight:700; font-size:14px; color:#666; margin-bottom:10px;">🗄 Database</div>
          <div style="font-size:11px; color:rgba(255,255,255,0.4);">
            <div>SQLite: %LOCALAPPDATA%\omibus-desktop\omibus.db</div>
            <div style="margin-top:6px;">API Keys: {{ keys.length }} configured</div>
          </div>
        </div>
      </div>
    </div>

  </div>

  <!-- ── Password modal (Export / Import) ──────────────────────────────────── -->
  <div v-if="pwModal" @click.self="closePwModal"
    style="position:fixed; inset:0; background:rgba(0,0,0,0.7); display:flex; align-items:center; justify-content:center; z-index:9999;">
    <div style="background:#0e1828; border:1px solid rgba(59,130,246,0.4); border-radius:10px; padding:28px 32px; min-width:340px; max-width:420px;">

      <template v-if="pwModal === 'export'">
        <div style="font-size:15px; font-weight:700; color:#e0e0e0; margin-bottom:18px;">⬆ Export Keys — Set Password</div>
        <p style="font-size:11px; color:rgba(255,255,255,0.4); margin-bottom:14px;">Fișierul .omnibus va fi criptat AES-256. Fără parolă nu poate fi citit.</p>
        <div class="form-group" style="margin-bottom:12px;">
          <label style="font-size:11px; color:rgba(255,255,255,0.5);">Parolă</label>
          <input v-model="pwValue" type="password" class="form-input" placeholder="Minimum 6 caractere" @keydown.enter="confirmExport" autofocus>
        </div>
        <div class="form-group" style="margin-bottom:16px;">
          <label style="font-size:11px; color:rgba(255,255,255,0.5);">Confirmă parola</label>
          <input v-model="pwValue2" type="password" class="form-input" placeholder="Repetă parola" @keydown.enter="confirmExport">
        </div>
        <div v-if="pwError" style="color:#eb0404; font-size:11px; margin-bottom:12px;">{{ pwError }}</div>
        <div style="display:flex; gap:10px; justify-content:flex-end;">
          <button @click="closePwModal" style="padding:8px 18px; font-size:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.15); border-radius:5px; color:rgba(255,255,255,0.5); cursor:pointer;">Anulează</button>
          <button @click="confirmExport" style="padding:8px 20px; font-size:12px; font-weight:700; background:rgba(16,235,4,0.2); border:1px solid rgba(16,235,4,0.4); border-radius:5px; color:#10eb04; cursor:pointer;">⬆ Exportă</button>
        </div>
      </template>

      <template v-if="pwModal === 'import'">
        <div style="font-size:15px; font-weight:700; color:#e0e0e0; margin-bottom:18px;">⬇ Import Keys — Introdu Parola</div>
        <p style="font-size:11px; color:rgba(255,255,255,0.4); margin-bottom:14px;">Fișier: <strong style="color:#60a5fa;">{{ pwFile?.name }}</strong></p>
        <div class="form-group" style="margin-bottom:16px;">
          <label style="font-size:11px; color:rgba(255,255,255,0.5);">Parola de decriptare</label>
          <input v-model="pwValue" type="password" class="form-input" placeholder="Parola folosită la export" @keydown.enter="confirmImport" autofocus>
        </div>
        <div v-if="pwError" style="color:#eb0404; font-size:11px; margin-bottom:12px;">{{ pwError }}</div>
        <div style="display:flex; gap:10px; justify-content:flex-end;">
          <button @click="closePwModal" style="padding:8px 18px; font-size:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.15); border-radius:5px; color:rgba(255,255,255,0.5); cursor:pointer;">Anulează</button>
          <button @click="confirmImport" style="padding:8px 20px; font-size:12px; font-weight:700; background:rgba(59,130,246,0.2); border:1px solid rgba(59,130,246,0.4); border-radius:5px; color:#60a5fa; cursor:pointer;">⬇ Importă</button>
        </div>
      </template>

    </div>
  </div>

</div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useInvoke } from '../composables/useTauri'
import { useSession } from '../composables/useSession'

const invoke  = useInvoke()
const router  = useRouter()
const session = useSession()

// ── Tabs ──────────────────────────────────────────────────────────────────────
const tabs = [
  { id: 'account',     icon: '👤', label: 'Account'      },
  { id: 'exchanges',   icon: '🔑', label: 'API Keys'     },
  { id: 'quickpairs',  icon: '⚡', label: 'Quick Pairs'  },
  { id: 'ai',          icon: '🤖', label: 'AI Providers' },
  { id: 'social',      icon: '🌐', label: 'Social'       },
  { id: 'specialized', icon: '🎯', label: 'Specialized'  },
]
const activeTab = ref('account')

// ── Auth ──────────────────────────────────────────────────────────────────────
async function logout() {
  await invoke('auth_logout').catch(() => {})
  session.clearUser()
  router.replace('/auth')
}

// ── Exchanges ─────────────────────────────────────────────────────────────────
const EXCHANGES = [
  { id: 'kraken',   name: 'Kraken'   },
  { id: 'coinbase', name: 'Coinbase' },
  { id: 'binance',  name: 'Binance'  },
  { id: 'bybit',    name: 'Bybit'    },
  { id: 'okx',      name: 'OKX'      },
  { id: 'kucoin',   name: 'KuCoin'   },
  { id: 'gate',     name: 'Gate.io'  },
  { id: 'mexc',     name: 'MEXC'     },
  { id: 'lcx',      name: 'LCX'      },
  { id: 'hyperliquid', name: 'Hyperliquid' },
]

// ── API Keys ──────────────────────────────────────────────────────────────────
interface ApiKey { id: number; exchange: string; label: string; api_key_masked: string }
const keys       = ref<ApiKey[]>([])
const loadingKeys = ref(false)
const keyMsg     = ref('')
const newKey     = reactive({ exchange: 'kraken', label: '', api_key: '', api_secret: '' })
const editingKeyId = ref<number | null>(null)
const editingLabel = ref('')

async function loadKeys() {
  loadingKeys.value = true
  try { keys.value = await invoke<ApiKey[]>('get_api_keys', {}) }
  catch (e) { console.error(e) }
  finally { loadingKeys.value = false }
}

async function addApiKey() {
  if (!newKey.api_key || !newKey.api_secret) { keyMsg.value = 'Key and Secret required'; return }
  try {
    const secret = newKey.api_secret.replace(/\\n/g, '\n')
    await invoke('save_api_key', { exchange: newKey.exchange, label: newKey.label || 'Main Key', apiKey: newKey.api_key, apiSecret: secret })
    keyMsg.value = 'Key saved!'
    newKey.label = ''; newKey.api_key = ''; newKey.api_secret = ''
    await loadKeys()
    setTimeout(() => keyMsg.value = '', 3000)
  } catch (e) { keyMsg.value = 'Error: ' + e }
}

async function removeApiKey(id: number) {
  if (!confirm('Delete this API key?')) return
  await invoke('delete_api_key', { id }).catch((e: any) => alert(e))
  await loadKeys()
}

function startEditLabel(key: ApiKey) { editingKeyId.value = key.id; editingLabel.value = key.label }
function cancelEditLabel() { editingKeyId.value = null; editingLabel.value = '' }

async function saveEditLabel(id: number) {
  const label = editingLabel.value.trim(); if (!label) return
  await invoke('rename_api_key', { id, label }).catch((e: any) => alert(e))
  editingKeyId.value = null; editingLabel.value = ''
  await loadKeys()
}

// ── Quick Pairs ───────────────────────────────────────────────────────────────
const quickPairsEdit = reactive<Record<string, string>>({})
const quickPairsMsg  = ref('')

const defaultPairs: Record<string, string[]> = {
  kraken:   ['XBT/USD','XBT/EUR','ETH/USD','ETH/EUR','SOL/USD'],
  coinbase: ['BTC-USD','BTC-EUR','ETH-USD','ETH-EUR','SOL-USD'],
  binance:  ['BTCUSDT','ETHUSDT','SOLUSDT','BNBUSDT'],
  bybit:    ['BTCUSDT','ETHUSDT','SOLUSDT'],
  lcx:      ['LCX/EUR','BTC/EUR','ETH/EUR','SOL/EUR'],
}

function loadQuickPairs() {
  for (const ex of EXCHANGES) {
    const saved = localStorage.getItem(`quickPairs_${ex.id}`)
    quickPairsEdit[ex.id] = saved ? JSON.parse(saved).join('\n') : (defaultPairs[ex.id] ?? []).join('\n')
  }
}

function saveQuickPairs(id: string) {
  const pairs = quickPairsEdit[id].split('\n').map((s: string) => s.trim()).filter((s: string) => s)
  localStorage.setItem(`quickPairs_${id}`, JSON.stringify(pairs))
  quickPairsMsg.value = `${id}: ${pairs.length} pairs saved!`
  setTimeout(() => quickPairsMsg.value = '', 3000)
}

function resetQuickPairs(id: string) {
  localStorage.removeItem(`quickPairs_${id}`)
  quickPairsEdit[id] = (defaultPairs[id] ?? []).join('\n')
  quickPairsMsg.value = `${id}: reset to defaults`
  setTimeout(() => quickPairsMsg.value = '', 3000)
}

// ── AI Providers ──────────────────────────────────────────────────────────────
const aiProviders = [
  { id: 'openai',      name: 'OpenAI',          color: '#10a37f', field: 'openai_key',      placeholder: 'sk-…'      },
  { id: 'anthropic',   name: 'Claude/Anthropic', color: '#d97706', field: 'anthropic_key',   placeholder: 'sk-ant-…'  },
  { id: 'gemini',      name: 'Google Gemini',    color: '#4285f4', field: 'gemini_key',       placeholder: 'AIza…'     },
  { id: 'groq',        name: 'Groq',             color: '#f55036', field: 'groq_key',         placeholder: 'gsk_…'     },
  { id: 'mistral',     name: 'Mistral',          color: '#ff7000', field: 'mistral_key',      placeholder: ''          },
  { id: 'perplexity',  name: 'Perplexity',       color: '#20b2aa', field: 'perplexity_key',   placeholder: 'pplx-…'   },
  { id: 'huggingface', name: 'HuggingFace',      color: '#ffd21e', field: 'huggingface_key',  placeholder: 'hf_…'     },
  { id: 'replicate',   name: 'Replicate',        color: '#3b82f6', field: 'replicate_key',    placeholder: 'r8_…'     },
]
const aiKeys = reactive<Record<string, string>>({})
const aiMsg  = ref('')

function saveAiKeys() {
  localStorage.setItem('omibus_ai_keys', JSON.stringify({ ...aiKeys }))
  aiMsg.value = 'Saved!'; setTimeout(() => aiMsg.value = '', 3000)
}

function loadAiKeys() {
  try {
    const saved = localStorage.getItem('omibus_ai_keys')
    if (saved) Object.assign(aiKeys, JSON.parse(saved))
  } catch {}
}

// ── Social ────────────────────────────────────────────────────────────────────
const socialPlatforms = [
  { id: 'telegram_token',   name: 'Telegram Bot Token', icon: '📱', color: '#0088cc', type: 'token'  },
  { id: 'telegram_chat_id', name: 'Telegram Chat ID',   icon: '💬', color: '#0088cc', type: 'id'     },
  { id: 'discord_webhook',  name: 'Discord Webhook',    icon: '🎮', color: '#5865f2', type: 'url'    },
  { id: 'twitter_handle',   name: 'X / Twitter',        icon: '🐦', color: '#1da1f2', type: 'handle' },
  { id: 'github_handle',    name: 'GitHub',              icon: '🐙', color: '#aaa',    type: 'handle' },
  { id: 'website',          name: 'Website',             icon: '🌐', color: '#3b82f6', type: 'url'    },
]
const socialKeys = reactive<Record<string, string>>({})
const socialMsg  = ref('')

function saveSocialKeys() {
  localStorage.setItem('omibus_social_keys', JSON.stringify({ ...socialKeys }))
  socialMsg.value = 'Saved!'; setTimeout(() => socialMsg.value = '', 3000)
}

function loadSocialKeys() {
  try {
    const saved = localStorage.getItem('omibus_social_keys')
    if (saved) Object.assign(socialKeys, JSON.parse(saved))
  } catch {}
}

// ── Export / Import (AES-GCM + PBKDF2 via Web Crypto) ────────────────────────
async function deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const enc = new TextEncoder()
  const mat = await crypto.subtle.importKey('raw', enc.encode(password), 'PBKDF2', false, ['deriveKey'])
  return crypto.subtle.deriveKey(
    { name: 'PBKDF2', salt: salt as unknown as ArrayBuffer, iterations: 200_000, hash: 'SHA-256' },
    mat, { name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt'])
}

async function encryptJson(obj: any, password: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(16))
  const iv   = crypto.getRandomValues(new Uint8Array(12))
  const key  = await deriveKey(password, salt)
  const ct   = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, new TextEncoder().encode(JSON.stringify(obj)))
  const buf  = new Uint8Array(16 + 12 + ct.byteLength)
  buf.set(salt, 0); buf.set(iv, 16); buf.set(new Uint8Array(ct), 28)
  return btoa(String.fromCharCode(...buf))
}

async function decryptJson(b64: string, password: string): Promise<any> {
  const buf  = Uint8Array.from(atob(b64), c => c.charCodeAt(0))
  const salt = buf.slice(0, 16); const iv = buf.slice(16, 28); const ct = buf.slice(28)
  const key  = await deriveKey(password, salt)
  const pt   = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, ct)
  return JSON.parse(new TextDecoder().decode(pt))
}

const pwModal  = ref<'export' | 'import' | null>(null)
const pwValue  = ref(''); const pwValue2 = ref('')
const pwFile   = ref<File | null>(null); const pwError = ref('')

function openExportModal() { pwModal.value = 'export'; pwValue.value = ''; pwValue2.value = ''; pwError.value = '' }
function closePwModal()    { pwModal.value = null; pwValue.value = ''; pwValue2.value = ''; pwFile.value = null; pwError.value = '' }

async function exportPlain() {
  const all = await invoke<any[]>('get_api_keys', {})
  const data = all.map(k => ({ exchange: k.exchange, label: k.label, api_key: k.api_key, api_secret: k.api_secret }))
  const ts   = new Date().toISOString().slice(0,16).replace('T','_').replace(':','-')
  download(JSON.stringify(data, null, 2), `OMIBUS-VAULT_${ts}.json`, 'application/json')
  keyMsg.value = `✓ Exported ${data.length} keys (plain JSON)`
  setTimeout(() => keyMsg.value = '', 4000)
}

async function confirmExport() {
  if (!pwValue.value)                     { pwError.value = 'Enter a password'; return }
  if (pwValue.value !== pwValue2.value)   { pwError.value = 'Passwords do not match'; return }
  try {
    const all  = await invoke<any[]>('get_api_keys', {})
    const data = all.map(k => ({ exchange: k.exchange, label: k.label, api_key: k.api_key, api_secret: k.api_secret }))
    const enc  = await encryptJson(data, pwValue.value)
    const ts   = new Date().toISOString().slice(0,16).replace('T','_').replace(':','-')
    download(enc, `OMIBUS-VAULT_${ts}.omnibus`, 'application/octet-stream')
    keyMsg.value = `✓ Exported ${data.length} keys (encrypted)`
    setTimeout(() => keyMsg.value = '', 4000)
    closePwModal()
  } catch (e) { pwError.value = 'Export error: ' + e }
}

function openImportModal() {
  const input = document.createElement('input')
  input.type = 'file'; input.accept = '.omnibus,.json'
  input.onchange = (e: any) => {
    const f = e.target.files[0]; if (!f) return
    pwFile.value = f
    if (f.name.endsWith('.json')) importPlainFile(f)
    else { pwModal.value = 'import'; pwValue.value = ''; pwError.value = '' }
  }
  input.click()
}

async function importPlainFile(file: File) {
  try {
    const ks = JSON.parse(await file.text()); if (!Array.isArray(ks)) { keyMsg.value = 'Invalid format'; return }
    let n = 0
    for (const k of ks) {
      if (!k.exchange || !k.api_key || !k.api_secret) continue
      await invoke('save_api_key', { exchange: k.exchange, label: k.label || 'Imported', apiKey: k.api_key, apiSecret: k.api_secret })
      n++
    }
    keyMsg.value = `✓ Imported ${n} keys`; await loadKeys()
    setTimeout(() => keyMsg.value = '', 4000)
  } catch (e) { keyMsg.value = 'Import error: ' + e }
}

async function confirmImport() {
  if (!pwValue.value) { pwError.value = 'Enter password'; return }
  if (!pwFile.value)  { pwError.value = 'No file'; return }
  try {
    let ks: any[]
    try { ks = await decryptJson(await pwFile.value.text(), pwValue.value) }
    catch { pwError.value = 'Wrong password or corrupted file'; return }
    if (!Array.isArray(ks)) { pwError.value = 'Invalid format'; return }
    let n = 0
    for (const k of ks) {
      if (!k.exchange || !k.api_key || !k.api_secret) continue
      await invoke('save_api_key', { exchange: k.exchange, label: k.label || 'Imported', apiKey: k.api_key, apiSecret: k.api_secret })
      n++
    }
    keyMsg.value = `✓ Imported ${n} keys`; await loadKeys()
    setTimeout(() => keyMsg.value = '', 4000)
    closePwModal()
  } catch (e) { pwError.value = 'Import error: ' + e }
}

function download(content: string, name: string, mime: string) {
  const a = document.createElement('a')
  a.href = URL.createObjectURL(new Blob([content], { type: mime }))
  a.download = name; a.click(); URL.revokeObjectURL(a.href)
}

onMounted(() => { loadKeys(); loadQuickPairs(); loadAiKeys(); loadSocialKeys() })
</script>

<style scoped>
.card { background: rgba(14,22,40,0.5); border: 1px solid rgba(59,130,246,0.1); border-radius: 8px; padding: 16px; }
.card-title { font-size: 14px; font-weight: 700; color: #e0e0e0; margin-bottom: 12px; }
.form-group { display: flex; flex-direction: column; gap: 4px; }
.form-group label { font-size: 11px; color: rgba(255,255,255,0.4); }
.form-input {
  padding: 7px 9px; background: rgba(15,20,35,0.8);
  border: 1px solid rgba(59,130,246,0.2); border-radius: 4px;
  color: #e0e0e0; font-size: 13px; outline: none;
}
.form-input:focus { border-color: rgba(59,130,246,0.6); }
.btn { padding: 7px 16px; border-radius: 5px; cursor: pointer; font-size: 12px; font-weight: 700; border: none; }
.btn-primary { background: #3b82f6; color: #fff; }
.btn-primary:disabled { opacity: .5; cursor: not-allowed; }
</style>
