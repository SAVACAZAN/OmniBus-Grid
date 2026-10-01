//! Focused extraction of the original `src-tauri/src/grid.rs` grid rules.
//! Price spacing and per-level sizing retain their original semantics; all
//! inputs are validated before order generation.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side { Buy, Sell }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GridType { Linear, Geometric }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AmountType { PerGrid, Total, Incremental }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AmountUnit { Base, Quote }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide { Both, BuyOnly, SellOnly }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub symbol: String,
    pub lower_price: f64,
    pub upper_price: f64,
    pub nr_of_grids: usize,
    pub amount: f64,
    pub amount_type: AmountType,
    pub amount_unit: AmountUnit,
    pub incremental_pct_buy: f64,
    pub incremental_pct_sell: f64,
    pub side: OrderSide,
    pub grid_type: GridType,
    pub fee_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOrder {
    pub level: usize,
    pub side: Side,
    pub price: f64,
    pub amount: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPreview {
    pub levels: Vec<f64>,
    pub orders: Vec<GridOrder>,
    pub buy_orders: usize,
    pub sell_orders: usize,
    pub buy_reserve_quote: f64,
    pub sell_reserve_base: f64,
    pub avg_buy_price: f64,
    pub avg_sell_price: f64,
}

fn positive(value: f64) -> bool { value.is_finite() && value > 0.0 }

impl GridConfig {
    pub fn validate(&self, current_price: f64) -> Result<(), String> {
        if self.symbol.trim().is_empty() || self.symbol.len() > 32 {
            return Err("Symbol must contain 1–32 characters".into());
        }
        if !positive(self.lower_price) || !positive(self.upper_price) || self.lower_price >= self.upper_price {
            return Err("Enter positive bounds with lower price below upper price".into());
        }
        if !positive(current_price) { return Err("Current price must be positive and finite".into()); }
        if self.lower_price > 1e15 || self.upper_price > 1e15 || current_price > 1e15 {
            return Err("Prices above 1e15 are not supported".into());
        }
        if !(2..=200).contains(&self.nr_of_grids) { return Err("Grid count must be between 2 and 200".into()); }
        if !positive(self.amount) || self.amount > 1e15 { return Err("Amount must be positive, finite, and at most 1e15".into()); }
        if !self.fee_pct.is_finite() || !(0.0..=5.0).contains(&self.fee_pct) {
            return Err("Fee must be between 0% and 5%".into());
        }
        for (label, pct) in [("Buy increment", self.incremental_pct_buy), ("Sell increment", self.incremental_pct_sell)] {
            if !pct.is_finite() || !(0.0..=100.0).contains(&pct) {
                return Err(format!("{label} must be between 0% and 100%"));
            }
        }
        Ok(())
    }

    pub fn levels(&self) -> Vec<f64> {
        let n = self.nr_of_grids;
        (0..=n).map(|i| match self.grid_type {
            GridType::Linear => self.lower_price + (self.upper_price - self.lower_price) * i as f64 / n as f64,
            GridType::Geometric => self.lower_price * (self.upper_price / self.lower_price).powf(i as f64 / n as f64),
        }).collect()
    }

    fn quantity(&self, price: f64, side: Side, distance: usize, reference: f64) -> f64 {
        let divisor = if self.amount_unit == AmountUnit::Base { 1.0 } else { price };
        match self.amount_type {
            AmountType::PerGrid => self.amount / divisor,
            // Kept from the original grid: total is divided by the number of
            // intervals, even if a level is skipped at the current price.
            AmountType::Total => self.amount / self.nr_of_grids as f64 / divisor,
            AmountType::Incremental => {
                let pct = if side == Side::Buy { self.incremental_pct_buy } else { self.incremental_pct_sell };
                let ref_divisor = if self.amount_unit == AmountUnit::Base { 1.0 } else { reference };
                self.amount / ref_divisor * (1.0 + pct * distance as f64 / 100.0)
            }
        }
    }

    pub fn preview(&self, current_price: f64) -> Result<GridPreview, String> {
        self.validate(current_price)?;
        let levels = self.levels();
        if levels.iter().any(|p| !positive(*p)) { return Err("Grid prices overflowed".into()); }
        let buys: Vec<usize> = (0..levels.len()).filter(|&i| levels[i] < current_price && self.side != OrderSide::SellOnly).rev().collect();
        let sells: Vec<usize> = (0..levels.len()).filter(|&i| levels[i] > current_price && self.side != OrderSide::BuyOnly).collect();
        let mut orders = Vec::with_capacity(buys.len() + sells.len());
        let buy_ref = buys.first().map(|&i| levels[i]).unwrap_or(current_price);
        let sell_ref = sells.first().map(|&i| levels[i]).unwrap_or(current_price);
        let mut buy_reserve_quote = 0.0;
        let mut sell_reserve_base = 0.0;
        let mut buy_amount = 0.0;
        let mut sell_notional = 0.0;
        for (distance, level) in buys.iter().copied().enumerate() {
            let price = levels[level];
            let amount = self.quantity(price, Side::Buy, distance, buy_ref);
            let total = price * amount;
            if !positive(amount) || !positive(total) { return Err("Calculated buy amount overflowed".into()); }
            buy_reserve_quote += total;
            buy_amount += amount;
            orders.push(GridOrder { level, side: Side::Buy, price, amount, total });
        }
        for (distance, level) in sells.iter().copied().enumerate() {
            let price = levels[level];
            let amount = self.quantity(price, Side::Sell, distance, sell_ref);
            let total = price * amount;
            if !positive(amount) || !positive(total) { return Err("Calculated sell amount overflowed".into()); }
            sell_reserve_base += amount;
            sell_notional += total;
            orders.push(GridOrder { level, side: Side::Sell, price, amount, total });
        }
        if !buy_reserve_quote.is_finite() || !sell_reserve_base.is_finite()
            || !buy_amount.is_finite() || !sell_notional.is_finite() {
            return Err("Required balance overflowed".into());
        }
        if orders.is_empty() { return Err("No grid orders at this market price".into()); }
        Ok(GridPreview {
            levels, orders,
            buy_orders: buys.len(), sell_orders: sells.len(),
            buy_reserve_quote, sell_reserve_base,
            avg_buy_price: if buy_amount > 0.0 { buy_reserve_quote / buy_amount } else { 0.0 },
            avg_sell_price: if sell_reserve_base > 0.0 { sell_notional / sell_reserve_base } else { 0.0 },
        })
    }
}

#[cfg(test)]
pub(crate) fn test_config() -> GridConfig {
    GridConfig { symbol: "BTC/USDT".into(), lower_price: 90.0, upper_price: 110.0,
        nr_of_grids: 4, amount: 100.0, amount_type: AmountType::PerGrid,
        amount_unit: AmountUnit::Quote, incremental_pct_buy: 0.0,
        incremental_pct_sell: 0.0, side: OrderSide::Both,
        grid_type: GridType::Linear, fee_pct: 0.1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_split_and_reservations() {
        let p = test_config().preview(100.0).unwrap();
        assert_eq!(p.levels, vec![90.0, 95.0, 100.0, 105.0, 110.0]);
        assert_eq!((p.buy_orders, p.sell_orders), (2, 2));
        assert_eq!(p.orders.iter().map(|o| o.level).collect::<Vec<_>>(), vec![1, 0, 3, 4]);
        assert!((p.buy_reserve_quote - 200.0).abs() < 1e-9);
        assert!((p.sell_reserve_base - (100.0 / 105.0 + 100.0 / 110.0)).abs() < 1e-9);
    }

    #[test]
    fn geometric_and_incremental() {
        let mut c = test_config();
        c.lower_price = 100.0; c.upper_price = 1600.0; c.grid_type = GridType::Geometric;
        c.amount_type = AmountType::Incremental; c.incremental_pct_buy = 10.0;
        let p = c.preview(500.0).unwrap();
        for (actual, expected) in p.levels.iter().zip([100.0, 200.0, 400.0, 800.0, 1600.0]) {
            assert!((*actual - expected).abs() < 1e-8);
        }
        assert!(p.orders[1].amount > p.orders[0].amount);
    }

    #[test]
    fn rejects_invalid_and_non_finite() {
        let mut c = test_config(); c.nr_of_grids = 0;
        assert!(c.preview(100.0).is_err());
        c.nr_of_grids = 4; c.amount = f64::NAN;
        assert!(c.preview(100.0).is_err());
        c.amount = 100.0; c.lower_price = 110.0;
        assert!(c.preview(100.0).is_err());
        c.lower_price = 90.0; c.fee_pct = f64::INFINITY;
        assert!(c.preview(100.0).is_err());
    }
}
