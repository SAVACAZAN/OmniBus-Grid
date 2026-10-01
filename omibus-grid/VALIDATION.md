# Validation status

## Current release: AI Research 3 — 2026-09-29

- 32 Python tests passed. Coverage includes a positive synthetic example for every one of the 20 formation detectors, causal prefixes, forming-versus-confirmed states, all 20 indicator panels, RSI/EMA comparisons, AND conditions, unselected-input rejection, and provisional candles being unable to confirm formations or fire rules. Existing delayed labels, LONG/SHORT/TP accounting, restart persistence and model promotion checks pass.
- 13 Rust tests passed, including all six existing Grid/core tests, module switches, selected input validation, eight supported timeframes and custom-rule validation.
- Nine JavaScript tests passed, including matching paper markers to the correct market/timeframe/inputs/rules and all eight module configurations. Frontend syntax checks passed.
- `cargo tauri build --no-bundle -- --locked` completed successfully. Both `Omibus Grid Modular.exe` and `Omibus Grid Research.exe` match the release executable SHA-256: `EE0D165A43EF2B8641AB4B872EB739630928745FD17FFE8681D47CC4692B11F9`.
- The old Modular window was closed gracefully and the updated Modular executable reopened. Its observed title is **Omibus Grid · AI Research 3**. The existing `paper-bots.json` checksum is unchanged before/after replacement. The release verification record is `target/ai-v3-release-validation.json`.
- Public Binance chart smoke test returned 600 BTCUSDT 1m candles, a provisional current candle and RSI/EMA panels. Browser QA used real chart requests and verified RSI-only, EMA-only, numeric RSI crossings, EMA-to-EMA crossings, automatic removal of deselected indicators' rules, timeframe changes, formation labels, zoom/history navigation and selection retention across Grid/AI tab switches. Real 1h data displayed double-top, symmetrical-triangle and rising-wedge breakouts. No browser warnings/errors were recorded after the final reload.
- The independent training smoke test used `target/ai-v3-live-smoke`: 3,000 closed BTCUSDT 1h candles and 2,895 training examples per side. Holdout Brier was 0.192852 versus 0.191254 for the fixed base-rate baseline, with zero entries at threshold 0.60. It did not establish predictive advantage. The user's research store was not used for the smoke test.
- Native startup was verified, but native Start/Stop clicks were not automated. Browser training controls are deliberately disabled; it exercises the same frontend and Python chart reader through a development bridge, not the native Tauri IPC path.
- This is approximately five-second chart polling, with completed-candle research checks every 15 seconds. Formations use geometric heuristics; synthetic fixtures are functional checks, not a real-world precision/recall study. Weekly charts can be available while weekly training lacks the required 800 labeled examples. All entries/exits remain paper simulations.

## Historical validation notes

The frontend JavaScript passed `node --check`. Cargo manifests and Tauri JSON parsed successfully. The HTML IDs used by the UI and all six invoked Rust command names were cross-checked. A scan of the new tree found no API credentials or private keys.

`cargo test` and a Tauri desktop build could not run in the creation environment because the Rust toolchain and webview build dependencies were unavailable. The core includes unit tests for grid levels, balance reservations, fill/repost accounting, lifecycle, and serialization. Run `cargo test -p omibus-core` and `cargo tauri dev` on a configured machine before relying on the app. No Windows installer is included in this source ZIP.
# Modular workspace validation

- `cargo test --workspace --locked`: 10 Rust tests passed (6 engine tests, 4 module configuration tests).
- `node --test tests/modules.test.mjs`: 5 lifecycle/configuration/interface tests passed.
- Browser preview: verified both modules, Grid only, Charts only, and neither enabled. Grid and Charts layouts render independently. Configuration was restored to both enabled after testing.
- TradingView's external frame timed out during this validation session; the timeout/help state appeared. Live chart data was not reverified in this session.
- Runtime configuration is loaded from `modules.json` beside the executable. Disabling Grid skips store initialization; saved paper data is retained.
- Native desktop interaction was not automated in this session. The Windows release build is the packaging check; browser preview cannot execute native paper commands.
# AI Research validation

- 11 Python tests passed: causal feature prefixes, flat markets, candle validation, delayed labels, chronological holdout separation, daily training idempotence, restart continuity, stale/slow-run entry rejection, non-overlapping accounting, and conservative promotion gates.
- 11 Rust tests and 6 JavaScript tests passed, including all eight module configurations and restricted module command interfaces.
- Real public Binance smoke test: 3,000 closed BTCUSDT 1h candles, 2,895 learner updates, 579 historical holdout examples. Simple price/volume features won validation over the full indicator set. Holdout Brier ~0.201 versus ~0.198 for the base-rate predictor; no trades at threshold 0.60. This run did not establish predictive advantage or trading profitability.
- Browser visual QA rendered this real saved report through an explicit preview backend. Native desktop Start/Stop controls were not click-tested in that preview.
- Portable CPython 3.14.7 downloaded from python.org; SHA-256 matched the release page before extraction.
- Both existing modules remain enabled; AI Research is enabled after validation, with the worker stopped until the user presses Start.

# AI Research version 2 — 2026-09-28

- 22 Python tests passed. New coverage checks every selectable input's exact feature set, RSI-only training without substitution, canonical profile identity, all four synthetic formations, missing-breakout rejection, incorrect geometry, causal prefix invariance, LONG/SHORT decisions, TP after costs, time exits, stale-provider handling, and persistent single-position accounting across restart. Existing delayed-label, holdout, daily-update and promotion checks also pass.
- 12 Rust tests passed, including all 6 unchanged Grid/core tests, module switches, input whitelist/duplicate validation, TP bounds and reading legacy preferences.
- 8 JavaScript tests passed, including exact form selections, empty-input rejection, profile matching across reordered selections, and existing module isolation/lifecycle tests.
- The Windows release built successfully with `cargo tauri build --no-bundle -- --locked`.
- Public Binance check: 3,000 completed BTCUSDT 1h candles, 2,895 training examples per side, seven selected numerical features and seven historical formation detections. The default profile's 579-example holdout Brier was 0.192747 versus 0.191240 for the fixed base-rate baseline, with zero entries at threshold 0.60. The first recorded forward action was WAIT. No trading advantage was established.
- Browser verification using that real saved report: default input selection, RSI-only (one feature), EMA-only (three features), a custom combination, empty-selection validation, selection retention across Grid/AI tab switches, and the annotated historical head-and-shoulders chart all worked. No browser console errors appeared after the final UI reload. Browser controls use a preview backend; native desktop Start/Stop clicks were not automated.
- No Grid engine, Grid UI, Charts module or shared navigation code was changed. Version 1 AI databases remain separate from version 2 profiles. No user research worker was started by validation; the network check used `target/ai-v2-live-smoke`.
- The previous `Omibus Grid Modular.exe` was running, so the new release is delivered as `Omibus Grid Research.exe`. Close the old app before opening the new build. It requires the adjacent `ai-worker` folder.
