mod db;
mod models;

use clap::{Parser, Subcommand};
use models::DEFAULT_CATEGORIES;
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
        /// Miles earned per spending block
        #[arg(long)]
        miles_per_dollar: f64,
        /// Spending block size in dollars (default: 1)
        #[arg(long, default_value_t = 1.0)]
        block_size: f64,
        /// Day of month the statement renews (1-31)
        #[arg(long, default_value_t = 1)]
        renewal_date: i32,
        /// Maximum reward limit per cycle (0 = unlimited)
        #[arg(long, default_value_t = 0.0)]
        max_reward_limit: f64,
        /// Minimum spend required to earn rewards
        #[arg(long, default_value_t = 0.0)]
        min_spend: f64,
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
}

fn main() {
    let cli = Cli::parse();

    let conn = db::init_db().expect("Failed to initialize database");

    match cli.command {
        Commands::AddCard {
            name,
            category,
            miles_per_dollar,
            block_size,
            renewal_date,
            max_reward_limit,
            min_spend,
        } => {
            // If no categories provided, default to all
            let categories = if category.is_empty() {
                DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect()
            } else {
                category
            };

            let id = db::add_card(
                &conn, &name, &categories, miles_per_dollar, block_size,
                renewal_date, max_reward_limit, min_spend,
            )
            .expect("Failed to add card");
            println!("Added card '{}' with ID {} (categories: {:?})", name, id, categories);
        }

        Commands::ListCards => {
            let cards = db::list_cards(&conn).expect("Failed to list cards");
            if cards.is_empty() {
                println!("No cards found. Add one with: cc-tracker add-card --name \"...\" --miles-per-dollar 3");
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
    }
}
