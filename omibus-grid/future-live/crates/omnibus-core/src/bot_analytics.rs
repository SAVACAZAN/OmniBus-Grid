use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillMetrics {
    pub bot_id: String,
    pub fill_id: String,
    pub timestamp: i64,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub filled_price: f64,

    // PER-FILL METRICS
    pub slippage_percent: f64,
    pub profit_net: f64,
    pub spread_paid: f64,
    pub execution_time_ms: i64,
    pub lot_size_percent: f64,
    pub recovery_rate_percent: f64,
    pub directional_efficiency: f64,
    pub fill_velocity_seconds: f64,
    pub orderbook_depth_impact: f64,
    pub rolling_pnl_10fill: f64,

    pub grid_level: Option<i32>,
    pub grid_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridMetrics {
    pub bot_id: String,
    pub grid_id: String,
    pub grid_level: i32,
    pub symbol: String,

    // PER-GRID METRICS
    pub win_rate_percent: f64,
    pub avg_profit_per_grid: f64,
    pub utilization_percent: f64,
    pub grid_density: i32,
    pub efficiency_score: f64,
    pub cumulative_volume: f64,
    pub stoploss_hits: i32,
    pub reversal_frequency: i32,
    pub grid_alpha: f64,
    pub churn_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceActionMetrics {
    pub bot_id: String,
    pub timestamp: i64,

    // PRICE ACTION METRICS (8)
    pub buy_dip_efficiency: f64,
    pub sell_rip_efficiency: f64,
    pub volatility_adaptation_speed: f64,
    pub trend_following_score: f64,
    pub breakout_sensitivity: f64,
    pub false_breakout_filter_accuracy: f64,
    pub price_distance_decay: f64,
    pub momentum_drift: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMetrics {
    pub bot_id: String,
    pub timestamp: i64,

    // GLOBAL BOT METRICS (12)
    pub total_net_profit: f64,
    pub total_return_percent: f64,
    pub win_rate_percent: f64,
    pub profit_factor: f64,
    pub avg_win_per_fill: f64,
    pub avg_loss_per_fill: f64,
    pub max_consecutive_wins: i32,
    pub max_consecutive_losses: i32,
    pub avg_trade_duration_seconds: f64,
    pub total_fills: i32,
    pub gross_exposure_percent: f64,
    pub inventory_turnover: f64,
    pub bot_uptime_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub bot_id: String,
    pub timestamp: i64,

    // RISK & HEALTH METRICS (7)
    pub current_drawdown_percent: f64,
    pub max_drawdown_percent: f64,
    pub recovery_factor: f64,
    pub sharpe_ratio: f64,
    pub calmar_ratio: f64,
    pub risk_of_ruin_percent: f64,
    pub correlation_to_btc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueGrowthMetrics {
    pub bot_id: String,
    pub timeframe: String, // 'hourly', 'daily', 'weekly', 'monthly'
    pub period_start: i64,
    pub period_end: i64,

    // VALUE GROWTH METRICS (3+)
    pub compounded_daily_return_percent: f64,
    pub value_growth_adjusted: f64,
    pub profit_per_period: f64,
    pub profit_std_dev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquitySnapshot {
    pub bot_id: String,
    pub timestamp: i64,
    pub equity_value: f64,
    pub pnl: f64,
    pub fills_count: i32,
}

/// Calculate all metrics for a bot given its fill history
pub fn calculate_bot_metrics(
    bot_id: &str,
    fills: &[FillMetrics],
    initial_capital: f64,
) -> BotMetrics {
    let total_fills = fills.len() as i32;
    let mut total_profit = 0.0;
    let mut winning_fills = 0;
    let mut losing_fills = 0;
    let mut sum_wins = 0.0;
    let mut sum_losses = 0.0;
    let mut max_consec_wins = 0;
    let mut max_consec_losses = 0;
    let mut current_consec_wins = 0;
    let mut current_consec_losses = 0;
    let mut sum_durations = 0.0;
    let mut total_volume = 0.0;

    for fill in fills {
        total_profit += fill.profit_net;

        if fill.profit_net > 0.0 {
            winning_fills += 1;
            sum_wins += fill.profit_net;
            current_consec_wins += 1;
            current_consec_losses = 0;
        } else {
            losing_fills += 1;
            sum_losses += fill.profit_net.abs();
            current_consec_losses += 1;
            current_consec_wins = 0;
        }

        max_consec_wins = max_consec_wins.max(current_consec_wins);
        max_consec_losses = max_consec_losses.max(current_consec_losses);

        sum_durations += fill.fill_velocity_seconds;
        total_volume += fill.quantity * fill.filled_price;
    }

    let win_rate = if total_fills > 0 {
        (winning_fills as f64 / total_fills as f64) * 100.0
    } else {
        0.0
    };

    let profit_factor = if sum_losses > 0.0 && total_fills > 0 {
        sum_wins / sum_losses
    } else if total_fills > 0 && winning_fills > 0 {
        1.0
    } else {
        0.0
    };

    let avg_win = if winning_fills > 0 {
        sum_wins / winning_fills as f64
    } else {
        0.0
    };

    let avg_loss = if losing_fills > 0 {
        sum_losses / losing_fills as f64
    } else {
        0.0
    };

    let avg_duration = if total_fills > 0 {
        sum_durations / total_fills as f64
    } else {
        0.0
    };

    let total_return = if initial_capital > 0.0 {
        (total_profit / initial_capital) * 100.0
    } else {
        0.0
    };

    let inventory_turnover = if initial_capital > 0.0 {
        total_volume / initial_capital
    } else {
        0.0
    };

    BotMetrics {
        bot_id: bot_id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,

        total_net_profit: total_profit,
        total_return_percent: total_return,
        win_rate_percent: win_rate,
        profit_factor,
        avg_win_per_fill: avg_win,
        avg_loss_per_fill: avg_loss,
        max_consecutive_wins,
        max_consecutive_losses,
        avg_trade_duration_seconds: avg_duration,
        total_fills,
        gross_exposure_percent: 0.0, // calculated separately
        inventory_turnover,
        bot_uptime_percent: 100.0, // calculated from errors
    }
}

/// Calculate risk metrics
pub fn calculate_risk_metrics(
    bot_id: &str,
    equity_history: &[EquitySnapshot],
    fills: &[FillMetrics],
) -> RiskMetrics {
    let mut max_equity = 0.0;
    let mut current_dd = 0.0;
    let mut max_dd = 0.0;

    let mut returns = Vec::new();

    for snapshot in equity_history {
        if snapshot.equity_value > max_equity {
            max_equity = snapshot.equity_value;
        }

        let dd = ((max_equity - snapshot.equity_value) / max_equity * 100.0).max(0.0);
        if dd > max_dd {
            max_dd = dd;
        }
        current_dd = dd;

        returns.push(snapshot.pnl);
    }

    let total_profit = fills.iter().map(|f| f.profit_net).sum::<f64>();
    let recovery_factor = if max_dd > 0.0 {
        total_profit / (max_dd / 100.0)
    } else {
        0.0
    };

    // Sharpe Ratio calculation
    let sharpe = if !returns.is_empty() {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            (mean_return / std_dev) * (252.0_f64.sqrt()) // annualized
        } else {
            0.0
        }
    } else {
        0.0
    };

    RiskMetrics {
        bot_id: bot_id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,

        current_drawdown_percent: current_dd,
        max_drawdown_percent: max_dd,
        recovery_factor,
        sharpe_ratio: sharpe,
        calmar_ratio: if max_dd > 0.0 {
            (total_profit * 252.0) / max_dd // annualized return / MDD
        } else {
            0.0
        },
        risk_of_ruin_percent: calculate_risk_of_ruin(&returns),
        correlation_to_btc: 0.0, // requires external price data
    }
}

fn calculate_risk_of_ruin(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 || mean <= 0.0 {
        return 50.0; // no edge
    }

    let ruin_threshold = -0.3; // 30% loss
    let normalized = (ruin_threshold - mean) / std_dev;
    let prob = (1.0 + (normalized / (1.0 + normalized.abs())).erf()) / 2.0; // approximation
    (prob * 100.0).min(99.0)
}

/// Calculate per-grid metrics
pub fn calculate_grid_metrics(
    bot_id: &str,
    grid_id: &str,
    grid_level: i32,
    symbol: &str,
    fills_at_level: &[FillMetrics],
) -> GridMetrics {
    let total_fills = fills_at_level.len();
    let winning = fills_at_level
        .iter()
        .filter(|f| f.profit_net > 0.0)
        .count();

    let win_rate = if total_fills > 0 {
        (winning as f64 / total_fills as f64) * 100.0
    } else {
        0.0
    };

    let avg_profit = if total_fills > 0 {
        fills_at_level
            .iter()
            .map(|f| f.profit_net)
            .sum::<f64>()
            / total_fills as f64
    } else {
        0.0
    };

    let total_volume = fills_at_level
        .iter()
        .map(|f| f.quantity * f.filled_price)
        .sum::<f64>();

    GridMetrics {
        bot_id: bot_id.to_string(),
        grid_id: grid_id.to_string(),
        grid_level,
        symbol: symbol.to_string(),

        win_rate_percent: win_rate,
        avg_profit_per_grid: avg_profit,
        utilization_percent: 75.0, // would need time tracking
        grid_density: 0, // requires time period context
        efficiency_score: if total_volume > 0.0 {
            (avg_profit / total_volume) * 10000.0
        } else {
            0.0
        },
        cumulative_volume: total_volume,
        stoploss_hits: 0, // track separately
        reversal_frequency: 0, // track separately
        grid_alpha: 0.0, // vs overall bot
        churn_rate: 0.0, // track separately
    }
}

pub fn calculate_value_growth_metrics(
    bot_id: &str,
    equity_history: &[EquitySnapshot],
    timeframe: &str,
) -> Option<ValueGrowthMetrics> {
    if equity_history.is_empty() {
        return None;
    }

    let first = equity_history.first().unwrap();
    let last = equity_history.last().unwrap();

    let total_growth = last.equity_value - first.equity_value;
    let periods = match timeframe {
        "daily" => 1.0,
        "weekly" => 7.0,
        "monthly" => 30.0,
        "hourly" => 1.0 / 24.0,
        _ => 1.0,
    };

    let cdr = ((last.equity_value / first.equity_value).powf(1.0 / periods) - 1.0) * 100.0;

    Some(ValueGrowthMetrics {
        bot_id: bot_id.to_string(),
        timeframe: timeframe.to_string(),
        period_start: first.timestamp,
        period_end: last.timestamp,

        compounded_daily_return_percent: cdr,
        value_growth_adjusted: total_growth,
        profit_per_period: total_growth / periods,
        profit_std_dev: 0.0, // calculate from returns
    })
}
