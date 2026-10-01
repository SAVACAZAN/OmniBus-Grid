//! Exchange independent grid planning and local paper execution.
pub mod grid;
pub mod paper;

pub use grid::{AmountType, AmountUnit, GridConfig, GridOrder, GridPreview, GridType, OrderSide, Side};
pub use paper::{BotStatus, Fill, PaperBot, PaperOrder, Wallet};
