-- Bot Analytics Schema
-- Stores comprehensive metrics for all trading bots

-- Per-fill metrics (1 row per order fill)
CREATE TABLE IF NOT EXISTS fill_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL,
  fill_id TEXT NOT NULL UNIQUE,
  timestamp INTEGER NOT NULL,

  -- Basic fill data
  symbol TEXT NOT NULL,
  side TEXT NOT NULL, -- 'buy' or 'sell'
  quantity REAL NOT NULL,
  filled_price REAL NOT NULL,

  -- CATEGORIA 1: PER-FILL METRICS
  slippage_percent REAL, -- (expected vs actual) %
  profit_net REAL, -- after fees
  spread_paid REAL, -- bid/ask difference
  execution_time_ms INTEGER, -- milliseconds to confirm
  lot_size_percent REAL, -- % of capital allocated
  recovery_rate_percent REAL, -- % of prev drawdown covered
  directional_efficiency REAL, -- 0-100 score (at support/resistance)
  fill_velocity_seconds REAL, -- seconds from placement to fill
  orderbook_depth_impact REAL, -- % of orderbook consumed
  rolling_pnl_10fill REAL, -- PnL contribution to last 10 fills

  -- Derived for grid calculation
  grid_level INTEGER,
  grid_id TEXT,

  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Per-grid metrics (1 row per grid level per bot)
CREATE TABLE IF NOT EXISTS grid_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL,
  grid_id TEXT NOT NULL,
  grid_level INTEGER NOT NULL,
  symbol TEXT NOT NULL,

  -- CATEGORIA 2: PER-GRID METRICS
  win_rate_percent REAL, -- % of fills profitable
  avg_profit_per_grid REAL, -- gross avg profit per fill
  utilization_percent REAL, -- active vs inactive time
  grid_density INTEGER, -- fills per hour/day
  efficiency_score REAL, -- (profit / spread) * 100
  cumulative_volume REAL, -- total traded at this level
  stoploss_hits INTEGER, -- count of SL triggers
  reversal_frequency INTEGER, -- buy->sell direction changes
  grid_alpha REAL, -- profit vs overall bot avg
  churn_rate REAL, -- recreate/reposition frequency

  UNIQUE(bot_id, grid_id, grid_level),
  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Price action metrics (1 row per bot, updated continuously)
CREATE TABLE IF NOT EXISTS price_action_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,

  -- CATEGORIA 3: PRICE ACTION (8 metrics)
  buy_dip_efficiency REAL, -- % buys at local bottom
  sell_rip_efficiency REAL, -- % sells at local top
  volatility_adaptation_speed REAL, -- response time to ATR +20%
  trend_following_score REAL, -- profit in trend vs range
  breakout_sensitivity REAL, -- fills from breakout levels %
  false_breakout_filter_accuracy REAL, -- % fills that reverted
  price_distance_decay REAL, -- profit decay vs entry distance
  momentum_drift REAL, -- avg_buy_price - avg_sell_price

  UNIQUE(bot_id, timestamp),
  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Global bot metrics (1 row per bot, updated after each fill)
CREATE TABLE IF NOT EXISTS bot_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL UNIQUE,
  timestamp INTEGER NOT NULL,

  -- CATEGORIA 4: GLOBAL BOT (12 metrics)
  total_net_profit REAL, -- gross - fees - funding
  total_return_percent REAL, -- TNP / invested capital %
  win_rate_percent REAL, -- profitable fills / total
  profit_factor REAL, -- gross_profit / abs(gross_loss)
  avg_win_per_fill REAL,
  avg_loss_per_fill REAL,
  max_consecutive_wins INTEGER,
  max_consecutive_losses INTEGER,
  avg_trade_duration_seconds REAL,
  total_fills INTEGER,
  gross_exposure_percent REAL, -- capital in active orders %
  inventory_turnover REAL, -- volume / capital
  bot_uptime_percent REAL, -- active without errors %

  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Risk & Health metrics (1 row per bot, updated after each fill)
CREATE TABLE IF NOT EXISTS risk_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL UNIQUE,
  timestamp INTEGER NOT NULL,

  -- CATEGORIA 5: RISK & HEALTH (7 metrics)
  current_drawdown_percent REAL, -- % loss from last ATH
  max_drawdown_percent REAL, -- largest drawdown ever
  recovery_factor REAL, -- TNP / MDD
  sharpe_ratio REAL, -- (return / std_dev) * sqrt(days)
  calmar_ratio REAL, -- annual_return / MDD
  risk_of_ruin_percent REAL, -- probability to lose 30%
  correlation_to_btc REAL, -- 0=uncorrelated (ideal)

  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Value growth metrics (1 row per bot per day/week/month)
CREATE TABLE IF NOT EXISTS value_growth_metrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL,
  timeframe TEXT NOT NULL, -- 'hourly', 'daily', 'weekly', 'monthly'
  period_start INTEGER NOT NULL,
  period_end INTEGER NOT NULL,

  -- CATEGORIA 6: VALUE GROWTH (3 metrics)
  compounded_daily_return_percent REAL, -- CDR
  value_growth_adjusted REAL, -- growth - deposits/withdrawals
  profit_per_period REAL, -- average for this timeframe
  profit_std_dev REAL, -- volatility per period

  UNIQUE(bot_id, timeframe, period_start),
  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Hourly/Daily snapshots for charting
CREATE TABLE IF NOT EXISTS bot_equity_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bot_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  equity_value REAL, -- total account value at this time
  pnl REAL, -- profit/loss
  fills_count INTEGER, -- cumulative fills

  UNIQUE(bot_id, timestamp),
  FOREIGN KEY(bot_id) REFERENCES bots(id)
);

-- Create indices for fast queries
CREATE INDEX IF NOT EXISTS idx_fill_metrics_bot_id ON fill_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_fill_metrics_timestamp ON fill_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_grid_metrics_bot_id ON grid_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_price_action_bot_id ON price_action_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_metrics_bot_id ON bot_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_risk_metrics_bot_id ON risk_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_equity_history_bot_id ON bot_equity_history(bot_id, timestamp);
