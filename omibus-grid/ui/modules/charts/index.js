import view from './view.js';
export function mount(root) {
root.innerHTML = view;
  const select = id => root.querySelector(`#${id}`);
  const intervals = new Set(['1', '5', '15', '60', '240', 'D', 'W', 'M']);
  const validSymbol = symbol => /^[A-Z0-9_]{1,20}:[A-Z0-9_.!/-]{1,35}$/.test(symbol);
  let mounted = false;
  let timer;

  function loadChart() {
    const symbol = select('chart-symbol').value.trim().toUpperCase();
    const interval = select('chart-interval').value;
    if (!validSymbol(symbol) || !intervals.has(interval)) {
      select('chart-status').textContent = 'Use EXCHANGE:SYMBOL, for example BINANCE:BTCUSDT or BITSTAMP:BTCUSD.';
      return;
    }
    select('chart-symbol').value = symbol;
    try { localStorage.setItem('omibus-chart-start', JSON.stringify({ symbol, interval })); } catch { /* Storage may be disabled. */ }
    const settings = {
      autosize: true, symbol, interval, timezone: 'Etc/UTC', theme: 'dark', style: '1', locale: 'en',
      allow_symbol_change: true, hide_side_toolbar: false, hide_top_toolbar: false,
      hide_legend: false, hide_volume: false, withdateranges: true, save_image: true,
      details: true, calendar: false, hotlist: false,
      watchlist: ['BINANCE:BTCUSDT', 'BINANCE:ETHUSDT', 'BINANCE:SOLUSDT', 'BITSTAMP:BTCUSD'],
      studies: [], support_host: 'https://www.tradingview.com'
    };
    const url = new URL('https://www.tradingview-widget.com/embed-widget/advanced-chart/');
    url.searchParams.set('locale', 'en');
    url.hash = encodeURIComponent(JSON.stringify(settings));
    const frame = document.createElement('iframe');
    frame.title = 'TradingView interactive market chart';
    frame.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads');
    frame.setAttribute('allow', 'fullscreen');
    frame.referrerPolicy = 'strict-origin-when-cross-origin';
    select('chart-status').textContent = 'Opening TradingView…';
    clearTimeout(timer);
    timer = setTimeout(() => {
      select('chart-status').textContent = 'Chart taking too long? Check your internet connection, then press Reconnect.';
    }, 20000);
    frame.addEventListener('load', () => {
      clearTimeout(timer);
      // A cross-origin load event does not prove that market data has arrived.
      select('chart-status').textContent = 'Use the chart toolbar to explore markets. If the chart is blank, press Reconnect.';
    });
    frame.src = url.href;
    select('chart-host').replaceChildren(frame);
    mounted = true;
  }


  function initialize() {
    try {
      const saved = JSON.parse(localStorage.getItem('omibus-chart-start'));
      if (saved && validSymbol(saved.symbol) && intervals.has(saved.interval)) {
        select('chart-symbol').value = saved.symbol;
        select('chart-interval').value = saved.interval;
      }
    } catch { /* Fall back to the default market. */ }
    select('chart-form').addEventListener('submit', event => { event.preventDefault(); loadChart(); });
    select('chart-reload').addEventListener('click', loadChart);
  }
  initialize();
  return { activate() { if (!mounted) loadChart(); }, dispose() { clearTimeout(timer); root.replaceChildren(); } };
}
