import view from './view.js';
import { sameProfile, configFromForm, selectedFeatureCount, chartConfig, chartMatchesModel } from './settings.js';
import { createLiveChart } from './chart.js';
import { createTriggerEditor } from './triggers.js';

export function mount(root, context) {
  root.innerHTML = view;
  const $ = id => root.querySelector(`#${id}`);
  const form = $('ai-form');
  let timer, busy = false, initialized = false, active = false, disposed = false, pollGeneration = 0, catalog;
  let chartTimer, chartBusy=false, chartRevision=0, lastSnapshot, lastReport, lastStatus;
  const selectedInputs=()=>[...form.querySelectorAll('[name="inputs"]:checked')].map(input=>input.value);
  const liveChart=createLiveChart($('ai-live-chart'));
  const triggerEditor=createTriggerEditor($('ai-trigger-editor'),()=>catalog ? catalog.inputs.filter(item=>selectedInputs().includes(item.id)).flatMap(item=>item.series) : [],()=>{updateButtons();scheduleChart();});
  function draftConfig() { return {...configFromForm(new FormData(form)),triggers:triggerEditor.get()}; }
  function updateButtons() {
    let unchanged=false;try {unchanged=lastStatus && sameProfile(draftConfig(),lastStatus.config);} catch {}
    $('ai-start').disabled=busy||!initialized||Boolean(lastStatus?.running&&unchanged);
    $('ai-start').textContent=lastStatus?.running?'Apply & restart learning':'Start daily learning';
    $('ai-stop').disabled=busy||!lastStatus?.running;
  }
  function scheduleChart() {chartRevision++;clearTimeout(chartTimer);if(active&&initialized)chartTimer=setTimeout(refreshChart,350);}
  function displayChart() {
    if(!lastSnapshot)return;
    const matching=lastReport && chartMatchesModel(lastSnapshot.config,lastReport.config);
    liveChart.update(lastSnapshot,matching?lastReport:null);
    $('ai-chart-context').textContent=matching ? `Paper markers: saved ${lastReport.config.symbol} ${lastReport.config.interval} model · score ≥ ${lastReport.config.threshold} · TP ${lastReport.config.take_profit_pct}% after costs. Custom BUY rules: ${lastSnapshot.rule_status.buy?'met':'not met'}; SELL rules: ${lastSnapshot.rule_status.sell?'met':'not met'}.` : 'Chart preview follows the selected inputs. Start/apply learning with these settings to see matching model BUY / SELL / TP markers. Rule and formation markers are available without training.';
    const items=[...lastSnapshot.forming,...lastSnapshot.confirmed.slice(-5).reverse()];
    $('ai-live-formations').replaceChildren(...items.map(item=>node('p',`${item.status==='forming'?'FORMING':'CONFIRMED'} · ${item.name} · ${item.direction}${item.bull_trigger!==null?' · break above '+num(item.bull_trigger,4):''}${item.bear_trigger!==null?' · break below '+num(item.bear_trigger,4):''}${item.confirmed_at?' · '+date(item.confirmed_at)+' UTC':''}`)));
    if(!items.length)$('ai-live-formations').textContent='No matching formations in the recent chart window for the selected pattern inputs.';
  }
  async function refreshChart() {
    if(disposed||!active||!initialized)return;
    if(chartBusy){chartTimer=setTimeout(refreshChart,500);return;}
    let config;try{config=chartConfig(draftConfig());}catch(error){$('ai-chart-status').textContent=String(error);return;}
    const revision=chartRevision;chartBusy=true;let delay=5000;
    $('ai-chart-status').textContent=`Updating ${config.symbol} ${config.interval}…${lastSnapshot?' Previous snapshot remains visible.':''}`;
    try {
      const snapshot=await context.invoke('ai_chart_snapshot',{config});
      if(disposed||revision!==chartRevision)return;
      if(snapshot.error){delay=Math.max(15000,Math.min(3600000,snapshot.retry_seconds*1000||15000));throw new Error(snapshot.error);}
      lastSnapshot=snapshot;displayChart();
      $('ai-chart-status').classList.remove('error');
      $('ai-chart-status').textContent=`${snapshot.preview?'SAVED TEST SNAPSHOT':snapshot.delayed?'DELAYED DATA':'LIVE · 5-second refresh'} · ${config.symbol} ${config.interval} · updated ${date(snapshot.updated_at)} UTC · ${snapshot.provisional?'current candle provisional':'completed candles'}`;
    } catch(error) {
      if(!disposed&&revision===chartRevision){$('ai-chart-status').classList.add('error');$('ai-chart-status').textContent=`Chart unavailable: ${error}. ${lastSnapshot?'Last successful snapshot: '+date(lastSnapshot.updated_at)+' UTC.':''}`;delay=Math.max(delay,15000);}
    } finally {chartBusy=false;if(active&&!disposed)chartTimer=setTimeout(refreshChart,revision===chartRevision?delay:0);}
  }
  const date = value => value ? new Date(value).toISOString().replace('T', ' ').slice(0, 19) : '—';
  const num = (value, digits = 3) => typeof value === 'number' ? value.toFixed(digits) : '—';
  function node(tag, text, cls) {
    const element = document.createElement(tag);
    if (text !== undefined) element.textContent = text;
    if (cls) element.className = cls;
    return element;
  }
  function metric(label, value) {
    const card = node('div'); card.append(node('span', label), node('strong', value)); return card;
  }
  function updateCount() {
    if (!catalog) return;
    const inputs = [...form.querySelectorAll('[name="inputs"]:checked')].map(input => input.value);
    $('ai-input-count').textContent = `${inputs.length} inputs selected · ${selectedFeatureCount(catalog,inputs)} numerical features · no hidden inputs`;
    $('ai-input-compact').textContent=`${inputs.length} selected`;
    triggerEditor.refresh(); updateButtons(); scheduleChart();
  }
  function buildInputs(status) {
    catalog = status.catalog;
    if (!catalog) throw new Error('Input catalog unavailable. Reopen the updated desktop app.');
    const interval=form.elements.namedItem('interval');interval.replaceChildren(...catalog.intervals.map(item=>{const option=node('option',item.label);option.value=item.id;return option;}));
    const host = $('ai-input-options'); host.replaceChildren();
    for (const [kind,title] of [['indicator','Indicators'],['pattern','Chart formations'],['price','Additional price data']]) {
      const group = node('div', undefined, 'ai-input-group');
      group.append(node('h3', title));
      const options = node('div', undefined, 'ai-checkbox-grid');
      for (const item of catalog.inputs.filter(item => item.kind === kind)) {
        const label = node('label', undefined, 'ai-checkbox');
        const input = node('input');
        input.type = 'checkbox'; input.name = 'inputs'; input.value = item.id;
        input.checked = status.config.inputs.includes(item.id);
        input.addEventListener('change', updateCount);
        label.append(input, node('span',item.label)); options.append(label);
      }
      group.append(options); host.append(group);
    }
    updateCount();
  }
  for (const button of form.querySelectorAll('[data-preset]')) button.addEventListener('click', () => {
    const presets = {rsi:['rsi'],ema:['ema'],combined:['rsi','ema','head_shoulders','inverse_head_shoulders','double_top','double_bottom'],none:[]};
    const selection = button.dataset.preset === 'all' ? catalog.inputs.map(item=>item.id) : button.dataset.preset==='patterns' ? catalog.inputs.filter(item=>item.kind==='pattern').map(item=>item.id) : presets[button.dataset.preset];
    for (const input of form.querySelectorAll('[name="inputs"]')) input.checked = selection.includes(input.value);
    updateCount();
  });
  function render(status) {
    if (disposed) return;
    lastStatus=status;
    if (!initialized) {
      buildInputs(status);
      for (const [name,value] of Object.entries(status.config)) if (!['inputs','triggers'].includes(name)) form.elements.namedItem(name).value = value;
      triggerEditor.set(status.config.triggers);
      initialized = true;
      scheduleChart();
    }
    updateButtons();
    const worker = status.worker || {};
    const error = status.error || (status.running && worker.error);
    const stale = status.running && worker.heartbeat && Date.now()-worker.heartbeat > 420000;
    $('ai-notice').textContent = error ? `Worker needs attention: ${error}` : stale ? 'Worker heartbeat is stale. Stop and restart if it does not recover.' : status.running ? ({fetching:'Downloading closed public candles…',evaluating:'Evaluating outcomes and training when due…',waiting:'Running · checks every 15 seconds · learns every 24 hours'}[worker.phase] || 'Starting the AI worker…') : 'Learning stopped. The live chart still works. Select inputs and triggers, then start learning when ready.';
    $('ai-notice').classList.toggle('error', Boolean(error || stale));
    $('ai-directory').textContent = `Research files: ${status.data_directory}`;
    const report = worker.report;
    if (!report || report.schema !== 3 || !sameProfile(status.config,report.config)) {
      if(lastReport){lastReport=null;displayChart();}
      for (const id of ['ai-metrics','ai-indicators','ai-signals','ai-trades','ai-history','ai-weights','ai-patterns','ai-position']) $(id).replaceChildren();
      $('ai-selected').textContent = '';
      $('ai-current-signal').textContent = 'Waiting for the first report for these settings.';
      $('ai-pattern-summary').textContent = 'Selected formations appear after a confirmed neckline break.';
      $('ai-evidence').textContent = 'Waiting for the first research report for these settings.';
      return;
    }
    const changedReport=lastReport?.updated_at!==report.updated_at;
    lastReport=report;
    if(changedReport)displayChart();
    $('ai-directory').textContent = `Profile files: ${report.profile}`;
    $('ai-metrics').replaceChildren(metric('CLOSED CANDLES',report.candle_count.toLocaleString()),
      metric('EXAMPLES LEARNED / SIDE',report.training_updates.toLocaleString()),
      metric('LAST TRAINING · UTC',date(report.last_train)),metric('NEXT TRAINING · UTC',date(report.next_train)));
    $('ai-indicators').replaceChildren(...report.indicators.map(name=>node('span',name)));
    $('ai-selected').textContent = `${report.feature_count} features in each model. These exact inputs are used for training and signals: ${report.feature_names.join(', ')}.`;
    const latest = report.signals[0];
    const signalHost = $('ai-current-signal'); signalHost.replaceChildren();
    if (latest) {
      signalHost.append(node('strong',latest.signal,'ai-signal '+(latest.signal === 'BUY' ? 'long' : latest.signal === 'SELL' ? 'short' : '')),
        node('p',`Long score ${num(latest.p)} · Short score ${num(latest.short_p)} · threshold ${num(report.config.threshold,2)}`),
        node('p',`${date(latest.issued_at)} UTC · ${latest.reason}`));
      if (!status.running || worker.error || Date.now()-latest.issued_at > 2*catalog.intervals.find(item=>item.id===report.config.interval).ms) {
        signalHost.append(node('p','Saved research signal — not a current market alert.','ai-note'));
      }
    } else signalHost.textContent = 'Waiting for a fresh candle; missed predictions are never backfilled.';
    const position = report.position;
    $('ai-position').replaceChildren(node('p',position ? `${position.status} ${position.side} · entry ${date(position.entry_time)} UTC · TP price ${num(position.target_price,4)} · time exit ${date(position.deadline)} UTC` : 'No pending or open paper position.'));
    const lastExit = report.trades.find(trade=>trade.status === 'CLOSED');
    if (lastExit) $('ai-position').append(node('p',`Latest exit: ${lastExit.reason} · closed ${lastExit.side} · ${num(lastExit.net*100,2)}% after costs · ${date(lastExit.exit_time)} UTC`));
    const patternsEnabled = report.config.inputs.some(id=>catalog.inputs.find(item=>item.id===id)?.kind === 'pattern');
    const pattern = report.patterns[0];
    $('ai-pattern-summary').textContent = !patternsEnabled ? 'Pattern learning is off for this saved profile.' : pattern ? `Latest historical confirmation: ${pattern.name} · ${pattern.direction} · ${date(pattern.confirmed_at)} UTC.` : 'No confirmed matches in the saved training history.';
    $('ai-patterns').replaceChildren(...report.patterns.slice(0,6).map(item=>node('p',`${date(item.confirmed_at)} UTC · ${item.name} · ${item.direction} · neckline ${num(item.neckline,4)}`)));
    const evidence = $('ai-evidence'); evidence.replaceChildren();
    evidence.append(node('p',`Current model v${report.champion_version} · candidate ${report.challenger_generation} · ${report.evidence_count} / ${report.evidence_required} non-overlapping forward outcomes.`),node('p',report.last_decision));
    const table = node('table'), header = node('tr');
    for (const label of ['Evaluation','Brier ↓','Trades','Net return','Drawdown']) header.append(node('th',label));
    table.append(header);
    for (const [label,m] of [['Historical validation',report.bootstrap.validation],['Historical holdout',report.bootstrap.holdout],['Fixed base-rate baseline',report.bootstrap.holdout_baseline],['Forward paper record',report.forward]]) {
      const tr = node('tr'); for (const value of [label,num(m.brier),String(m.trades),`${num(m.return_pct,2)}%`,`${num(m.drawdown_pct,2)}%`]) tr.append(node('td',value)); table.append(tr);
    }
    evidence.append(table,node('p',`Forward prediction samples: ${report.forward.samples}. Brier averages LONG and SHORT prediction errors; lower is better. Forward returns use recorded paper positions. Drawdown uses closed positions only.`));
    $('ai-signals').replaceChildren(...report.signals.map(signal=>{
      const tr = node('tr'); for (const value of [date(signal.issued_at),signal.signal,num(signal.p),num(signal.short_p),date(signal.entry_time),signal.reason]) tr.append(node('td',value)); return tr;
    }));
    $('ai-trades').replaceChildren(...report.trades.slice(0,20).map(trade=>{
      const tr = node('tr'); for (const value of [date(trade.entry_time),trade.side,trade.reason || trade.status,num(trade.entry_price,4),num(trade.exit_price,4),trade.net === undefined ? 'Pending' : `${num(trade.net*100,2)}%`]) tr.append(node('td',value)); return tr;
    }));
    if (!report.trades.length) { const row = node('tr'), cell = node('td','No paper positions yet.'); cell.colSpan=6; row.append(cell); $('ai-trades').append(row); }
    $('ai-history').replaceChildren(...report.runs.map(run=>node('p',`${date(run.at)} · ${run.kind}${run.examples !== undefined ? ` · ${run.examples} new examples per side` : ''}${run.reason ? ` · ${run.reason}` : ''}`)));
    $('ai-weights').replaceChildren(...report.weights.map(item=>node('span',`${item.side} ${item.feature}: ${num(item.weight)}`)));
  }
  async function refresh() {
    try { render(await context.invoke('ai_status')); }
    catch (error) { if (!disposed) { $('ai-notice').textContent=String(error); $('ai-notice').classList.add('error'); } }
  }
  async function poll(generation) {
    await refresh();
    if (active && !disposed && generation === pollGeneration) timer=setTimeout(()=>poll(generation),5000);
  }
  async function action(name,args) {
    if (busy) return;
    busy=true;
    updateButtons();
    try { if(name==='ai_start'&&lastStatus?.running)await context.invoke('ai_stop'); await context.invoke(name,args); busy=false; await refresh(); }
    catch (error) { busy=false; await refresh(); if (!disposed) { $('ai-notice').textContent=String(error); $('ai-notice').classList.add('error'); } }
  }
  form.addEventListener('submit',event=>{
    event.preventDefault();
    try { void action('ai_start',{config:draftConfig()}); }
    catch (error) { $('ai-notice').textContent=String(error); $('ai-notice').classList.add('error'); }
  });
  $('ai-stop').addEventListener('click',()=>action('ai_stop'));
  form.addEventListener('change',event=>{if(event.target.name!=='inputs'){updateButtons();scheduleChart();}});
  return {
    activate() { if (!active) { active=true; void poll(++pollGeneration);scheduleChart(); } },
    deactivate() { active=false; ++pollGeneration; ++chartRevision;clearTimeout(timer);clearTimeout(chartTimer); },
    dispose() { disposed=true; active=false; ++pollGeneration; ++chartRevision;clearTimeout(timer);clearTimeout(chartTimer);liveChart.dispose();root.replaceChildren(); }
  };
}
