use serde::Serialize;
use tabled::Tabled;

pub const DEFAULT_CATEGORIES: &[&str] = &[
    "dining",
    "travel",
    "groceries",
    "transport",
    "shopping",
    "entertainment",
];

pub const DEFAULT_PAYMENT_CATEGORIES: &[&str] = &[
    "contactless",
    "mobile contactless",
    "online",
];

fn display_option_f64(val: &Option<f64>) -> String {
    match val {
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Tabled)]
pub struct Card {
    pub id: i64,
    pub name: String,
    /// JSON array of spending categories (e.g. ["dining", "travel"])
    pub categories: String,
    /// JSON array of payment categories (e.g. ["contactless", "online"])
    pub payment_categories: String,
    pub miles_per_dollar: f64,
    /// Miles per dollar for foreign currency transactions (defaults to miles_per_dollar)
    #[tabled(display_with = "display_option_f64")]
    pub miles_per_dollar_foreign: Option<f64>,
    pub block_size: f64,
    pub statement_renewal_date: i32,
    #[tabled(display_with = "display_option_f64")]
    pub max_reward_limit: Option<f64>,
    #[tabled(display_with = "display_option_f64")]
    pub min_spend: Option<f64>,
}

/// Used for the "best-card" query result
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct CardRecommendation {
    pub card_name: String,
    pub miles_per_dollar: f64,
    pub block_size: f64,
    pub effective_rate: f64,
}

#[derive(Debug, Clone, Serialize, Tabled)]
pub struct Spending {
    pub id: i64,
    pub card_id: i64,
    pub amount: f64,
    pub category: String,
    /// YYYY-MM-DD
    pub date: String,
    /// Miles earned from this transaction
    pub miles_earned: f64,
}
