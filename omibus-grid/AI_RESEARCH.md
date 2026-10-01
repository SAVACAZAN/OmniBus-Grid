# AI Research 3 — live charts, selected inputs and custom triggers

AI Research runs two small regularized logistic models locally: one predicts a
positive net LONG outcome and the other a positive net SHORT outcome. It updates
their learner weights daily. A causal geometric detector recognizes 20 named
formations and can supply those detections as model inputs. It does not invent
new pattern definitions, rewrite itself, or use an LLM.

## Start and choose inputs

1. Open **Omibus Grid Research.exe**. The window title identifies **AI Research 3**.
   Keep the adjacent **ai-worker** folder and **modules.json** with it.
2. Open **AI Research**. Choose the market, timeframe and simulation settings.
3. Check exactly the indicators/formations to learn. **RSI only**, **EMA only**,
   **RSI + EMA + formations**, **All 20 formations**, **Select all**, and **Clear** are convenience
   presets. Individual boxes allow any nonempty combination.
4. Optionally add custom BUY/SELL conditions. Set the entry score threshold and
   **Take profit after costs (%)**, then press **Start daily learning**.
   For a running worker, **Apply & restart learning** applies edited settings.
5. Keep Omibus running and the PC awake. Tab changes do not stop training.
   Closing Omibus stops its worker; a previously started worker resumes on
   reopening. Explicit Stop saves research and disables automatic resume.

Defaults: BTCUSDT, 1h, 3,000 history bars, maximum holding period 4 bars, 10 bps
fees and 5 bps slippage per side, score threshold 0.60, net TP 1%.
The initial inputs are RSI14, EMA12/26, MACD12/26/9, head-and-shoulders and inverse
head-and-shoulders: seven numerical features. Indicator periods are fixed and
shown in the selector; custom periods are not implemented.

The shared catalog is **ai-worker/catalog.json**. It contains 20 indicator
families, an optional candle/return/volume group and 20 optional formations.
Only the selected families' declared features reach either model. RSI alone
uses one feature; EMA alone uses three EMA/price ratios, with no RSI, ATR,
volume or pattern inputs. Composite indicators such as Keltner retain their
own mathematical ingredients. No automatic fallback swaps your selection for
a different model. Feature selection and model weights are visible on screen.

## Live chart and custom conditions

The chart fetches public Binance Spot candles through a read-only native command.
It works while training is stopped. Select 1m, 5m, 15m, 30m, 1h, 4h, 1d or 1w.
Price candles and selected indicators refresh approximately every five seconds
after each response. This uses polling, not a tick-by-tick WebSocket stream.
The current candle and its indicator values are provisional. Data errors and
stale snapshots are labeled; failures never substitute invented candles.

Moving averages, bands and channels overlay price. Oscillators such as RSI and
MACD have separate panels. Zoom, earlier/later navigation, return-to-live and
a candle crosshair are available. The chart fetches 600 bars and reserves the
first 100 for indicator warmup, leaving up to 500 visible bars. This local
research chart does not include the entire TradingView drawing/Pine workspace;
the existing Market Charts tab still provides the hosted TradingView chart.

Training requires at least 800 labeled examples after warmup. A weekly chart
can work even when the exchange has too few weekly candles to train a model;
the worker reports the available count and asks for a shorter timeframe.

Custom entry filters support **above**, **below**, **crosses above** and
**crosses below**, comparing a selected indicator with a number or another
selected indicator. Examples: RSI crosses above 30; EMA12 crosses above EMA26.
Up to four conditions per BUY/SELL side combine with AND. An empty side uses
the model score alone. Removing an indicator removes dependent conditions and
shows a notice. Indicator periods remain fixed as named in the catalog.

Conditions use completed candles and gate learned entries in historical
evaluation and forward simulation. They do not replace the model threshold.
**RULE BUY / RULE SELL** marks a transition to satisfied custom conditions;
**BREAK** marks a confirmed formation breakout. **BUY / SELL / TP / TIME EXIT**
marks actual saved paper positions for the matching market, timeframe, inputs
and rules. Selecting a different draft chart does not relabel old trades as new
signals. Numeric oscillator conditions also draw their threshold levels.

## BUY, SELL, HOLD, WAIT and TP

- **BUY** schedules a simulated LONG position.
- **SELL** schedules a simulated SHORT position.
- **WAIT** schedules no position.
- **HOLD LONG / HOLD SHORT** retains the existing pending/open position.
- **TP** closes that position at its configured profit target.
- **TIME EXIT** closes it at the maximum holding period if TP was not reached;
  this can realize a loss and is never mislabeled as TP.

Both directional scores are learned from the selected features. A new entry
requires a unique higher eligible score at or above the threshold; that side's
custom conditions must pass. The scores are
uncalibrated and are not complements; both hypothetical directions can reach
TP on different intrabar moves. They must not be read as calibrated odds.

Only one paper position is pending/open at a time. The journal persists pending
entries and resolved exits across restarts. Duplicate polls do not create
duplicate positions. The dashboard shows the latest model action, both scores,
current position, TP price once the entry candle is available, latest exit,
and the saved signal/trade journals. No real orders or account connections exist.

## Formations and their evidence

Supported patterns:
- Head-and-shoulders and inverse head-and-shoulders.
- Double tops and double bottoms.
- Triple tops and triple bottoms.
- Ascending, descending and symmetrical triangles.
- Rising and falling wedges.
- Bull and bear flags; bull and bear pennants.
- Rectangles; rising and falling channels.
- Cup-and-handle and inverse cup-and-handle.

A swing requires three closed candles on each side. Candidates must match
their pattern's alternating pivots, trend or flagpole requirements, relative
heights, slopes and width/depth tolerances. Dashed amber overlays show forming
candidates; a completed-candle boundary break confirms a detection. These
delayed, confirmed pivots avoid pretending a swing was known at its peak.
Detections are timestamped
at confirmation, never backdated to the head or shoulder. Outside bars with
ambiguous swing order are skipped. The chart shows formation paths, boundaries,
names and breakout markers. To limit clutter, it draws up to six recent
confirmations and six developing formations; the list identifies other matches.

This is an explicit geometric heuristic. It can miss formations and find false
matches. Its 12–120 bar width, 30-bar breakout deadline and price tolerances
are implementation choices, not a universal textbook definition or proven
trading edge. Volume is not required by the detector; a volume indicator is
included in learning only when explicitly selected. A detection contributes
a feature that decays over 20 bars and is invalidated by a close beyond the
formation's extreme. Detection does not itself force a trade.

These are 20 common formation types, not a statistically ranked "top 20".
Synthetic positive examples verify each detector; they do not establish its
precision, recall or profitability on unseen markets.

Conceptual references:
[Chart pattern overview](https://chartschool.stockcharts.com/table-of-contents/chart-analysis/chart-patterns),
[flags and pennants](https://chartschool.stockcharts.com/table-of-contents/chart-analysis/chart-patterns/flag-pennant),
[Head and shoulders](https://chartschool.stockcharts.com/table-of-contents/chart-analysis/chart-patterns/head-and-shoulders-top)
and [double top](https://chartschool.stockcharts.com/table-of-contents/chart-analysis/chart-patterns/double-top-reversal).
The local detector adapts their geometry and neckline concept to fixed bar-based
rules; it does not claim to reproduce all of those sources' criteria.

## Learning, timing and evaluation

Completed Binance Spot candles are checked every 15 seconds. Initial training
runs immediately; daily learning uses a persisted 24-hour deadline.

Features at candle i use only data through i. A prediction's proposed entry
is candle i+2 open, skipping one full bar. The worker checks its clock after
fetching/training so it cannot issue an entry that already occurred.
At most the next configured holding-period candles define each outcome.
Training waits until that entire horizon is closed for both directional labels.

TP is specified as net return after the configured fees/slippage. Each direction's
raw target price is solved from that net target. A closed candle's high/low
can establish a touch, conservatively filled at the target even across gaps.
Because its actual intrabar time is unknown, TP accounting timestamps use the
touching candle's close. Otherwise the final candle close supplies TIME EXIT.
Historical labels, historical simulation and forward paper positions use the
same outcome function. There is no stop-loss in this version.

The initial history uses a chronological 60/20/20 train/validation/holdout split.
Labels crossing a boundary are purged. The initial signal models are trained
only on the training partition; validation and holdout do not pick features or
change parameters. The separate learner subsequently incorporates known later
outcomes. Daily updates use only previously untrained, matured examples.

Frozen candidate and current models predict on the same future candles.
After 100 non-overlapping resolved examples, two disjoint groups of 50 must
both show lower average LONG/SHORT Brier error than the current model and a
fixed training base-rate predictor, at least five hypothetical trades, positive
returns above the current model and no worse exit-based drawdown. Otherwise the
current model remains. Each new candidate gets an entirely new evidence window.

Forward prediction error uses recorded forecasts; forward returns use positions
actually scheduled by the paper journal. Historical/candidate metrics simulate
their own non-overlapping entries. Results are full-notional, without leverage,
and compound closed trade returns. Equity is floored at zero if exhausted and
further paper entries are suppressed. No significance level or improvement
guarantee is claimed. Trying many manually selected profiles also creates
selection risk; choosing the best historical result is not new forward evidence.

SHORT prices come from Spot candles as a proxy. Funding, borrowing availability,
liquidations, margin mechanics, order-book depth, changing spreads, partial fills
and intratrade drawdown are not simulated. An unleveraged paper short still has
large-loss exposure before a time exit. This is not a futures execution model.

## Persistence and module boundaries

Schema version 3 and the full normalized configuration—including inputs, triggers, TP,
symbol, timeframe, horizon, costs and threshold—identify each saved profile.
Reordering the same inputs resumes the same profile. Changing a setting starts
or resumes a different profile. There is no delete/reset or profile-browser UI.

Version 1 and 2 research remains untouched in its original directories and is not
mixed with the new labels or models. Legacy preferences load with the new
defaults, but automatic resume pauses on upgrade until inputs are reviewed and
Start is pressed. There is no migration of old model weights.

Each profile stores candles, directional models, checkpoints, training history,
forecasts and positions in research.sqlite. SQLite transactions and full
synchronous commits protect state. Worker status uses atomic JSON replacement.
The profile directory is shown in the module.

Rust supervises the fixed local Python worker using isolated interpreter
settings and no shell interpolation. A directory lock prevents two workers
from writing the research store simultaneously. The worker exits when its
parent closes. Grid calculations, Grid UI and Charts code are unchanged and
have no dependency on the new input or pattern logic. A separate read-only
chart subprocess never opens research databases. Its native command is async,
allows only one request in flight and does not hold the training supervisor
mutex. Chart polling pauses when the AI tab is hidden; learning continues.

Only public HTTPS GET requests to Binance time/klines endpoints are used.
Network failures retry and surface an error. Candle closure, continuity,
OHLC ranges and revisions are checked. Bounded catch-up never invents missed
signals, but previously scheduled paper positions are resolved from later
downloaded closed candles. Windows startup/service installation is not included.

## Validation and development

Run:

~~~powershell
.\ai-worker\runtime\python.exe -I ai-worker\test.py
node --test tests/modules.test.mjs tests/ai-settings.test.mjs
cargo test --workspace --locked
cargo tauri build --no-bundle -- --locked
~~~

An isolated live-data check:

~~~powershell
.\ai-worker\runtime\python.exe -I ai-worker\worker.py --data-dir target\ai-v3-live-smoke --once
~~~

See VALIDATION.md for the exact checks and limitations of this build.

For a browser preview with real read-only market data and a saved model report:

~~~powershell
node tools/preview.cjs --ai-report target/ai-v3-report.json --live-chart
~~~

The preview binds only to 127.0.0.1:4174 and labels itself as development.
Training and Grid mutation commands still require the desktop application.
