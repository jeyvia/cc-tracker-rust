use serde::Serialize;
use tabled::Tabled;

pub const DEFAULT_CATEGORIES: &[&str] = &[
    "dining",
    "travel",
    "groceries",
    "transport",
    "shopping",
    "entertainment"
];

#[derive(Debug, Clone, Serialize, Tabled)]
pub struct Card {
    pub id: i64,
    pub name: String,
    /// JSON array of categories (e.g. ["dining", "travel"])
    pub categories: String,
    pub miles_per_dollar: f64,
    /// Spending block size in dollars (e.g. 1 or 5)
    pub block_size: f64,
    /// Day of month the statement renews (1-31)
    pub statement_renewal_date: i32,
    /// Maximum reward limit per statement cycle (0 = unlimited)
    pub max_reward_limit: f64,
    /// Minimum spend required to earn rewards
    pub min_spend: f64,
}

/// Used for the "best-card" query result
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct CardRecommendation {
    pub card_name: String,
    pub miles_per_dollar: f64,
    pub block_size: f64,
    /// Effective miles per $1 spent (miles_per_dollar / block_size)
    pub effective_rate: f64,
}
