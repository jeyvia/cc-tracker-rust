mod db;
mod models;

use clap::{Parser, Subcommand};
use models::{DEFAULT_CATEGORIES, DEFAULT_PAYMENT_CATEGORIES};
use tabled::Table;

/// Credit Card Miles Tracker — find the best card for every purchase
#[derive(Parser)]
#[command(name = "cc-tracker", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new credit card
    AddCard {
        /// Card name (e.g. "Chase Sapphire Preferred")
        #[arg(long)]
        name: String,
        /// Spending categories this card earns on (omit for all categories)
        #[arg(long, num_args = 1..)]
        category: Vec<String>,
        /// Payment categories: contactless, "mobile contactless", online (omit for all)
        #[arg(long, num_args = 1..)]
        payment_category: Vec<String>,
        /// Miles earned per spending block
        #[arg(long)]
        miles_per_dollar: f64,
        /// Miles per dollar for foreign currency (defaults to miles-per-dollar if omitted)
        #[arg(long)]
        miles_per_dollar_foreign: Option<f64>,
        /// Spending block size in dollars
        #[arg(long)]
        block_size: f64,
        /// Day of month the statement renews (1-31)
        #[arg(long)]
        renewal_date: i32,
        /// Maximum reward limit per cycle (omit if none)
        #[arg(long)]
        max_reward_limit: Option<f64>,
        /// Minimum spend required to earn rewards (omit if none)
        #[arg(long)]
        min_spend: Option<f64>,
    },

    /// List all saved credit cards
    ListCards,

    /// Remove a credit card by ID
    RemoveCard {
        /// Card ID to remove
        #[arg(long)]
        id: i64,
    },

    /// Find the best card for a spending category
    BestCard {
        /// Spending category to look up (e.g. "dining")
        #[arg(long)]
        category: String,
    },

    /// Record a spending transaction
    AddSpending {
        /// Card ID used for this purchase
        #[arg(long)]
        card_id: i64,
        /// Amount spent in dollars
        #[arg(long)]
        amount: f64,
        /// Spending category (e.g. "dining")
        #[arg(long)]
        category: String,
        /// Date of purchase (YYYY-MM-DD)
        #[arg(long)]
        date: String,
    },

    /// List spending transactions (optionally filter by card)
    ListSpending {
        /// Card ID to filter by (omit to show all)
        #[arg(long)]
        card_id: Option<i64>,
    },
}

fn main() {
    let cli = Cli::parse();

    let conn = db::init_db().expect("Failed to initialize database");

    match cli.command {
        Commands::AddCard {
            name,
            category,
            payment_category,
            miles_per_dollar,
            miles_per_dollar_foreign,
            block_size,
            renewal_date,
            max_reward_limit,
            min_spend,
        } => {
            let categories = if category.is_empty() {
                DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect()
            } else {
                category
            };
            let payment_categories = if payment_category.is_empty() {
                DEFAULT_PAYMENT_CATEGORIES.iter().map(|s| s.to_string()).collect()
            } else {
                payment_category
            };

            let id = db::add_card(
                &conn, &name, &categories, &payment_categories, miles_per_dollar,
                miles_per_dollar_foreign, block_size, renewal_date, max_reward_limit, min_spend,
            )
            .expect("Failed to add card");
            println!("Added card '{}' with ID {}", name, id);
        }

        Commands::ListCards => {
            let cards = db::list_cards(&conn).expect("Failed to list cards");
            if cards.is_empty() {
                println!("No cards found. Add one with: cc-tracker add-card --help");
            } else {
                println!("{}", Table::new(&cards));
            }
        }

        Commands::RemoveCard { id } => {
            let removed = db::remove_card(&conn, id).expect("Failed to remove card");
            if removed {
                println!("Removed card with ID {}", id);
            } else {
                println!("No card found with ID {}", id);
            }
        }

        Commands::BestCard { category } => {
            let results = db::best_card_for_category(&conn, &category)
                .expect("Failed to query best card");
            if results.is_empty() {
                println!("No cards have rewards for category '{}'", category);
            } else {
                println!("Best cards for '{}':", category);
                println!("{}", Table::new(&results));
            }
        }

        Commands::AddSpending {
            card_id,
            amount,
            category,
            date,
        } => {
            let (id, miles) = db::add_spending(&conn, card_id, amount, &category, &date)
                .expect("Failed to add spending");
            println!(
                "Recorded ${:.2} on card {} for '{}' — earned {:.0} miles (ID {})",
                amount, card_id, category, miles, id
            );
        }

        Commands::ListSpending { card_id } => {
            let spending = db::list_spending(&conn, card_id).expect("Failed to list spending");
            if spending.is_empty() {
                println!("No spending records found.");
            } else {
                println!("{}", Table::new(&spending));
            }
        }
    }
}
