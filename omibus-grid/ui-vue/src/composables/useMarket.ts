import { ref, computed, readonly } from 'vue'

export interface ApiKey { id: number; exchange: string; label: string; api_key_masked: string }

// ── Singleton state ───────────────────────────────────────────────────────────
// EX1 (blue)
const _exchange       = ref('kraken')
const _symbol         = ref('XBT/USD')
const _selectedKeyIds = ref<number[]>([])

// EX2 (purple) — optional second exchange
const _exchange2       = ref('')        // empty = disabled
const _symbol2         = ref('')
const _selectedKeyIds2 = ref<number[]>([])

const _allKeys      = ref<ApiKey[]>([])
const _quickVersion = ref(0)

export const KEY_COLORS = ['#3b82f6','#10eb04','#f59e0b','#a78bfa','#f87171','#34d399','#fb923c','#60a5fa']

export const DEFAULT_PAIRS: Record<string, string[]> = {
  kraken:      ['XBT/USD','XBT/EUR','ETH/USD','ETH/EUR','SOL/USD','ADA/USD','DOT/USD'],
  coinbase:    ['BTC-USD','BTC-EUR','ETH-USD','ETH-EUR','SOL-USD','DOGE-USD'],
  hyperliquid: ['BTC','ETH','SOL','AVAX','ARB','OP'],
}

export function useMarket() {
  // ── EX1 ──────────────────────────────────────────────────────────────────
  const exchangeKeys = computed(() =>
    _allKeys.value.filter(k => k.exchange === _exchange.value)
  )

  const quickPairs = computed(() => {
    _quickVersion.value
    const saved = localStorage.getItem(`quickPairs_${_exchange.value}`)
    return saved ? (JSON.parse(saved) as string[]) : (DEFAULT_PAIRS[_exchange.value] ?? [])
  })

  function setExchange(ex: string) {
    if (ex === _exchange.value) return
    _exchange.value = ex
    _symbol.value = quickPairsFor(ex)[0] ?? ''
    const keys = _allKeys.value.filter(k => k.exchange === ex)
    _selectedKeyIds.value = keys.length ? [keys[0].id] : []
    // prevent EX1 === EX2
    if (_exchange2.value === ex) _exchange2.value = ''
  }

  function setSymbol(sym: string) { _symbol.value = sym }

  function toggleKey(id: number) {
    const idx = _selectedKeyIds.value.indexOf(id)
    if (idx === -1) _selectedKeyIds.value.push(id)
    else _selectedKeyIds.value.splice(idx, 1)
  }

  // ── EX2 ──────────────────────────────────────────────────────────────────
  const exchangeKeys2 = computed(() =>
    _exchange2.value ? _allKeys.value.filter(k => k.exchange === _exchange2.value) : []
  )

  const quickPairs2 = computed(() => {
    _quickVersion.value
    if (!_exchange2.value) return []
    const saved = localStorage.getItem(`quickPairs_${_exchange2.value}`)
    return saved ? (JSON.parse(saved) as string[]) : (DEFAULT_PAIRS[_exchange2.value] ?? [])
  })

  function setExchange2(ex: string) {
    _exchange2.value = ex
    if (ex) {
      _symbol2.value = quickPairsFor(ex)[0] ?? ''
      const keys = _allKeys.value.filter(k => k.exchange === ex)
      _selectedKeyIds2.value = keys.length ? [keys[0].id] : []
    } else {
      _symbol2.value = ''
      _selectedKeyIds2.value = []
    }
  }

  function setSymbol2(sym: string) { _symbol2.value = sym }

  function toggleKey2(id: number) {
    const idx = _selectedKeyIds2.value.indexOf(id)
    if (idx === -1) _selectedKeyIds2.value.push(id)
    else _selectedKeyIds2.value.splice(idx, 1)
  }

  // ── Shared ────────────────────────────────────────────────────────────────
  function quickPairsFor(ex: string): string[] {
    const saved = localStorage.getItem(`quickPairs_${ex}`)
    return saved ? (JSON.parse(saved) as string[]) : (DEFAULT_PAIRS[ex] ?? [])
  }

  function setKeys(keys: ApiKey[]) {
    _allKeys.value = keys
    if (_selectedKeyIds.value.length === 0) {
      const mine = keys.filter(k => k.exchange === _exchange.value)
      if (mine.length) _selectedKeyIds.value = [mine[0].id]
    }
  }

  function bumpQuickPairs() { _quickVersion.value++ }

  return {
    // EX1
    exchange:       readonly(_exchange),
    symbol:         readonly(_symbol),
    selectedKeyIds: readonly(_selectedKeyIds),
    exchangeKeys,
    quickPairs,
    setExchange,
    setSymbol,
    toggleKey,
    // EX2
    exchange2:       readonly(_exchange2),
    symbol2:         readonly(_symbol2),
    selectedKeyIds2: readonly(_selectedKeyIds2),
    exchangeKeys2,
    quickPairs2,
    setExchange2,
    setSymbol2,
    toggleKey2,
    // shared
    allKeys: readonly(_allKeys),
    setKeys,
    bumpQuickPairs,
  }
}
