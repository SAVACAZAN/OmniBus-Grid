export default `
        <div class="chart-heading"><div><div class="eyebrow">MARKET LAB / 002</div><h1>Market charts<span>.</span></h1></div><span class="chart-source">TradingView · public market data</span></div>
        <form id="chart-form" class="chart-controls">
          <label>Starting market<input id="chart-symbol" value="BINANCE:BTCUSDT" maxlength="60" spellcheck="false" autocomplete="off" list="chart-markets" required></label>
          <datalist id="chart-markets"><option value="BINANCE:BTCUSDT"><option value="BINANCE:ETHUSDT"><option value="BINANCE:SOLUSDT"><option value="BITSTAMP:BTCUSD"><option value="COINBASE:BTCUSD"></datalist>
          <label>Starting timeframe<select id="chart-interval"><option value="1">1 minute</option><option value="5">5 minutes</option><option value="15">15 minutes</option><option value="60">1 hour</option><option value="240">4 hours</option><option value="D" selected>1 day</option><option value="W">1 week</option><option value="M">1 month</option></select></label>
          <button type="submit" class="small-btn">Load chart</button>
          <button type="button" id="chart-reload" class="small-btn">Reconnect</button>
        </form>
        <p id="chart-status" class="chart-status" role="status" aria-live="polite">Choose a market, or use symbol search inside the chart.</p>
        <section id="chart-host" class="chart-host" aria-label="Interactive market chart"></section>
        <div class="chart-attribution"><a href="https://www.tradingview.com/" target="_blank" rel="noopener noreferrer">Charts by TradingView</a><span>Chart prices do not feed paper bots.</span></div>
        <details class="chart-help"><summary>Chart tools & connection help</summary><p>Use the chart toolbar for symbols, timeframes, indicators and chart styles. Drawing tools are on the left. Scroll to zoom and drag to pan. The chart requires an internet connection; if it stays blank, check your connection and press Reconnect. Market availability depends on TradingView.</p><p>This embedded chart does not include custom Pine scripts, strategies or the full TradingView account workspace. Changing tabs keeps this chart open; reloading or closing the app can reset drawings and indicator changes. Starting market and timeframe above are saved locally; changes made inside TradingView are not synchronized to those fields.</p></details>
`;
