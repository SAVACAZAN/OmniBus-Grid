# Handoff to Codex on my Windows PC

This is a project handoff prepared from our chat on 24 September 2026. It is a summary of decisions and work, **not a verbatim export of every message**. Please read the local application files before editing them. Ask me for the folder if it is not already open.

## What I want now

Work directly in the application folder I have opened for you on my Windows PC. Read the project, explain what it currently does in clear steps, and update its files there. Do not start a different application or send repeated ZIP downloads. Start by confirming which folder and codebase you are looking at. Keep the trading research app distinct from the unrelated Omibus/Tauri app.

I am interested in a research system that learns from historical cryptocurrency candle patterns. I want one specialist per indicator (initially RSI and Bollinger Bands), with their results combined by a decision agent. The system should support choosing a pair and timeframe, use public historical Binance candle data or offline CSV, test chronologically without future data leakage, and log paper signals. I do not want exchange account connections, API keys, or live order placement at this stage. I want plain-language explanations and one step at a time when setting up Windows tools.

## Standalone Python app created in this chat

Name: `crypto_pattern_agents` / Crypto Pattern Agents. Python 3.10+ and standard library only; no Cargo, Visual Studio, C++, or pip packages required. The first distributed archive was `Crypto_Pattern_Agents.zip`. It was a separate app, not Omibus.

Initial files:

- `market.py`: retrieves closed Binance Spot candles from the public market data endpoint; CSV import/export.
- `agents.py`: RSI and Bollinger features, historical nearest-neighbor memory, opinions, and a decision rule requiring both to approve BUY; otherwise WAIT.
- `run.py`: `fetch`, `backtest`, and `watch` commands. Chronological 70/30 frozen-memory backtest; paper signal journal only.
- `tests/test_agents.py`, `README.md`, `RUN_BACKTEST.bat`, `RUN_PAPER_ONCE.bat`.

Default hypothetical entry is the next candle's open; exit after 12 bars. Estimated fees and slippage are included. No actual trading functionality, profitability guarantee, or autonomous reasoning. The original 5 tests passed. Live public API fetch was not verified in the cloud environment.

Initial Windows commands from the extracted folder:

```cmd
py -3 run.py backtest --symbol BTCUSDT --interval 1h --bars 3000
py -3 run.py watch --once --symbol BTCUSDT --interval 1h --bars 3000
```

## New learner written after the first archive

A later request asked whether models could update themselves. In the cloud workspace I added `online.py`: a simple online logistic classifier combining RSI and Bollinger feature vectors. It makes one training update per historical example only after the outcome has matured. I added `run.py learn` for a walk-forward test, optional `run.py watch --once --learn` for a separate current probability, documentation, and two timing/evaluation tests. The model's score does **not** change the original BUY/WAIT decision. The updated local cloud code passed 7 unit tests.

**Important:** these later changes were *not* placed in the previously downloaded ZIP and were *not* installed in my Windows folder. Inspect the actual local files first. If `online.py` or the `learn` command is absent, implement and validate those updates directly in my local folder. Use delayed labels, evaluate on later candles, and keep the learner's output separate until it demonstrates value. Running `watch` should remain paper-only. Do not claim the learner reasons like an LLM or that its probabilities are calibrated or profitable merely because it updates.

Suggested validation commands from the app folder:

```cmd
py -3 -m unittest discover -s tests -v
py -3 run.py learn --help
```

## Earlier unrelated Omibus episode

We had worked on `omibus-grid`, a separate Rust/Tauri grid/core application. I explicitly said to forget Omibus for the indicator-agent discussion. Earlier Windows setup problems included missing `cargo`, later missing MSVC `link.exe`, and a Tauri dependency version conflict. None of those are prerequisites for this Python research app. Please do not merge these codebases or revive Kraken/Coinbase connections.

## Working style

I want you to edit the opened Windows folder directly, like Claude editing local files. Tell me exactly which folder is open, what you changed, how you tested it, and what command I should run next. If you only have a cloud copy, say so plainly. Do not confuse installing the ChatGPT desktop app with granting a browser chat access to local files.
