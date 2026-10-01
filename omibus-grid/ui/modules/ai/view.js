export default `
<div class="ai-heading"><div><div class="eyebrow">AI RESEARCH 3 / LIVE CHART</div><h1>Read the market<span>.</span></h1><p>Your indicators, formations and triggers. One live research chart.</p></div><span class="ai-badge">RESEARCH 3 · PAPER</span></div>
<p class="ai-intro">Choose exactly what the model learns. BUY opens a simulated long; SELL opens a simulated short. TP closes either at its profit target.</p>
<div class="ai-status" role="status" aria-live="polite" id="ai-notice">Loading worker status…</div>
<form id="ai-form" class="ai-controls">
  <label>Binance symbol<input name="symbol" value="BTCUSDT" maxlength="20" required pattern="[A-Z0-9]{5,20}" autocomplete="off"></label>
  <label>Candle timeframe<select name="interval"><option value="15m">15 minutes</option><option value="1h" selected>1 hour</option><option value="4h">4 hours</option><option value="1d">1 day</option></select></label>
  <label>Maximum holding period (bars)<input name="horizon" type="number" value="4" min="1" max="24" required></label>
  <label>Take profit after costs (%)<input name="take_profit_pct" type="number" value="1" min="0.1" max="50" step="0.1" required></label>
  <details class="ai-picker"><summary>Simulation & training settings</summary><div class="ai-simulation-fields">
  <label>Initial history (bars)<input name="history_bars" type="number" value="3000" min="1500" max="10000" required></label>
  <label>Fee per side (bps)<input name="fee_bps" type="number" value="10" min="0" max="100" step="0.1" required></label>
  <label>Slippage per side (bps)<input name="slippage_bps" type="number" value="5" min="0" max="100" step="0.1" required></label>
  <label>Entry score threshold<input name="threshold" type="number" value="0.60" min="0.50" max="0.95" step="0.01" required></label>
  </div></details>
  <details class="ai-picker"><summary>Indicators & 20 chart formations · <span id="ai-input-compact"></span></summary>
  <fieldset class="ai-inputs"><legend>Learning inputs and chart studies</legend>
    <p>Only checked inputs enter the model. RSI only really means RSI only. Formations are optional learning inputs too.</p>
    <div class="ai-presets"><button type="button" data-preset="rsi">RSI only</button><button type="button" data-preset="ema">EMA only</button><button type="button" data-preset="combined">RSI + EMA + formations</button><button type="button" data-preset="patterns">All 20 formations</button><button type="button" data-preset="all">Select all</button><button type="button" data-preset="none">Clear</button></div>
    <div id="ai-input-options"></div><p id="ai-input-count" aria-live="polite"></p>
  </fieldset></details>
  <details class="ai-picker"><summary>Custom BUY / SELL trigger conditions</summary><div id="ai-trigger-editor"></div></details>
  <div class="ai-actions"><button class="small-btn" type="submit" id="ai-start" disabled>Start daily learning</button><button class="small-btn" type="button" id="ai-stop" disabled>Stop</button></div>
  <p class="ai-control-note">The chart follows your selections immediately. Apply & restart to train a running model on new settings. Each combination has its own saved research profile.</p>
</form>
<p class="ai-note">100 bps = 1%. Entry follows a one-bar delay. One paper position at a time, closed at TP or at the maximum holding period (TIME EXIT). Scores are uncalibrated. Short trades use Spot candles as a price proxy; funding, borrowing and liquidation are not modeled. No exchange orders are sent.</p>
<section class="ai-panel ai-live-panel"><h2>Live research chart</h2><p id="ai-chart-status" role="status">Loading chart settings…</p><p id="ai-chart-context" class="ai-note"></p><div id="ai-live-chart"></div><div id="ai-live-formations" class="ai-live-formations"></div><p class="ai-note">Prices and provisional indicator values refresh about every 5 seconds, subject to the connection. Dashed amber formations are developing. BREAK markers require a completed candle. RULE markers show custom conditions; BUY / SELL / TP markers show the saved model's paper positions. A rule or formation alone does not force a trade.</p></section>
<div class="ai-metrics" id="ai-metrics"></div>
<div class="ai-columns">
  <section class="ai-panel"><h2>Latest model signal</h2><div id="ai-current-signal">Start learning to receive model signals.</div><div id="ai-position"></div></section>
  <section class="ai-panel"><h2>Inputs used by this model</h2><div id="ai-indicators" class="ai-tags"></div><p id="ai-selected"></p></section>
</div>
<details class="ai-panel"><summary>Saved model's formation history</summary><p id="ai-pattern-summary"></p><div id="ai-patterns"></div><p class="ai-note">Fixed geometric rules can miss patterns and flag false matches. Developing patterns can disappear; they do not become learning inputs until confirmation.</p></details>
<section class="ai-panel"><h2>Model evidence</h2><div id="ai-evidence">The first run trains on older candles and evaluates on later, separate periods.</div></section>
<section class="ai-panel"><h2>Paper positions & exits</h2><div class="ai-table-wrap"><table><thead><tr><th>Entry (UTC)</th><th>Position</th><th>Status / exit</th><th>Entry price</th><th>Exit price</th><th>Net result</th></tr></thead><tbody id="ai-trades"></tbody></table></div></section>
<section class="ai-panel"><h2>Forward signal journal</h2><p>Predictions are recorded before their scheduled entry. HOLD keeps an existing position; WAIT opens none.</p><div class="ai-table-wrap"><table><thead><tr><th>Issued (UTC)</th><th>Action</th><th>Long score</th><th>Short score</th><th>Earliest entry (UTC)</th><th>Reason</th></tr></thead><tbody id="ai-signals"></tbody></table></div></section>
<details class="ai-panel"><summary>Training history & model weights</summary><div id="ai-history"></div><p>Weights belong to the daily learner. Current signal models stay frozen between accepted replacements. Coefficients are not causal importance.</p><div id="ai-weights" class="ai-tags"></div></details>
<details class="ai-panel"><summary>How learning and simulation work</summary><p>Two statistical models learn LONG and SHORT outcomes from exactly the selected inputs. Custom conditions filter both historical and future simulated entries; they do not change the prediction target. BUY opens LONG, SELL opens SHORT, TP closes at the target, and TIME EXIT closes at the holding limit.</p><p>Patterns are detected using fixed geometric rules. They are not a profitability ranking. The initial model uses chronological train/validation/holdout periods. A daily learner must pass two separate groups of 50 future outcomes before replacing it. Predictive advantage is not guaranteed.</p><p>The independent chart works without starting training. It refreshes every 5 seconds while this tab is visible. The research worker checks closed candles every 15 seconds and learns every 24 hours while Omibus is running. The current candle is provisional and cannot confirm a formation or fire a trade entry rule.</p><p>TP is simulated using completed candle highs/lows and fills at the target after configured costs. TIME EXIT can realize a loss. No stop-loss, funding, borrowing, liquidation or intrabar drawdown is modeled. Older research remains saved in separate versioned profiles.</p><code id="ai-directory"></code></details>
`;
