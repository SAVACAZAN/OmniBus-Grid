import { ref, readonly } from 'vue'

export type Layout = '1' | '2h' | '2v' | '4'

export const LAYOUTS: { id: Layout; icon: string; label: string }[] = [
  { id: '1',  icon: '▣',   label: '1 chart' },
  { id: '2h', icon: '⬛⬛', label: '2 side by side' },
  { id: '2v', icon: '▬▬',  label: '2 stacked' },
  { id: '4',  icon: '⊞',   label: '4 charts' },
]

// ── Singleton state ───────────────────────────────────────────────────────────
const _layout        = ref<Layout>('1')
const _slotCount     = ref(1)   // tracked so App.vue can show "+ chart" only when < 6

export function useCharts() {
  function setLayout(l: Layout) { _layout.value = l }
  function setSlotCount(n: number) { _slotCount.value = n }

  return {
    layout:    readonly(_layout),
    slotCount: readonly(_slotCount),
    setLayout,
    setSlotCount,
  }
}
