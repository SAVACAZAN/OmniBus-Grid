//! Deterministic local paper bot. Fills happen at the configured limit price
//! when a supplied price touches it. This is a simulation, not an exchange.
use serde::{Deserialize, Serialize};
use crate::grid::{GridConfig, GridOrder, Side};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BotStatus { Running, Paused, Stopped }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus { Open, Filled, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub base_free: f64,
    pub base_locked: f64,
    pub quote_free: f64,
    pub quote_locked: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperOrder {
    pub id: u64,
    pub level: usize,
    pub side: Side,
    pub price: f64,
    pub amount: f64,
    pub total: f64,
    pub status: OrderStatus,
}

impl PaperOrder {
    fn from_grid(id: u64, o: &GridOrder) -> Self {
        Self { id, level: o.level, side: o.side, price: o.price,
            amount: o.amount, total: o.total, status: OrderStatus::Open }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub order_id: u64,
    pub at_ms: u64,
    pub side: Side,
    pub price: f64,
    pub amount: f64,
    pub fee_amount: f64,
    pub fee_asset: String,
    pub replacement_order_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperBot {
    pub id: u64,
    pub name: String,
    pub config: GridConfig,
    pub status: BotStatus,
    pub current_price: f64,
    pub initial_base: f64,
    pub initial_quote: f64,
    pub wallet: Wallet,
    pub orders: Vec<PaperOrder>,
    pub fills: Vec<Fill>,
    pub warnings: Vec<String>,
    pub created_ms: u64,
    next_order_id: u64,
}

fn balance(value: f64) -> bool { value.is_finite() && value >= 0.0 }

impl PaperBot {
    pub fn new(id: u64, name: String, config: GridConfig, current_price: f64,
               initial_base: f64, initial_quote: f64, created_ms: u64) -> Result<Self, String> {
        if name.trim().is_empty() || name.len() > 60 { return Err("Bot name must contain 1–60 characters".into()); }
        if !balance(initial_base) || !balance(initial_quote) || initial_base > 1e18 || initial_quote > 1e18 {
            return Err("Initial balances must be nonnegative, finite, and at most 1e18".into());
        }
        let preview = config.preview(current_price)?;
        if preview.buy_reserve_quote > initial_quote {
            return Err(format!("Need {:.6} quote for buy orders, available {:.6}", preview.buy_reserve_quote, initial_quote));
        }
        if preview.sell_reserve_base > initial_base {
            return Err(format!("Need {:.8} base for sell orders, available {:.8}", preview.sell_reserve_base, initial_base));
        }
        let orders = preview.orders.iter().enumerate().map(|(i, o)| PaperOrder::from_grid(i as u64 + 1, o)).collect();
        Ok(Self {
            id, name: name.trim().into(), config, status: BotStatus::Running,
            current_price, initial_base, initial_quote,
            wallet: Wallet { base_free: (initial_base - preview.sell_reserve_base).max(0.0),
                base_locked: preview.sell_reserve_base,
                quote_free: (initial_quote - preview.buy_reserve_quote).max(0.0),
                quote_locked: preview.buy_reserve_quote },
            orders, fills: Vec::new(), warnings: Vec::new(), created_ms,
            next_order_id: preview.buy_orders as u64 + preview.sell_orders as u64 + 1,
        })
    }

    pub fn equity_quote(&self) -> f64 {
        self.wallet.quote_free + self.wallet.quote_locked
            + (self.wallet.base_free + self.wallet.base_locked) * self.current_price
    }

    pub fn equity_change_quote(&self) -> f64 {
        self.equity_quote() - (self.initial_quote + self.initial_base * self.current_price)
    }

    pub fn open_count(&self) -> usize {
        self.orders.iter().filter(|o| o.status == OrderStatus::Open).count()
    }

    fn warn(&mut self, message: String) {
        self.warnings.push(message);
        if self.warnings.len() > 30 { self.warnings.remove(0); }
    }

    /// A tick processes only orders that existed before the tick. A large price
    /// gap may cross several levels. Newly posted inverses wait for a later tick.
    pub fn tick(&mut self, market_price: f64, at_ms: u64) -> Result<usize, String> {
        if self.status != BotStatus::Running { return Err("Bot must be running to process a price".into()); }
        if !market_price.is_finite() || market_price <= 0.0 { return Err("Price must be positive and finite".into()); }
        let candidates: Vec<usize> = self.orders.iter().enumerate()
            .filter(|(_, o)| o.status == OrderStatus::Open && match o.side {
                Side::Buy => market_price <= o.price,
                Side::Sell => market_price >= o.price,
            }).map(|(i, _)| i).collect();
        for index in &candidates { self.fill_order(*index, at_ms); }
        self.current_price = market_price;
        Ok(candidates.len())
    }

    fn fill_order(&mut self, index: usize, at_ms: u64) {
        let order = self.orders[index].clone();
        let fee_fraction = self.config.fee_pct / 100.0;
        self.orders[index].status = OrderStatus::Filled;
        let (fee_amount, fee_asset) = match order.side {
            Side::Buy => {
                self.wallet.quote_locked = (self.wallet.quote_locked - order.total).max(0.0);
                let fee = order.amount * fee_fraction;
                self.wallet.base_free += order.amount - fee;
                (fee, "base")
            }
            Side::Sell => {
                self.wallet.base_locked = (self.wallet.base_locked - order.amount).max(0.0);
                let fee = order.total * fee_fraction;
                self.wallet.quote_free += order.total - fee;
                (fee, "quote")
            }
        };

        let replacement = match order.side {
            Side::Buy => self.config.levels().get(order.level + 1).copied()
                .map(|price| (Side::Sell, price, order.amount - fee_amount, order.level + 1)),
            Side::Sell => order.level.checked_sub(1).and_then(|level| self.config.levels().get(level).copied()
                .map(|price| (Side::Buy, price, (order.total - fee_amount) / price, level))),
        };
        let replacement_id = replacement.and_then(|(side, price, proposed_amount, level)| {
            let proposed_total = price * proposed_amount;
            if !price.is_finite() || !proposed_amount.is_finite() || !proposed_total.is_finite()
                || price <= 0.0 || proposed_amount <= 0.0 {
                self.warn(format!("Order #{} filled; replacement amount was invalid", order.id));
                return None;
            }
            let available = if side == Side::Buy { self.wallet.quote_free } else { self.wallet.base_free };
            let required = if side == Side::Buy { proposed_total } else { proposed_amount };
            if required > available + 1e-8 {
                self.warn(format!("Order #{} filled; insufficient paper balance for inverse order", order.id));
                return None;
            }
            // Small floating-point differences are rounded to the available
            // balance, rather than creating a fractional asset from nothing.
            let (amount, total) = if side == Side::Buy {
                let total = proposed_total.min(available);
                (total / price, total)
            } else {
                let amount = proposed_amount.min(available);
                (amount, price * amount)
            };
            if side == Side::Buy {
                self.wallet.quote_free -= total;
                self.wallet.quote_locked += total;
            } else {
                self.wallet.base_free -= amount;
                self.wallet.base_locked += amount;
            }
            let id = self.next_order_id;
            self.next_order_id += 1;
            self.orders.push(PaperOrder { id, level, side, price, amount, total, status: OrderStatus::Open });
            Some(id)
        });
        if replacement.is_none() {
            self.warn(format!("Order #{} filled at range edge; no inverse grid level", order.id));
        }
        self.fills.push(Fill { order_id: order.id, at_ms, side: order.side,
            price: order.price, amount: order.amount, fee_amount,
            fee_asset: fee_asset.into(), replacement_order_id: replacement_id });
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if self.status != BotStatus::Running { return Err("Only a running bot can be paused".into()); }
        self.status = BotStatus::Paused;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if self.status != BotStatus::Paused { return Err("Only a paused bot can resume".into()); }
        self.status = BotStatus::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.status == BotStatus::Stopped { return Err("Bot is already stopped".into()); }
        for order in self.orders.iter_mut().filter(|o| o.status == OrderStatus::Open) {
            order.status = OrderStatus::Cancelled;
            if order.side == Side::Buy {
                self.wallet.quote_locked = (self.wallet.quote_locked - order.total).max(0.0);
                self.wallet.quote_free += order.total;
            } else {
                self.wallet.base_locked = (self.wallet.base_locked - order.amount).max(0.0);
                self.wallet.base_free += order.amount;
            }
        }
        self.status = BotStatus::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::test_config;

    #[test]
    fn requires_balances_and_reserves_both_assets() {
        assert!(PaperBot::new(1, "x".into(), test_config(), 100.0, 0.0, 300.0, 0).is_err());
        let b = PaperBot::new(1, "x".into(), test_config(), 100.0, 2.0, 300.0, 0).unwrap();
        assert_eq!(b.open_count(), 4);
        assert!((b.wallet.quote_locked - 200.0).abs() < 1e-9);
        assert!((b.wallet.base_free + b.wallet.base_locked - 2.0).abs() < 1e-9);
    }

    #[test]
    fn buy_fill_then_inverse_sell_keeps_balances_accounted() {
        let mut b = PaperBot::new(1, "x".into(), test_config(), 100.0, 2.0, 300.0, 0).unwrap();
        assert_eq!(b.tick(94.0, 1).unwrap(), 1); // buy at 95
        assert_eq!(b.fills[0].side, Side::Buy);
        let replacement = b.orders.iter().find(|o| o.id == b.fills[0].replacement_order_id.unwrap()).unwrap();
        assert_eq!(replacement.side, Side::Sell);
        assert_eq!(replacement.price, 100.0);
        assert!((replacement.amount - (100.0 / 95.0 * 0.999)).abs() < 1e-9);
        assert_eq!(b.tick(101.0, 2).unwrap(), 1); // inverse sell at 100
        assert_eq!(b.fills[1].side, Side::Sell);
        assert!(b.wallet.quote_free >= 0.0 && b.wallet.base_free >= 0.0);
        let total_base = b.wallet.base_free + b.wallet.base_locked;
        let total_quote = b.wallet.quote_free + b.wallet.quote_locked;
        assert!((total_base - 2.0).abs() < 1e-8);
        assert!(total_quote > 300.0);
    }

    #[test]
    fn pause_stop_and_json_round_trip() {
        let mut b = PaperBot::new(2, "test".into(), test_config(), 100.0, 2.0, 300.0, 0).unwrap();
        b.pause().unwrap();
        assert!(b.tick(90.0, 1).is_err());
        b.resume().unwrap();
        b.stop().unwrap();
        assert_eq!(b.open_count(), 0);
        assert_eq!((b.wallet.base_locked, b.wallet.quote_locked), (0.0, 0.0));
        let mut restored: PaperBot = serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        assert_eq!(restored.status, BotStatus::Stopped);
        assert!(restored.tick(90.0, 2).is_err());
    }
}
