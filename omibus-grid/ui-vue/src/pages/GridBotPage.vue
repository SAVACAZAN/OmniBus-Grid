<template>
<div style="height:100%; display:grid; grid-template-columns:3fr 2fr; gap:2px; overflow:hidden;">

  <!-- LEFT: Form -->
  <div style="overflow-y:auto; padding:8px;">
    <!-- BOT MODE SELECTOR -->
    <div style="margin-bottom:6px; border:1px solid rgba(59,130,246,0.15); border-radius:4px; overflow:hidden;">
      <div style="display:grid; grid-template-columns:1fr 1fr 1fr 1fr; gap:0;">
        <button v-for="m in [{id:'classic',label:'⚡ Classic',color:'#3b82f6'},{id:'trailing',label:'📡 Trailing',color:'#a855f7'},{id:'pyramid',label:'🔺 Pyramid',color:'#10eb04'},{id:'pricegroup',label:'🎯 PriceGrp',color:'#ffa500'}]"
          :key="m.id"
          @click="form.botMode = m.id"
          :style="{
            padding:'5px 4px', border:'none', cursor:'pointer', fontSize:'9px', fontWeight:700,
            borderRight: m.id !== 'pricegroup' ? '1px solid rgba(59,130,246,0.15)' : 'none',
            background: form.botMode === m.id ? `rgba(${m.id==='classic'?'59,130,246':m.id==='trailing'?'168,85,247':m.id==='pyramid'?'16,235,4':'255,165,0'},0.18)` : 'rgba(26,31,46,0.8)',
            color: form.botMode === m.id ? m.color : 'rgba(255,255,255,0.35)',
            borderBottom: form.botMode === m.id ? `2px solid ${m.color}` : '2px solid transparent',
            transition:'all 0.15s'
          }">{{ m.label }}</button>
      </div>
      <!-- Pyramid config -->
      <div v-if="form.botMode === 'pyramid'" style="padding:8px; background:rgba(16,235,4,0.03); border-top:1px solid rgba(16,235,4,0.15); display:grid; grid-template-columns:1fr 1fr 1fr; gap:6px; font-size:9px;">
        <div>
          <div style="color:rgba(255,255,255,0.4); margin-bottom:2px; font-weight:700;">Type</div>
          <select v-model="form.pyramidType" class="form-input" style="width:100%; padding:2px 3px; font-size:9px; color:#10eb04; border-color:rgba(16,235,4,0.3);">
            <option value="linear">Linear (equal steps)</option>
            <option value="exponential">Exponential (×mult)</option>
          </select>
        </div>
        <div>
          <div style="color:rgba(255,255,255,0.4); margin-bottom:2px; font-weight:700;">Multiplier</div>
          <input type="number" v-model.number="form.pyramidMultiplier" step="0.1" min="1" max="5" class="form-input" style="width:100%; padding:2px 3px; font-size:10px; color:#10eb04; border-color:rgba(16,235,4,0.3);" :disabled="form.pyramidType==='linear'">
        </div>
      </div>
      <!-- Trailing config -->
      <div v-if="form.botMode === 'trailing'" style="padding:6px 8px; background:rgba(168,85,247,0.04); border-top:1px solid rgba(168,85,247,0.2); display:flex; align-items:center; gap:8px; font-size:9px;">
        <span style="color:rgba(168,85,247,0.7); font-weight:700;">Trigger when price exits range by</span>
        <input v-model.number="form.trailingPct" type="number" step="1" min="1" max="50" class="form-input" style="width:50px; padding:2px 4px; font-size:10px; color:#a855f7; border-color:rgba(168,85,247,0.3);">
        <span style="color:rgba(168,85,247,0.5);">%</span>
        <span style="color:rgba(255,255,255,0.25);">→ grid repositions around current price</span>
      </div>
    </div>

    <!-- Bot name + Type + Side + pair info -->
    <div style="display:flex; gap:4px; margin-bottom:6px; align-items:center;">
      <div style="display:flex; gap:0; height:28px;">
        <input v-model="form.name" class="form-input" style="width:110px; padding:2px 5px; font-size:10px; border-radius:4px 0 0 4px; border-right:none; height:100%;">
        <button @click="form.name = 'Grid_' + Math.random().toString(36).slice(2,10)" style="width:28px; height:100%; background:rgba(59,130,246,0.15); border:1px solid rgba(59,130,246,0.3); border-radius:0 4px 4px 0; color:#3b82f6; font-size:13px; cursor:pointer;" title="Random name">🎲</button>
      </div>
      <select v-model="form.amountType" class="form-input" style="padding:2px 4px; font-size:10px; height:28px; width:75px;"><option value="quantityPerGrid">Qty/Grid</option><option value="totalAmount">Total</option><option value="incrementalPercent">Inc%</option></select>
      <select v-model="form.gridType" class="form-input" style="padding:2px 4px; font-size:10px; height:28px; width:65px;"><option value="linear">Linear</option><option value="geometric">Geo</option><option value="pyramid">Pyramid</option></select>
      <select v-model="form.side" class="form-input" style="padding:2px 4px; font-size:10px; height:28px; width:65px;"><option value="buyOrSell">Both</option><option value="buyOnly">Buy</option><option value="sellOnly">Sell</option></select>
      <span class="rainbow-text" style="margin-left:auto; font-size:11px; font-weight:800; cursor:default;">{{ form.exchange }} / {{ form.symbol }}</span>
      <div @click="bid && (form.lower = bid.toFixed(5))" class="price-badge bid" style="padding:4px 10px; font-size:13px; font-weight:700; cursor:pointer;">
        <span style="font-size:9px; margin-right:3px;">BID</span>{{ bid?.toFixed(5) || '--' }}
      </div>
      <div @click="ask && (form.upper = ask.toFixed(5))" class="price-badge ask" style="padding:4px 10px; font-size:13px; font-weight:700; cursor:pointer;">
        <span style="font-size:9px; margin-right:3px;">ASK</span>{{ ask?.toFixed(5) || '--' }}
      </div>
    </div>

    <!-- Lower + Upper + Grids + Amount -->
    <div style="display:flex; gap:3px; margin-bottom:4px; align-items:flex-end;">
      <div style="width:100px; text-align:center;"><label style="font-size:10px; color:#10eb04; font-weight:600; display:block;">Lower</label><input type="number" v-model="form.lower" step="0.0001" class="form-input" style="padding:3px 4px; font-size:11px; border-color:rgba(16,235,4,0.3); color:#10eb04; width:100%;"></div>
      <div style="width:100px; text-align:center;"><label style="font-size:10px; color:#eb0404; font-weight:600; display:block;">Upper</label><input type="number" v-model="form.upper" step="0.0001" class="form-input" style="padding:3px 4px; font-size:11px; border-color:rgba(235,4,4,0.3); color:#eb0404; width:100%;"></div>
      <div style="width:60px; text-align:center;"><label style="font-size:10px; color:#3b82f6; font-weight:600; display:block;">Grids</label><input type="number" v-model.number="form.grids" min="1" max="200" class="form-input" style="padding:3px 5px; font-size:11px; width:100%; color:#3b82f6; border-color:rgba(59,130,246,0.3);"></div>
      <div style="width:110px; text-align:center;">
        <label style="font-size:10px; color:#facc15; font-weight:600; display:block;">Amount <span style="opacity:0.6;">({{ form.amountUnit === 'base' ? symBase : symQuote }})</span></label>
        <input type="number" v-model.number="form.amount" class="form-input" style="padding:3px 5px; font-size:11px; width:100%; color:#facc15; border-color:rgba(250,204,21,0.3);">
        <div style="display:flex; gap:2px; margin-top:2px;">
          <button type="button" @click="form.amountUnit='base'"
            :style="{flex:1, padding:'3px 2px', fontSize:'9px', fontWeight:700, borderRadius:'3px', cursor:'pointer',
                     background: form.amountUnit==='base' ? 'rgba(250,204,21,0.2)' : 'rgba(255,255,255,0.04)',
                     color: form.amountUnit==='base' ? '#facc15' : 'rgba(255,255,255,0.35)',
                     border: '1px solid '+ (form.amountUnit==='base' ? 'rgba(250,204,21,0.5)' : 'rgba(255,255,255,0.1)')}">
            💰 {{ symBase }}
          </button>
          <button type="button" @click="form.amountUnit='quote'"
            :style="{flex:1, padding:'3px 2px', fontSize:'9px', fontWeight:700, borderRadius:'3px', cursor:'pointer',
                     background: form.amountUnit==='quote' ? 'rgba(250,204,21,0.2)' : 'rgba(255,255,255,0.04)',
                     color: form.amountUnit==='quote' ? '#facc15' : 'rgba(255,255,255,0.35)',
                     border: '1px solid '+ (form.amountUnit==='quote' ? 'rgba(250,204,21,0.5)' : 'rgba(255,255,255,0.1)')}">
            💵 {{ symQuote }}
          </button>
        </div>
      </div>
      <div v-if="statusMsg" class="status-bar"
        :style="{
          height: '28px', padding:'0 10px', display:'inline-flex', alignItems:'center', justifyContent:'center', marginBottom:'4px',
          borderRadius:'14px', fontSize:'10px', fontWeight:700, color:'#fff', cursor:'default', whiteSpace:'nowrap',
          background: statusType==='error' ? 'rgba(235,4,4,0.3)' :
            statusBuyPct >= 0 ? `linear-gradient(90deg, rgba(16,235,4,0.3) 0%, rgba(16,235,4,0.25) ${statusBuyPct}%, rgba(235,4,4,0.25) ${statusBuyPct}%, rgba(235,4,4,0.3) 100%)` :
            'rgba(16,235,4,0.2)',
        }">{{ statusMsg }}</div>
    </div>

    <!-- Amount analysis -->
    <div v-if="amountAnalysis" :style="{
      padding:'4px 8px', marginBottom:'4px', fontSize:'9px', borderRadius:'3px', display:'flex', gap:'12px', alignItems:'center', flexWrap:'wrap',
      background: amountAnalysis.warning ? 'rgba(235,4,4,0.1)' : 'rgba(0,0,0,0.25)',
      borderLeft: '2px solid ' + (amountAnalysis.warning ? '#eb0404' : '#2a3441')
    }">
      <span style="color:rgba(255,255,255,0.5);">Total:</span>
      <span style="font-family:monospace; color:#ccc;">{{ amountAnalysis.totalBase.toFixed(4) }} {{ symBase }} / {{ amountAnalysis.totalQuote.toFixed(2) }} {{ symQuote }}</span>
      <template v-if="amountAnalysis.isIncremental">
        <span style="color:rgba(255,255,255,0.5);">First:</span>
        <span style="font-family:monospace; color:#ccc;">{{ amountAnalysis.firstBase.toFixed(4) }} {{ symBase }} / {{ amountAnalysis.firstQuote.toFixed(2) }} {{ symQuote }}</span>
        <span style="color:rgba(255,255,255,0.5);">Last:</span>
        <span style="font-family:monospace; color:#ccc;">{{ amountAnalysis.lastBase.toFixed(4) }} {{ symBase }} / {{ amountAnalysis.lastQuote.toFixed(2) }} {{ symQuote }}</span>
      </template>
      <template v-else>
        <span style="color:rgba(255,255,255,0.5);">Per order:</span>
        <span style="font-family:monospace; color:#ccc;">{{ amountAnalysis.firstBase.toFixed(4) }} {{ symBase }} / {{ amountAnalysis.firstQuote.toFixed(2) }} {{ symQuote }}</span>
      </template>
      <span v-if="amountAnalysis.warning" style="color:#ff5252; font-weight:700;">⚠ {{ amountAnalysis.warning }}</span>
      <span v-if="amountAnalysis.minRequired" style="color:#10eb04; font-weight:700;">👉 {{ amountAnalysis.minRequired }}</span>
    </div>

    <!-- Deviation table -->
    <table style="border-collapse:collapse; margin-bottom:6px; font-size:9px; border:1px solid rgba(59,130,246,0.15); border-radius:4px;"><tbody>
      <tr>
        <td style="width:45px;"></td>
        <td style="padding:3px 8px; background:rgba(16,235,4,0.1); color:#10eb04; font-weight:700; text-align:center; border:1px solid rgba(59,130,246,0.08);">BUY</td>
        <td style="padding:3px 8px; background:rgba(235,4,4,0.1); color:#eb0404; font-weight:700; text-align:center; border:1px solid rgba(59,130,246,0.08);">SELL</td>
      </tr>
      <tr>
        <td class="dev-label">INC%</td>
        <td class="dev-buy"><input v-model="form.incBuy" type="number" step="0.1" class="form-input dev-input-buy"></td>
        <td class="dev-sell"><input v-model="form.incSell" type="number" step="0.1" class="form-input dev-input-sell"></td>
      </tr>
      <template v-if="form.botMode === 'classic' || form.botMode === 'trailing'">
        <tr>
          <td class="dev-label">DevP%</td>
          <td class="dev-buy"><input v-model="form.devPriceBuy" type="number" step="0.1" class="form-input dev-input-buy"></td>
          <td class="dev-sell"><input v-model="form.devPriceSell" type="number" step="0.1" class="form-input dev-input-sell"></td>
        </tr>
        <tr>
          <td class="dev-label">DevA%</td>
          <td class="dev-buy"><input v-model="form.devAmtBuy" type="number" step="0.1" class="form-input dev-input-buy"></td>
          <td class="dev-sell"><input v-model="form.devAmtSell" type="number" step="0.1" class="form-input dev-input-sell"></td>
        </tr>
      </template>
      <template v-if="form.botMode === 'pricegroup'">
        <tr style="background:rgba(255,165,0,0.08)">
          <td class="dev-label" style="color:#ffa500;">Buy@</td>
          <td class="dev-buy" colspan="2"><input v-model="form.priceGroupBuy" placeholder="group buy price" class="form-input dev-input-buy" style="font-size:9px; width:100%;"></td>
        </tr>
        <tr style="background:rgba(255,165,0,0.08)">
          <td class="dev-label" style="color:#ffa500;">Sell@</td>
          <td class="dev-sell" colspan="2"><input v-model="form.priceGroupSell" placeholder="group sell price" class="form-input dev-input-sell" style="font-size:9px; width:100%;"></td>
        </tr>
        <tr style="background:rgba(255,165,0,0.04)">
          <td colspan="3" style="padding:3px 6px; font-size:8px; color:rgba(255,165,0,0.5);">
            All N grid orders → 2 group orders. Each fill accumulates amount into opposite group.
          </td>
        </tr>
      </template>
    </tbody></table>

    <!-- Buttons -->
    <div style="display:flex; gap:4px; flex-wrap:wrap; align-items:center;">
      <button class="btn btn-primary" @click="handleCreate" :disabled="placing" style="padding:6px 12px; font-size:11px;">{{ placing ? 'PLACING...' : '⚙ CREATE' }}</button>
      <button v-if="placing" @click="stopPlacement"
        style="padding:6px 12px; font-size:11px; font-weight:800; background:rgba(235,4,4,0.25); color:#eb0404; border:1px solid rgba(235,4,4,0.5); border-radius:6px; cursor:pointer; animation: pulse-red 1s infinite;">
        ■ STOP
      </button>
      <button v-if="!placing" @click="form.side='buyOnly'; handleCreate()" style="padding:6px 10px; font-size:11px; background:rgba(16,235,4,0.2); color:#10eb04; border:1px solid rgba(16,235,4,0.3); border-radius:6px; cursor:pointer; font-weight:700;">💚 BUY</button>
      <button v-if="!placing" @click="form.side='sellOnly'; handleCreate()" style="padding:6px 10px; font-size:11px; background:rgba(235,4,4,0.2); color:#eb0404; border:1px solid rgba(235,4,4,0.3); border-radius:6px; cursor:pointer; font-weight:700;">🔴 SELL</button>
      <button class="btn btn-demo" @click="togglePreview" style="padding:6px 12px; font-size:11px;">🎯 {{ preview ? 'CLOSE' : 'PREVIEW' }}</button>
      <button v-if="!placing" class="btn btn-reset" @click="resetForm" style="padding:6px 12px; font-size:11px;">🔄 RESET</button>
    </div>

    <!-- Stop dialog -->
    <div v-if="stopDialog.visible"
      style="position:fixed; inset:0; z-index:9999; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.6);">
      <div style="background:#0d1117; border:1px solid rgba(235,4,4,0.4); border-radius:10px; padding:24px 28px; min-width:340px; box-shadow:0 8px 40px rgba(0,0,0,0.7);">
        <div style="font-size:13px; font-weight:800; color:#eb0404; margin-bottom:6px;">■ Placement stopped</div>
        <div style="font-size:11px; color:rgba(255,255,255,0.6); margin-bottom:16px; line-height:1.5;">
          <b style="color:#10eb04;">{{ stopDialog.placed }}</b> orders placed on exchange before stopping.<br>
          What do you want to do with them?
        </div>
        <div style="display:flex; gap:8px; flex-direction:column;">
          <button @click="stopDialogCancelOrders"
            style="padding:8px 16px; font-size:11px; font-weight:700; background:rgba(235,4,4,0.2); color:#eb0404; border:1px solid rgba(235,4,4,0.4); border-radius:6px; cursor:pointer; text-align:left;">
            ✗ Cancel placed orders on exchange
            <span style="display:block; font-size:9px; font-weight:400; color:rgba(255,255,255,0.35); margin-top:2px;">Sends cancel request for each placed order</span>
          </button>
          <button @click="stopDialogKeepOrders"
            style="padding:8px 16px; font-size:11px; font-weight:700; background:rgba(250,204,21,0.1); color:#facc15; border:1px solid rgba(250,204,21,0.3); border-radius:6px; cursor:pointer; text-align:left;">
            ▐ Just stop — keep placed orders on exchange
            <span style="display:block; font-size:9px; font-weight:400; color:rgba(255,255,255,0.35); margin-top:2px;">Bot stays in DB with {{ stopDialog.placed }} orders</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Placement progress bar + live log -->
    <div v-if="placing || placingLog.length > 0" style="margin-top:4px; padding:6px 8px; background:rgba(59,130,246,0.1); border:1px solid rgba(59,130,246,0.3); border-radius:6px;">
      <div style="display:flex; justify-content:space-between; font-size:10px; margin-bottom:4px;">
        <span style="color:#3b82f6; font-weight:700;">{{ placing ? 'Placing orders on exchange...' : 'Placement complete' }}</span>
        <span style="font-weight:700;">
          <span style="color:#10eb04;">{{ placingProgress.placed }} OK</span>
          <span v-if="placingProgress.failed > 0" style="color:#eb0404; margin-left:6px;">{{ placingProgress.failed }} FAIL</span>
          <span style="color:rgba(255,255,255,0.3); margin-left:6px;">{{ placingProgress.current }}/{{ placingProgress.total }}</span>
        </span>
      </div>
      <div v-if="placing" style="height:6px; background:rgba(26,31,46,0.8); border-radius:3px; overflow:hidden; margin-bottom:4px;">
        <div :style="{height:'100%', borderRadius:'3px', transition:'width 0.3s', width: (placingProgress.total > 0 ? (placingProgress.current / placingProgress.total * 100) : 0) + '%', background:'linear-gradient(90deg, #3b82f6, #10eb04)'}"></div>
      </div>
      <div style="max-height:120px; overflow-y:auto; font-size:9px; font-family:monospace; background:rgba(14,22,40,0.6); border-radius:4px; padding:3px 5px;">
        <div v-for="(log, li) in placingLog" :key="li" style="display:flex; gap:4px; padding:1px 0; border-bottom:1px solid rgba(59,130,246,0.05);">
          <span style="color:rgba(255,255,255,0.3); width:22px; text-align:right;">{{ log.i }}</span>
          <span :style="{ color: log.side === 'buy' ? '#10eb04' : '#eb0404', fontWeight: '700', width: '24px' }">{{ log.side === 'buy' ? 'BUY' : 'SELL' }}</span>
          <span style="color:rgba(255,255,255,0.5); width:65px;">{{ typeof log.price === 'number' ? log.price.toFixed(5) : log.price }}</span>
          <span style="color:rgba(255,255,255,0.3); width:50px;">{{ typeof log.amount === 'number' ? log.amount.toFixed(2) : log.amount }}</span>
          <span v-if="log.status === 'OK'" style="color:#10eb04; flex:1;">OK {{ log.order_id ? log.order_id.slice(0,12) + '...' : '' }}</span>
          <span v-else-if="log.status === 'SKIP'" style="color:#666; flex:1;">SKIP</span>
          <span v-else style="color:#eb0404; flex:1;">FAIL: {{ log.error }}</span>
        </div>
      </div>
      <button v-if="!placing && placingLog.length > 0" @click="placingLog = []" style="margin-top:3px; padding:2px 8px; font-size:8px; background:rgba(100,100,100,0.2); color:#666; border:1px solid rgba(100,100,100,0.3); border-radius:3px; cursor:pointer;">Clear log</button>
    </div>

    <!-- Grid Strategies (inline stub) -->
    <div style="margin-top:6px; padding:6px 8px; background:rgba(26,31,46,0.4); border:1px solid rgba(59,130,246,0.15); border-radius:4px; font-size:9px;">
      <div style="display:flex; gap:6px; align-items:center; flex-wrap:wrap;">
        <span style="color:rgba(59,130,246,0.6); font-weight:700;">📋 STRATEGIES</span>
        <button @click="handleStrategySave" class="price-btn" style="padding:2px 6px; font-size:9px;">💾 Save current</button>
        <template v-for="(s, si) in strategies" :key="si">
          <button @click="loadStrategy(s)" class="price-btn" style="padding:2px 6px; font-size:9px; color:#10eb04; border-color:rgba(16,235,4,0.3);">{{ s.name }}</button>
          <button @click="strategies.splice(si,1); saveStrategies()" style="padding:2px 4px; font-size:9px; background:none; border:none; color:rgba(235,4,4,0.5); cursor:pointer;">✗</button>
        </template>
      </div>
    </div>

    <!-- Quick Actions (inline) -->
    <div style="margin-top:6px; padding:6px 8px; background:rgba(26,31,46,0.4); border:1px solid rgba(250,204,21,0.2); border-radius:4px; font-size:9px;">
      <div style="color:rgba(250,204,21,0.7); font-weight:700; margin-bottom:6px;">💰 QUICK ACTIONS</div>
      <div style="display:flex; gap:4px; flex-wrap:wrap; align-items:center;">
        <span style="color:rgba(255,255,255,0.3);">Lower:</span>
        <button v-for="p in [-90,-70,-50,-30,-20,-10,-5,-1]" :key="'l'+p" class="price-btn" style="padding:2px 5px; font-size:9px;" @click="form.lower=((bid||0)*(1+p/100)).toFixed(5)">{{ p }}%</button>
        <span style="color:rgba(255,255,255,0.3); margin-left:6px;">Upper:</span>
        <button v-for="p in [1,5,10,20,30,50,70,90]" :key="'u'+p" class="price-btn" style="padding:2px 5px; font-size:9px;" @click="form.upper=((ask||0)*(1+p/100)).toFixed(5)">+{{ p }}%</button>
      </div>
      <div style="display:flex; gap:4px; flex-wrap:wrap; align-items:center; margin-top:4px;">
        <span style="color:rgba(255,255,255,0.3);">Grids:</span>
        <button v-for="g in [5,10,15,20,30,50]" :key="'g'+g" class="price-btn" style="padding:2px 5px; font-size:9px;" @click="form.grids=g">{{ g }}</button>
        <span style="color:rgba(255,255,255,0.3); margin-left:6px;">INC B/S:</span>
        <button v-for="v in [0,0.5,1,1.5,2,3]" :key="'i'+v" class="price-btn" style="padding:2px 5px; font-size:9px;" @click="form.incBuy=String(v); form.incSell=String(v)">{{ v }}</button>
      </div>
    </div>
  </div>

  <!-- RIGHT: Preview + Chart + Active Bots -->
  <div style="overflow:hidden; padding:8px; display:flex; flex-direction:column; gap:4px;">

    <!-- Grid Preview -->
    <div v-if="preview" class="r-panel" :class="{collapsed: !showPreview}" style="border-color:rgba(59,130,246,0.2);">
      <div class="r-panel-header" @click="showPreview=!showPreview">
        <span class="r-arrow">{{ showPreview ? '▼' : '▶' }}</span>
        <span style="color:rgba(59,130,246,0.7);">GRID PREVIEW</span>
        <button @click.stop="exportCSV" class="price-btn" style="margin-left:auto; padding:2px 6px; font-size:9px;">💾 CSV</button>
      </div>
      <div v-if="showPreview" class="r-panel-body">
        <div style="display:grid; grid-template-columns:repeat(4, 1fr); gap:3px; margin-bottom:3px; font-size:9px;">
          <div style="text-align:center; padding:3px; background:rgba(26,31,46,0.6); border-radius:4px;"><div style="color:rgba(59,130,246,0.4);">ORDERS</div><div style="font-size:13px; font-weight:800; color:#3b82f6;">{{ previewStats.activeBuys + previewStats.activeSells }}</div></div>
          <div style="text-align:center; padding:3px; background:rgba(26,31,46,0.6); border-radius:4px;"><div style="color:rgba(16,235,4,0.4);">BUY</div><div style="font-size:13px; font-weight:800; color:#10eb04;">{{ previewStats.activeBuys }}</div></div>
          <div style="text-align:center; padding:3px; background:rgba(26,31,46,0.6); border-radius:4px;"><div style="color:rgba(235,4,4,0.4);">SELL</div><div style="font-size:13px; font-weight:800; color:#eb0404;">{{ previewStats.activeSells }}</div></div>
          <div style="text-align:center; padding:3px; background:rgba(26,31,46,0.6); border-radius:4px;"><div style="color:rgba(250,204,21,0.4);">SPREAD/G</div><div style="font-weight:700; color:#facc15;">{{ previewStats.spreadPerGrid }}</div></div>
        </div>
        <table style="width:100%; border-collapse:collapse; margin-bottom:4px; font-size:9px;">
          <tbody>
            <tr><td></td><td style="text-align:center; color:#10eb04; font-weight:700; background:rgba(16,235,4,0.06);">BUY</td><td style="text-align:center; color:#eb0404; font-weight:700; background:rgba(235,4,4,0.06);">SELL</td><td class="need-header" style="text-align:center; font-weight:800;">NEED</td><td style="text-align:center; color:#a855f7; font-weight:700;">PROFIT</td></tr>
            <tr><td style="color:rgba(255,255,255,0.4);">{{ pairQuote }}</td><td style="text-align:center; color:#10eb04;">{{ previewStats.buyQuote }}</td><td style="text-align:center; color:#eb0404;">{{ previewStats.sellQuote }}</td><td class="need-cell" style="text-align:center; font-weight:800;">{{ previewStats.buyQuote }} <span :style="{color: hasEnoughQuote ? '#10eb04' : '#eb0404'}">{{ hasEnoughQuote ? '✔' : '✘' }}</span></td><td style="text-align:center;" :style="{color: previewStats.profitQuoteRaw > 0 ? '#10eb04' : '#eb0404'}">{{ previewStats.profitQuote }}</td></tr>
            <tr><td style="color:rgba(255,255,255,0.4);">{{ pairBase }}</td><td style="text-align:center; color:#10eb04;">{{ previewStats.buyBase }}</td><td style="text-align:center; color:#eb0404;">{{ previewStats.sellBase }}</td><td class="need-cell" style="text-align:center; font-weight:800;">{{ previewStats.sellBase }} <span :style="{color: hasEnoughBase ? '#10eb04' : '#eb0404'}">{{ hasEnoughBase ? '✔' : '✘' }}</span></td><td style="text-align:center;" :style="{color: previewStats.profitBaseRaw > 0 ? '#10eb04' : '#eb0404'}">{{ previewStats.profitBase }}</td></tr>
          </tbody>
        </table>
        <div style="display:grid; grid-template-columns:1fr 1fr 1fr; gap:3px; margin-bottom:4px; font-size:9px;">
          <div style="text-align:center; padding:2px; background:rgba(16,235,4,0.04); border:1px solid rgba(16,235,4,0.1); border-radius:3px;"><div style="color:rgba(16,235,4,0.4);">AVG BUY</div><div style="font-weight:700; color:#10eb04;">{{ previewStats.avgBuy }}</div></div>
          <div style="text-align:center; padding:2px; background:rgba(235,4,4,0.04); border:1px solid rgba(235,4,4,0.1); border-radius:3px;"><div style="color:rgba(235,4,4,0.4);">AVG SELL</div><div style="font-weight:700; color:#eb0404;">{{ previewStats.avgSell }}</div></div>
          <div style="text-align:center; padding:2px; background:rgba(168,85,247,0.06); border:1px solid rgba(168,85,247,0.15); border-radius:3px;"><div style="color:rgba(168,85,247,0.5);">PROFIT</div><div style="font-weight:700;" :style="{color: previewStats.profitQuoteRaw > 0 ? '#10eb04' : '#eb0404'}">{{ previewStats.profitQuote }} ({{ previewStats.profitPct }}%)</div></div>
        </div>
        <!-- color bar -->
        <div style="height:16px; margin-bottom:4px; border-radius:3px; overflow:hidden; display:flex; background:rgba(26,31,46,0.6);">
          <div v-for="ord in preview.orders" :key="'v'+ord.index" :style="{flex:1, background: ord.side==='buy' ? 'rgba(16,235,4,0.3)' : 'rgba(235,4,4,0.3)', borderRight:'1px solid rgba(0,0,0,0.2)'}" :title="ord.side.toUpperCase()+' @ '+ord.price?.toFixed(5)"></div>
        </div>
        <!-- Allocation buttons -->
        <div style="display:flex; gap:3px; margin-bottom:4px; flex-wrap:wrap; align-items:center;">
          <div style="display:flex; gap:1px; align-items:center; background:rgba(250,204,21,0.05); border:1px solid rgba(250,204,21,0.2); border-radius:4px; padding:2px 4px;">
            <span style="font-size:7px; color:rgba(250,204,21,0.6); font-weight:700; margin-right:2px;">BANK</span>
            <button v-for="p in [10,20,30]" :key="'bk'+p" class="price-btn" :style="{padding:'1px 4px', fontSize:'8px', color: allocBank===p ? '#facc15' : 'rgba(250,204,21,0.4)', borderColor: allocBank===p ? 'rgba(250,204,21,0.4)' : 'rgba(250,204,21,0.15)', background: allocBank===p ? 'rgba(250,204,21,0.1)' : 'transparent'}" @click="allocBank = allocBank===p ? 0 : p">{{p}}%</button>
          </div>
          <div style="display:flex; gap:1px; align-items:center; background:rgba(59,130,246,0.05); border:1px solid rgba(59,130,246,0.2); border-radius:4px; padding:2px 4px;">
            <span style="font-size:7px; color:rgba(59,130,246,0.6); font-weight:700; margin-right:2px;">B/T</span>
            <button v-for="p in [10,20,30]" :key="'bt'+p" class="price-btn" :style="{padding:'1px 4px', fontSize:'8px', color: allocBT===p ? '#3b82f6' : 'rgba(59,130,246,0.4)', borderColor: allocBT===p ? 'rgba(59,130,246,0.4)' : 'rgba(59,130,246,0.15)', background: allocBT===p ? 'rgba(59,130,246,0.1)' : 'transparent'}" @click="allocBT = allocBT===p ? 0 : p">{{p}}%</button>
          </div>
          <div style="display:flex; gap:1px; align-items:center; background:rgba(16,235,4,0.03); border:1px solid rgba(16,235,4,0.15); border-radius:4px; padding:2px 4px;">
            <span style="font-size:7px; color:rgba(16,235,4,0.5); font-weight:700; margin-right:2px;">FIB</span>
            <button v-for="p in [10,20,30]" :key="'fb'+p" class="price-btn" :style="{padding:'1px 4px', fontSize:'8px', color: allocFib===p ? '#10eb04' : 'rgba(16,235,4,0.4)', borderColor: allocFib===p ? 'rgba(16,235,4,0.3)' : 'rgba(16,235,4,0.12)', background: allocFib===p ? 'rgba(16,235,4,0.1)' : 'transparent'}" @click="allocFib = allocFib===p ? 0 : p">{{p}}%</button>
          </div>
          <span v-if="allocTotal > 0" style="font-size:8px; font-weight:700; margin-left:2px;" :style="{color: allocTotal <= 100 ? '#facc15' : '#eb0404'}">= {{allocTotal}}%</span>
        </div>
        <!-- Orders list -->
        <div style="font-size:10px;">
          <div style="display:flex; gap:6px; padding:2px 0; color:rgba(59,130,246,0.4); font-weight:700; font-size:9px; border-bottom:1px solid rgba(59,130,246,0.1); position:sticky; top:0; background:rgba(20,25,30,0.95);">
            <span style="width:20px;">#</span><span style="width:30px;">S</span><span style="flex:1;">Price</span><span style="flex:1; text-align:right;">Amt</span><span style="flex:1; text-align:right;">Total</span>
          </div>
          <div v-for="ord in allocatedOrders" :key="ord.side+ord.index"
            :style="{display:'flex', gap:'6px', padding:'1px 0', borderBottom:'1px solid rgba(59,130,246,0.03)',
              background: ord.allocStatus==='bank' ? 'rgba(250,204,21,0.06)' : ord.side==='buy' ? `rgba(16,235,4,${0.02 + (ord.amount / maxOrderSize) * 0.12})` : `rgba(235,4,4,${0.02 + (ord.amount / maxOrderSize) * 0.12})`}">
            <span style="width:20px; color:rgba(255,255,255,0.2);">{{ ord.index }}</span>
            <span style="width:30px;" :style="{color: ord.side==='buy' ? '#10eb04' : '#eb0404', fontWeight:700}">{{ ord.side==='buy' ? 'B' : 'S' }}</span>
            <span style="flex:1; font-family:monospace;">{{ ord.price?.toFixed(5) }}</span>
            <span style="flex:1; text-align:right; font-family:monospace;">{{ ord.amount?.toFixed(2) }}</span>
            <span style="flex:1; text-align:right; font-family:monospace; color:rgba(255,255,255,0.4);">{{ ord.total?.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="r-panel" style="border-color:rgba(59,130,246,0.1); justify-content:center; align-items:center; color:rgba(255,255,255,0.2); font-size:12px;">
      Click PREVIEW to see grid orders
    </div>

    <!-- Chart panel (TradingView iframe) -->
    <div class="r-panel" :class="{collapsed: !showChart}" style="border-color:rgba(59,130,246,0.2);">
      <div class="r-panel-header" @click="showChart=!showChart">
        <span class="r-arrow">{{ showChart ? '▼' : '▶' }}</span>
        <span style="color:rgba(59,130,246,0.7);">CHART</span>
      </div>
      <div v-if="showChart" style="flex:1; min-height:200px; overflow:hidden;">
        <iframe ref="chartIframe"
          :src="chartUrl"
          style="width:100%; height:100%; border:none; background:#0d1117;"
          allow="fullscreen" />
      </div>
    </div>

    <!-- Active Bots -->
    <div class="r-panel" :class="{collapsed: !showBots}" style="border-color:rgba(16,235,4,0.2);">
      <div class="r-panel-header" @click="showBots=!showBots">
        <span class="r-arrow" style="color:rgba(16,235,4,0.5);">{{ showBots ? '▼' : '▶' }}</span>
        <span style="color:#10eb04;">ACTIVE BOTS ({{ bots.length }})</span>
      </div>
      <div v-if="showBots" class="r-panel-body">
        <div v-if="bots.length === 0" style="text-align:center; padding:12px; color:rgba(255,255,255,0.2); font-size:11px;">No bots yet</div>
        <div v-for="bot in bots" :key="bot.id" style="margin-bottom:4px; background:rgba(26,31,46,0.4); border-radius:4px; font-size:10px;">
          <div @click="expandedBot = expandedBot === bot.id ? null : bot.id" style="padding:6px; display:flex; align-items:center; gap:8px; cursor:pointer;">
            <span style="font-size:9px; color:rgba(59,130,246,0.4);">{{ expandedBot === bot.id ? '▼' : '▶' }}</span>
            <div style="flex:1;">
              <div style="font-weight:700; color:#3b82f6;">{{ bot.name }}</div>
              <div style="color:rgba(255,255,255,0.3); font-size:9px;">{{ bot.symbol }} | {{ bot.nr_of_grids }}g</div>
            </div>
            <span :style="{padding:'1px 6px', borderRadius:'3px', fontSize:'8px', fontWeight:700, background: bot.status==='running' ? 'rgba(16,235,4,0.15)' : 'rgba(100,100,100,0.15)', color: bot.status==='running' ? '#10eb04' : '#666'}">{{ bot.status }}</span>
            <button v-if="bot.status==='running'" @click.stop="stopBot(bot.id)" style="padding:2px 6px; background:rgba(235,4,4,0.2); color:#eb0404; border:1px solid rgba(235,4,4,0.3); border-radius:3px; cursor:pointer; font-size:9px;">Stop</button>
            <button v-else @click.stop="startBot(bot.id)" style="padding:2px 6px; background:rgba(16,235,4,0.2); color:#10eb04; border:1px solid rgba(16,235,4,0.3); border-radius:3px; cursor:pointer; font-size:9px;">Start</button>
            <button @click.stop="cancelBotOrders(bot.id, false)" style="padding:2px 6px; background:rgba(250,204,21,0.2); color:#facc15; border:1px solid rgba(250,204,21,0.3); border-radius:3px; cursor:pointer; font-size:9px;">Cancel All</button>
            <button @click.stop="cancelBotOrders(bot.id, true)" style="padding:2px 6px; background:rgba(235,4,4,0.2); color:#eb0404; border:1px solid rgba(235,4,4,0.3); border-radius:3px; cursor:pointer; font-size:9px;">Cancel+Del</button>
            <button @click.stop="removeBot(bot.id)" style="padding:2px 6px; background:rgba(100,100,100,0.2); color:#666; border:1px solid rgba(100,100,100,0.3); border-radius:3px; cursor:pointer; font-size:9px;">Del</button>
          </div>
          <div v-if="expandedBot === bot.id" style="padding:4px 8px 8px; border-top:1px solid rgba(59,130,246,0.1); font-size:9px;">
            <div style="display:grid; grid-template-columns:repeat(4,1fr); gap:3px; margin-bottom:4px;">
              <div style="text-align:center; padding:2px; background:rgba(26,31,46,0.6); border-radius:3px;"><div style="font-size:7px; color:rgba(16,235,4,0.4);">LIVE</div><div style="font-weight:700; color:#10eb04;">{{ parseBotOrders(bot, 'active').filter((o:any) => o.id && !o.error).length }}</div></div>
              <div style="text-align:center; padding:2px; background:rgba(26,31,46,0.6); border-radius:3px;"><div style="font-size:7px; color:rgba(235,4,4,0.4);">FAILED</div><div style="font-weight:700; color:#eb0404;">{{ parseBotOrders(bot, 'active').filter((o:any) => o.error).length }}</div></div>
              <div style="text-align:center; padding:2px; background:rgba(26,31,46,0.6); border-radius:3px;"><div style="font-size:7px; color:rgba(59,130,246,0.4);">FILLED</div><div style="font-weight:700; color:#3b82f6;">{{ parseBotOrders(bot, 'filled').length }}</div></div>
              <div style="text-align:center; padding:2px; background:rgba(26,31,46,0.6); border-radius:3px;"><div style="font-size:7px; color:rgba(250,204,21,0.4);">PROFIT</div><div style="font-weight:700; color:#facc15;">~</div></div>
            </div>
            <div v-for="(ord, oi) in parseBotOrders(bot, 'active')" :key="'ao'+oi" style="display:flex; gap:4px; padding:1px 0; border-bottom:1px solid rgba(59,130,246,0.03);">
              <span :style="{ color: ord.side === 'buy' ? '#10eb04' : '#eb0404', fontWeight: '700', width: '20px' }">{{ ord.side === 'buy' ? 'B' : 'S' }}</span>
              <span style="flex:1; font-family:monospace;">{{ typeof ord.price === 'number' ? ord.price.toFixed(5) : ord.price }}</span>
              <span style="flex:1; text-align:right; font-family:monospace;">{{ typeof ord.amount === 'number' ? ord.amount.toFixed(2) : ord.amount }}</span>
              <span v-if="ord.id && !ord.error" style="color:#10eb04; font-size:8px; width:80px; text-align:right;">{{ ord.id.slice(0,8) }}...</span>
              <span v-else-if="ord.error" style="color:#eb0404; font-size:8px; width:80px; text-align:right;" :title="ord.error">FAIL</span>
              <span v-else style="color:#666; font-size:8px; width:80px; text-align:right;">pending</span>
            </div>
          </div>
        </div>
      </div>
    </div>

  </div>

</div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMarket } from '../composables/useMarket'

const { exchange: mktExchange, symbol: mktSymbol } = useMarket()

const form = reactive({
  name: 'Grid_' + Math.random().toString(36).slice(2, 7),
  symbol: mktSymbol.value || 'BTC/USD',
  exchange: mktExchange.value || 'kraken',
  lower: '', upper: '', grids: 20, amount: 2,
  amountUnit: 'quote' as 'base' | 'quote',
  amountType: 'incrementalPercent', side: 'buyOrSell',
  incBuy: '1', incSell: '1', devPriceBuy: '1', devPriceSell: '1', devAmtBuy: '0.9', devAmtSell: '0.9',
  trailing: false, trailingPct: 5, gridType: 'linear',
  pyramidEnabled: false,
  pyramidType: 'linear' as 'linear' | 'exponential',
  pyramidMultiplier: 1.5,
  usePriceGroup: false,
  priceGroupBuy: '' as string,
  priceGroupSell: '' as string,
  botMode: 'classic' as 'classic' | 'trailing' | 'pyramid' | 'pricegroup',
})

watch(() => form.botMode, (mode) => {
  form.pyramidEnabled = mode === 'pyramid'
  form.usePriceGroup  = mode === 'pricegroup'
  form.trailing       = mode === 'trailing'
})

watch(mktSymbol, (s) => { form.symbol = s; form.lower = ''; form.upper = ''; preview.value = null; fetchTicker() })
watch(mktExchange, (e) => { form.exchange = e; form.lower = ''; form.upper = ''; preview.value = null; fetchTicker(); loadBalances() })

interface GridPreview {
  orders: Array<{ index: number; side: string; price: number; amount: number; total: number }>
  buy_orders: number; sell_orders: number
}
interface Bot {
  id: string | number; name: string; symbol: string; nr_of_grids: number
  lower_price: number; upper_price: number; status: string
  active_orders?: string; filled_orders?: string
  api_key_label?: string
}

const preview = ref<GridPreview | null>(null)
const bots = ref<Bot[]>([])
const statusMsg = ref('')
const statusType = ref('success')
const showChart = ref(true)
const showBots = ref(true)
const showPreview = ref(true)
const expandedBot = ref<string | number | null>(null)
const chartIframe = ref<HTMLIFrameElement | null>(null)

// Local bid/ask — polled via get_ticker
const localBid = ref(0)
const localAsk = ref(0)
const bid = computed(() => localBid.value)
const ask = computed(() => localAsk.value)

// Allocation
const allocBank = ref(0)
const allocBT = ref(0)
const allocFib = ref(0)
const allocTotal = computed(() => allocBank.value + allocBT.value)

// Strategies (persisted to localStorage)
interface Strategy {
  name: string; lowerPct: number; upperPct: number; grids: number; amount: number
  amountType: string; side: string; incBuy: string; incSell: string
  devPriceBuy: string; devPriceSell: string; devAmtBuy: string; devAmtSell: string
}
const strategies = ref<Strategy[]>([])
function saveStrategies() { localStorage.setItem('gbp_strategies', JSON.stringify(strategies.value)) }
function loadStrategiesFromStorage() {
  try { strategies.value = JSON.parse(localStorage.getItem('gbp_strategies') || '[]') } catch {}
}

function handleStrategySave() {
  const name = prompt('Strategy name:', form.name || 'Strategy') || ''
  if (!name) return
  const bidVal = bid.value || 1; const askVal = ask.value || 1
  const lowerVal = parseFloat(form.lower) || 0; const upperVal = parseFloat(form.upper) || 0
  strategies.value.push({
    name,
    lowerPct: bidVal > 0 ? Math.round((lowerVal / bidVal - 1) * 10000) / 100 : -20,
    upperPct: askVal > 0 ? Math.round((upperVal / askVal - 1) * 10000) / 100 : 20,
    grids: form.grids, amount: form.amount, amountType: form.amountType, side: form.side,
    incBuy: form.incBuy, incSell: form.incSell,
    devPriceBuy: form.devPriceBuy, devPriceSell: form.devPriceSell,
    devAmtBuy: form.devAmtBuy, devAmtSell: form.devAmtSell,
  })
  saveStrategies()
  statusMsg.value = 'Strategy saved'; statusType.value = 'success'
}

function loadStrategy(s: Strategy) {
  const bidVal = bid.value || 0; const askVal = ask.value || 0
  Object.assign(form, {
    lower: bidVal > 0 ? (bidVal * (1 + s.lowerPct / 100)).toFixed(5) : '',
    upper: askVal > 0 ? (askVal * (1 + s.upperPct / 100)).toFixed(5) : '',
    grids: s.grids, amount: s.amount, amountType: s.amountType, side: s.side,
    incBuy: s.incBuy, incSell: s.incSell,
    devPriceBuy: s.devPriceBuy, devPriceSell: s.devPriceSell,
    devAmtBuy: s.devAmtBuy, devAmtSell: s.devAmtSell,
  })
  statusMsg.value = `Loaded: ${s.name}`; statusType.value = 'success'
}

const pairBase  = computed(() => form.symbol.split(/[\/\-]/)[0] || '')
const pairQuote = computed(() => form.symbol.split(/[\/\-]/)[1] || 'USD')
const symBase   = pairBase
const symQuote  = pairQuote

const amountAnalysis = computed(() => {
  const a = Number(form.amount) || 0
  const grids = Number(form.grids) || 0
  const bidVal = bid.value || 0; const askVal = ask.value || 0
  const price = (bidVal && askVal) ? (bidVal + askVal) / 2 : (bidVal || askVal || 0)
  if (!a || !grids || !price) return null
  const aBase = form.amountUnit === 'base' ? a : a / price
  const incMax = Math.max(parseFloat(form.incBuy as any) || 0, parseFloat(form.incSell as any) || 0)
  let firstBase = 0, lastBase = 0, totalBase = 0
  if (form.amountType === 'quantityPerGrid') { firstBase = lastBase = aBase; totalBase = aBase * grids }
  else if (form.amountType === 'totalAmount') { firstBase = lastBase = aBase / grids; totalBase = aBase }
  else {
    firstBase = aBase
    const r = 1 + incMax / 100
    if (Math.abs(r - 1) < 1e-9) { totalBase = aBase * grids; lastBase = aBase }
    else { lastBase = aBase * Math.pow(r, grids - 1); totalBase = aBase * (Math.pow(r, grids) - 1) / (r - 1) }
  }
  return {
    totalBase, totalQuote: totalBase * price,
    firstBase, firstQuote: firstBase * price,
    lastBase, lastQuote: lastBase * price,
    isIncremental: form.amountType === 'incrementalPercent',
    warning: null as string | null, minRequired: null as string | null,
  }
})

const maxOrderSize = computed(() => preview.value ? Math.max(...preview.value.orders.map(o => o.amount), 1) : 1)

const allocatedOrders = computed(() => {
  if (!preview.value) return []
  const orders = preview.value.orders.map((o: any) => ({ ...o, origAmount: o.amount, origTotal: o.total, allocStatus: 'active' as string }))
  if (allocTotal.value === 0 && allocFib.value === 0) return orders
  const buys = orders.filter((o: any) => o.side === 'buy')
  const sells = orders.filter((o: any) => o.side === 'sell')

  if (allocBank.value > 0) {
    const markInterior = (list: any[], pct: number) => {
      if (list.length <= 2) return
      const count = Math.max(1, Math.round(list.length * pct / 100))
      const interior = list.length - 2
      const step = Math.max(1, interior / count)
      for (let i = 0; i < count; i++) {
        const idx = 1 + Math.min(interior - 1, Math.round(i * step))
        if (list[idx].allocStatus === 'active') list[idx].allocStatus = 'bank'
      }
    }
    markInterior(buys, allocBank.value); markInterior(sells, allocBank.value)
  }

  if (allocBT.value > 0) {
    const activeBuys = buys.filter((o: any) => o.allocStatus === 'active')
    const activeSells = sells.filter((o: any) => o.allocStatus === 'active')
    const btPct = allocBT.value / 100
    let buyCollected = 0
    for (let i = 1; i < activeBuys.length; i++) { const take = activeBuys[i].amount * btPct; buyCollected += take; activeBuys[i].amount -= take; activeBuys[i].total = activeBuys[i].amount * activeBuys[i].price; activeBuys[i].allocStatus = 'bt' }
    if (activeBuys.length > 0) { activeBuys[0].amount += buyCollected; activeBuys[0].total = activeBuys[0].amount * activeBuys[0].price; activeBuys[0].allocStatus = 'bt-target' }
    let sellCollected = 0
    for (let i = 0; i < activeSells.length - 1; i++) { const take = activeSells[i].amount * btPct; sellCollected += take; activeSells[i].amount -= take; activeSells[i].total = activeSells[i].amount * activeSells[i].price; activeSells[i].allocStatus = 'bt' }
    if (activeSells.length > 0) { activeSells[activeSells.length-1].amount += sellCollected; activeSells[activeSells.length-1].total = activeSells[activeSells.length-1].amount * activeSells[activeSells.length-1].price; activeSells[activeSells.length-1].allocStatus = 'bt-target' }
  }

  if (allocFib.value > 0) {
    const fibLevels = [0.236, 0.382, 0.5, 0.618, 0.786]
    const lower = parseFloat(form.lower) || 0; const upper = parseFloat(form.upper) || 0
    const currentPrice = bid.value || ((lower + upper) / 2)
    const fibPct = allocFib.value / 100
    const fibTargetsBuy: Set<any> = new Set(); const fibTargetsSell: Set<any> = new Set()
    const buyRange = currentPrice - lower
    if (buyRange > 0) { for (const f of fibLevels) { const fp = currentPrice - buyRange * f; let closest: any = null, minDist = Infinity; for (const o of buys) { if (o.allocStatus !== 'active') continue; const dist = Math.abs(o.price - fp); if (dist < minDist) { minDist = dist; closest = o } } if (closest) fibTargetsBuy.add(closest) } }
    const sellRange = upper - currentPrice
    if (sellRange > 0) { for (const f of fibLevels) { const fp = currentPrice + sellRange * f; let closest: any = null, minDist = Infinity; for (const o of sells) { if (o.allocStatus !== 'active') continue; const dist = Math.abs(o.price - fp); if (dist < minDist) { minDist = dist; closest = o } } if (closest) fibTargetsSell.add(closest) } }
    let buyCollected = 0; for (const o of buys) { if (o.allocStatus !== 'active' || fibTargetsBuy.has(o)) continue; const take = o.amount * fibPct; buyCollected += take; o.amount -= take; o.total = o.amount * o.price; o.allocStatus = 'fib-donor' }
    if (fibTargetsBuy.size > 0) { const perTarget = buyCollected / fibTargetsBuy.size; for (const o of fibTargetsBuy) { o.amount += perTarget; o.total = o.amount * o.price; o.allocStatus = 'fib' } }
    let sellCollected = 0; for (const o of sells) { if (o.allocStatus !== 'active' || fibTargetsSell.has(o)) continue; const take = o.amount * fibPct; sellCollected += take; o.amount -= take; o.total = o.amount * o.price; o.allocStatus = 'fib-donor' }
    if (fibTargetsSell.size > 0) { const perTarget = sellCollected / fibTargetsSell.size; for (const o of fibTargetsSell) { o.amount += perTarget; o.total = o.amount * o.price; o.allocStatus = 'fib' } }
  }

  return orders
})

const previewStats = computed(() => {
  const empty = { avgBuy:'--', avgSell:'--', spreadPerGrid:'--', buyQuote:'--', buyBase:'--', sellQuote:'--', sellBase:'--', profitQuote:'--', profitBase:'--', profitPct:'0', profitQuoteRaw:0, profitBaseRaw:0, activeBuys:0, activeSells:0 }
  if (!preview.value) return empty
  const allOrds = allocatedOrders.value
  const activeOrds = allOrds.filter((o: any) => o.allocStatus !== 'bank')
  const buys = activeOrds.filter((o: any) => o.side === 'buy')
  const sells = activeOrds.filter((o: any) => o.side === 'sell')
  const buyQuote = buys.reduce((s: number, o: any) => s + o.total, 0)
  const buyBase = buys.reduce((s: number, o: any) => s + o.amount, 0)
  const sellBase = sells.reduce((s: number, o: any) => s + o.amount, 0)
  const sellQuote = sells.reduce((s: number, o: any) => s + o.total, 0)
  const avgBuy = buys.length > 0 ? buyQuote / buyBase : 0
  const avgSell = sells.length > 0 ? sellQuote / sellBase : 0
  const lower = parseFloat(form.lower) || 0; const upper = parseFloat(form.upper) || 0
  const spread = form.grids > 1 ? (upper - lower) / form.grids : 0
  const fmt = (n: number) => n.toLocaleString('en-US', { maximumFractionDigits: 2 })
  const profitQuoteRaw = sellQuote - buyQuote
  const profitBaseRaw = buyBase - sellBase
  const midPrice = avgBuy > 0 && avgSell > 0 ? (avgBuy + avgSell) / 2 : avgBuy || avgSell
  const totalCapital = buyQuote + sellBase * midPrice
  const profitPct = totalCapital > 0 ? (profitQuoteRaw / totalCapital * 100) : 0
  return {
    buyQuote: fmt(buyQuote), buyBase: fmt(buyBase),
    sellQuote: fmt(sellQuote), sellBase: fmt(sellBase),
    profitQuote: (profitQuoteRaw >= 0 ? '+' : '') + fmt(profitQuoteRaw),
    profitBase: (profitBaseRaw >= 0 ? '+' : '') + fmt(profitBaseRaw),
    profitPct: profitPct.toFixed(2), profitQuoteRaw, profitBaseRaw,
    avgBuy: avgBuy > 0 ? avgBuy.toFixed(5) : '--',
    avgSell: avgSell > 0 ? avgSell.toFixed(5) : '--',
    spreadPerGrid: spread > 0 ? spread.toFixed(5) : '--',
    activeBuys: buys.length, activeSells: sells.length,
  }
})

const statusBuyPct = computed(() => {
  if (!preview.value) return -1
  const b = (preview.value as any).buy_orders || 0
  const s = (preview.value as any).sell_orders || 0
  return b + s > 0 ? Math.round(b / (b + s) * 100) : 50
})

const balances = ref<Record<string, number>>({})
async function loadBalances() {
  try {
    const bals = await invoke('get_balances', { exchange: form.exchange }) as any[]
    if (bals) for (const b of bals) {
      const coin = (b.coin || b.asset || '').toUpperCase()
      const free = b.free || b.freeBalance || b.available || b.balance || 0
      if (coin) balances.value[coin] = parseFloat(free) || 0
    }
  } catch {}
}
const balanceQuote = computed(() => {
  const q = pairQuote.value.toUpperCase()
  return balances.value[q] || balances.value[q === 'USDC' ? 'USD' : q === 'USD' ? 'USDC' : ''] || balances.value['ZUSD'] || 0
})
const balanceBase  = computed(() => balances.value[pairBase.value.toUpperCase()] || 0)
const needQuoteRaw = computed(() => allocatedOrders.value.filter((o: any) => o.allocStatus !== 'bank' && o.side === 'buy').reduce((s: number, o: any) => s + o.total, 0))
const needBaseRaw  = computed(() => allocatedOrders.value.filter((o: any) => o.allocStatus !== 'bank' && o.side === 'sell').reduce((s: number, o: any) => s + o.amount, 0))
const hasEnoughQuote = computed(() => balanceQuote.value >= needQuoteRaw.value)
const hasEnoughBase  = computed(() => balanceBase.value  >= needBaseRaw.value)

// Placement
const placing = ref(false)
const placingProgress = ref({ current: 0, total: 0, side: '', price: 0, placed: 0, failed: 0 })
const placingLog = ref<any[]>([])
const stopDialog = ref({ visible: false, botId: 0 as number | string, placed: 0 })
let unlistenPlacing: (() => void) | null = null

async function stopPlacement() { try { await invoke('stop_bot_placement') } catch {} }
async function stopDialogCancelOrders() {
  stopDialog.value.visible = false; statusMsg.value = 'Cancelling...'; statusType.value = 'info'
  try { const r = await invoke('cancel_gridbot_orders', { botId: stopDialog.value.botId, deleteAfter: false }) as any; statusMsg.value = `Cancelled ${r.cancelled}/${r.total}`; statusType.value = 'success' } catch (e) { statusMsg.value = 'Cancel error: ' + e; statusType.value = 'error' }
  await loadBots()
}
function stopDialogKeepOrders() { stopDialog.value.visible = false; statusMsg.value = `Stopped — ${stopDialog.value.placed} orders remain`; statusType.value = 'info'; loadBots() }

function exportCSV() {
  if (!preview.value) return
  const csv = '#,Side,Price,Amount,Total\n' + preview.value.orders.map(o => `${o.index},${o.side},${o.price},${o.amount},${o.total}`).join('\n')
  const a = document.createElement('a'); a.href = URL.createObjectURL(new Blob([csv], {type:'text/csv'})); a.download = `grid_${form.symbol}_${form.grids}g.csv`; a.click()
}

function parseBotOrders(bot: any, type: 'active' | 'filled'): any[] {
  try { return JSON.parse(typeof bot[type === 'active' ? 'active_orders' : 'filled_orders'] === 'string' ? bot[type === 'active' ? 'active_orders' : 'filled_orders'] : '[]') } catch { return [] }
}

// Chart URL (TradingView)
const chartUrl = computed(() => {
  const sym = form.symbol.replace('/', '').replace('-', '')
  const ex = form.exchange.toLowerCase()
  let tvSym = sym
  if (ex === 'kraken') tvSym = `KRAKEN:${sym}`
  else if (ex === 'coinbase' || ex === 'coinbaseadvanced') tvSym = `COINBASE:${sym}`
  const cfg = { symbol: tvSym, interval: '15', theme: 'dark', style: '1', locale: 'en', toolbar_bg: '#0d1117', enable_publishing: false, hide_top_toolbar: false, hide_legend: false, save_image: false, container_id: 'gbp_chart' }
  return `https://www.tradingview.com/widgetembed/?frameElementId=gbp_chart&symbol=${tvSym}&interval=15&hidesidetoolbar=0&symboledit=1&saveimage=0&toolbarbg=0d1117&studies=[]&theme=dark&style=1&timezone=Etc%2FUTC&withdateranges=1&hidevolume=0`
})

function togglePreview() {
  if (preview.value) { preview.value = null; statusMsg.value = ''; return }
  handlePreview()
}

async function handlePreview() {
  if (!form.lower && bid.value) form.lower = (bid.value * 0.8).toFixed(5)
  if (!form.upper && ask.value) form.upper = (ask.value * 1.2).toFixed(5)
  const lower = parseFloat(form.lower) || 0; const upper = parseFloat(form.upper) || 0
  if (!lower || !upper || lower >= upper) { statusMsg.value = 'Set prices'; statusType.value = 'error'; return }
  try {
    preview.value = await invoke('calculate_grid', {
      config: { lower_price: lower, upper_price: upper, nr_of_grids: form.grids, amount: form.amount,
        amount_unit: form.amountUnit, amount_type: form.amountType,
        incremental_pct_buy: parseFloat(form.incBuy)||1, incremental_pct_sell: parseFloat(form.incSell)||1,
        deviation_price_buy: parseFloat(form.devPriceBuy)||1, deviation_price_sell: parseFloat(form.devPriceSell)||1,
        deviation_amount_buy: parseFloat(form.devAmtBuy)||0.9, deviation_amount_sell: parseFloat(form.devAmtSell)||0.9,
        side: form.side, grid_type: form.gridType,
        pyramid_enabled: form.pyramidEnabled, pyramid_type: form.pyramidType,
        pyramid_multiplier: form.pyramidMultiplier },
      exchange: form.exchange, symbol: form.symbol,
    })
    if (preview.value) { statusMsg.value = `${preview.value.orders.length} orders (${preview.value.buy_orders}B + ${preview.value.sell_orders}S)`; statusType.value = 'success' }
  } catch (e) { statusMsg.value = 'Error: ' + e; statusType.value = 'error' }
}

async function handleCreate() {
  if (!form.lower && bid.value) form.lower = (bid.value * 0.8).toFixed(5)
  if (!form.upper && ask.value) form.upper = (ask.value * 1.2).toFixed(5)
  const lower = parseFloat(form.lower) || 0; const upper = parseFloat(form.upper) || 0
  if (!lower || !upper || lower >= upper) { statusMsg.value = 'Set prices'; statusType.value = 'error'; return }
  await loadBalances().catch(() => {})
  placing.value = true; placingLog.value = []; placingProgress.value = { current: 0, total: form.grids, side: '', price: 0, placed: 0, failed: 0 }
  statusMsg.value = `Placing ${form.grids} orders...`; statusType.value = 'info'
  try {
    const r = await invoke<any>('create_bot', {
      botType: 'grid', name: form.name, exchange: form.exchange, symbol: form.symbol,
      lowerPrice: lower, upperPrice: upper, nrOfGrids: form.grids, amount: form.amount,
      amountUnit: form.amountUnit, amountType: form.amountType, ordersSide: form.side,
      incPctBuy: parseFloat(form.incBuy)||1, incPctSell: parseFloat(form.incSell)||1,
      devPriceBuy: parseFloat(form.devPriceBuy)||1, devPriceSell: parseFloat(form.devPriceSell)||1,
      devAmountBuy: parseFloat(form.devAmtBuy)||0.9, devAmountSell: parseFloat(form.devAmtSell)||0.9,
      apiKeyId: null, gridType: form.gridType,
      pyramidEnabled: form.pyramidEnabled, pyramidType: form.pyramidType, pyramidMultiplier: form.pyramidMultiplier,
      trailing: form.trailing, trailingTriggerPct: form.trailingPct,
      usePriceGroup: form.usePriceGroup,
      priceGroupBuy: parseFloat(form.priceGroupBuy) || 0, priceGroupSell: parseFloat(form.priceGroupSell) || 0,
    })
    statusMsg.value = `Created #${r.id} — ${r.placed} placed${r.failed > 0 ? `, ${r.failed} failed` : ''}`
    statusType.value = r.failed > 0 ? 'error' : 'success'
    form.name = 'Grid_' + Math.random().toString(36).slice(2, 7)
    await loadBots()
  } catch (e) { statusMsg.value = 'Error: ' + e; statusType.value = 'error' }
  placing.value = false
}

async function loadBots() { try { bots.value = await invoke('get_bots', { botType: 'grid', status: null }) } catch {} }
async function stopBot(id: string | number) { await invoke('update_bot_status', { botId: id, status: 'stopped' }); await loadBots() }
async function startBot(id: string | number) { await invoke('update_bot_status', { botId: id, status: 'running' }); await loadBots() }
async function removeBot(id: string | number) { if (!confirm('Delete bot from DB?')) return; await invoke('delete_bot', { botId: id }); await loadBots() }
async function cancelBotOrders(id: string | number, deleteAfter: boolean) {
  if (!confirm(deleteAfter ? 'Cancel orders + delete bot?' : 'Cancel all orders?')) return
  statusMsg.value = 'Cancelling...'; statusType.value = 'info'
  try { const r = await invoke('cancel_gridbot_orders', { botId: id, deleteAfter }) as any; statusMsg.value = `Cancelled ${r.cancelled}/${r.total}${r.deleted ? ' + deleted' : ''}`; statusType.value = 'success'; await loadBots() }
  catch (e) { statusMsg.value = 'Error: ' + e; statusType.value = 'error' }
}
function resetForm() { Object.assign(form, { lower:'', upper:'', grids:20, amount:1000, incBuy:'1', incSell:'1', devPriceBuy:'1', devPriceSell:'1', devAmtBuy:'0.9', devAmtSell:'0.9' }); preview.value = null; statusMsg.value = '' }

let tickerPoll: any = null
let exchangePublicReady = false
async function fetchTicker() {
  if (!exchangePublicReady) {
    try { await invoke('exchange_register_public', { exchange: form.exchange }); exchangePublicReady = true } catch {}
  }
  try {
    const t = await invoke('exchange_ticker', { exchange: form.exchange, symbol: form.symbol }) as any
    if (t) { localBid.value = t.bid || 0; localAsk.value = t.ask || 0 }
  } catch {}
}

onMounted(async () => {
  loadBots(); fetchTicker(); loadBalances(); loadStrategiesFromStorage()
  tickerPoll = setInterval(fetchTicker, 2000)
  unlistenPlacing = await listen('bot-placing-orders', (event: any) => {
    const d = event.payload
    placingProgress.value = { current: d.current||0, total: d.total||0, side: d.side||'', price: d.price||0, placed: d.placed||0, failed: d.failed||0 }
    if (d.status) placingLog.value.push({ i: d.current, side: d.side, price: d.price, amount: d.amount, status: d.status, order_id: d.order_id||'', error: d.error||'' })
    if (d.done) { placing.value = false; if (d.user_stopped && d.placed > 0) { stopDialog.value = { visible: true, botId: d.bot_id??0, placed: d.placed??0 } } else { loadBots() } }
  })
})
onUnmounted(() => { if (tickerPoll) clearInterval(tickerPoll); if (unlistenPlacing) unlistenPlacing() })
</script>

<style scoped>
@keyframes pulse-red {
  0%, 100% { box-shadow: 0 0 0 0 rgba(235,4,4,0.4); }
  50%       { box-shadow: 0 0 8px 4px rgba(235,4,4,0.2); }
}
.status-bar { transition: all 0.3s; }
.status-bar:hover { transform: scale(1.03); filter: brightness(1.3) saturate(1.5); }
.dev-label  { padding:2px 4px; color:rgba(255,255,255,0.4); border:1px solid rgba(59,130,246,0.08); font-size:9px; }
.dev-buy    { padding:2px 4px; border:1px solid rgba(16,235,4,0.1); transition:background 0.2s; }
.dev-buy:hover  { background:rgba(16,235,4,0.06); }
.dev-sell   { padding:2px 4px; border:1px solid rgba(235,4,4,0.1); transition:background 0.2s; }
.dev-sell:hover { background:rgba(235,4,4,0.06); }
.dev-input-buy  { padding:2px 3px; font-size:10px; width:50px; color:#10eb04; border-color:rgba(16,235,4,0.3); }
.dev-input-sell { padding:2px 3px; font-size:10px; width:50px; color:#eb0404; border-color:rgba(235,4,4,0.3); }
.rainbow-text {
  background: linear-gradient(90deg, #ff0000, #ff8000, #ffff00, #00ff00, #00ffff, #0080ff, #8000ff, #ff00ff);
  background-size: 200% auto; -webkit-background-clip: text; -webkit-text-fill-color: transparent;
  background-clip: text; animation: rainbow-shift 3s linear infinite;
}
@keyframes rainbow-shift { 0% { background-position: 0% center; } 100% { background-position: 200% center; } }
.need-header {
  background: linear-gradient(135deg, rgba(250,204,21,0.3), rgba(235,100,4,0.3), rgba(235,4,100,0.3), rgba(250,204,21,0.3));
  background-size: 300% 300%; animation: need-flash 2s ease infinite; color: #fff; border-radius: 3px;
}
.need-cell {
  background: linear-gradient(135deg, rgba(250,204,21,0.15), rgba(235,100,4,0.2), rgba(235,4,100,0.15), rgba(250,150,21,0.2));
  background-size: 300% 300%; animation: need-flash 2s ease infinite; color: #facc15; border-radius: 3px;
}
@keyframes need-flash { 0% { background-position: 0% 50%; } 50% { background-position: 100% 50%; } 100% { background-position: 0% 50%; } }
.r-panel { flex:1; min-height:0; background:rgba(20,25,30,0.8); border:1px solid; border-radius:6px; display:flex; flex-direction:column; overflow:hidden; }
.r-panel.collapsed { flex:0 0 auto; min-height:auto; }
.r-panel-header { display:flex; align-items:center; gap:6px; padding:4px 8px; cursor:pointer; flex-shrink:0; font-size:10px; font-weight:700; }
.r-arrow { font-size:9px; color:rgba(59,130,246,0.5); }
.r-panel-body { flex:1; min-height:0; overflow-y:auto; padding:0 8px 8px; }
</style>
