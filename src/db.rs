use rusqlite::{Connection, Result, params};

use crate::models::{Card, CardRecommendation, Spending};

/// Creates tables on the given connection.
pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cards (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT,
            name                    TEXT NOT NULL,
            categories              TEXT NOT NULL,
            payment_categories      TEXT NOT NULL,
            miles_per_dollar        REAL NOT NULL,
            miles_per_dollar_foreign REAL,
            block_size              REAL NOT NULL,
            statement_renewal_date  INTEGER NOT NULL,
            max_reward_limit        REAL,
            min_spend               REAL
        );
        CREATE TABLE IF NOT EXISTS spending (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id      INTEGER NOT NULL REFERENCES cards(id),
            amount       REAL NOT NULL,
            category     TEXT NOT NULL,
            date         TEXT NOT NULL,
            miles_earned REAL NOT NULL
        );",
    )?;
    Ok(())
}

/// Opens (or creates) the SQLite database file and ensures tables exist.
pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("cc_tracker.db")?;
    init_tables(&conn)?;
    Ok(conn)
}

// ── Card operations ──────────────────────────────────────────────

pub fn add_card(
    conn: &Connection,
    name: &str,
    categories: &[String],
    payment_categories: &[String],
    miles_per_dollar: f64,
    miles_per_dollar_foreign: Option<f64>,
    block_size: f64,
    statement_renewal_date: i32,
    max_reward_limit: Option<f64>,
    min_spend: Option<f64>,
) -> Result<i64> {
    let categories_json = serde_json::to_string(categories).unwrap();
    let payment_categories_json = serde_json::to_string(payment_categories).unwrap();
    conn.execute(
        "INSERT INTO cards (name, categories, payment_categories, miles_per_dollar, miles_per_dollar_foreign, block_size, statement_renewal_date, max_reward_limit, min_spend)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![name, categories_json, payment_categories_json, miles_per_dollar, miles_per_dollar_foreign, block_size, statement_renewal_date, max_reward_limit, min_spend],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, categories, payment_categories, miles_per_dollar,
                miles_per_dollar_foreign, block_size,
                statement_renewal_date, max_reward_limit, min_spend
         FROM cards",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Card {
            id: row.get(0)?,
            name: row.get(1)?,
            categories: row.get(2)?,
            payment_categories: row.get(3)?,
            miles_per_dollar: row.get(4)?,
            miles_per_dollar_foreign: row.get(5)?,
            block_size: row.get(6)?,
            statement_renewal_date: row.get(7)?,
            max_reward_limit: row.get(8)?,
            min_spend: row.get(9)?,
        })
    })?;

    let mut cards = Vec::new();
    for card in rows {
        cards.push(card?);
    }
    Ok(cards)
}

pub fn remove_card(conn: &Connection, id: i64) -> Result<bool> {
    conn.execute("DELETE FROM spending WHERE card_id = ?1", params![id])?;
    let changed = conn.execute("DELETE FROM cards WHERE id = ?1", params![id])?;
    Ok(changed > 0)
}

pub fn best_card_for_category(
    conn: &Connection,
    category: &str,
) -> Result<Vec<CardRecommendation>> {
    let mut stmt = conn.prepare(
        "SELECT c.name, c.miles_per_dollar, c.block_size,
                (c.miles_per_dollar / c.block_size) AS effective_rate
         FROM cards c, json_each(c.categories) j
         WHERE LOWER(j.value) = LOWER(?1)
         ORDER BY effective_rate DESC",
    )?;

    let rows = stmt.query_map(params![category], |row| {
        Ok(CardRecommendation {
            card_name: row.get(0)?,
            miles_per_dollar: row.get(1)?,
            block_size: row.get(2)?,
            effective_rate: row.get(3)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// ── Spending operations ──────────────────────────────────────────

/// Calculates miles earned: floor(amount / block_size) * miles_per_dollar
fn calculate_miles(amount: f64, block_size: f64, miles_per_dollar: f64) -> f64 {
    (amount / block_size).floor() * miles_per_dollar
}

pub fn add_spending(
    conn: &Connection,
    card_id: i64,
    amount: f64,
    category: &str,
    date: &str,
) -> Result<(i64, f64)> {
    // Look up the card to calculate miles
    let (miles_per_dollar, block_size): (f64, f64) = conn.query_row(
        "SELECT miles_per_dollar, block_size FROM cards WHERE id = ?1",
        params![card_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let miles_earned = calculate_miles(amount, block_size, miles_per_dollar);

    conn.execute(
        "INSERT INTO spending (card_id, amount, category, date, miles_earned)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![card_id, amount, category, date, miles_earned],
    )?;

    Ok((conn.last_insert_rowid(), miles_earned))
}

pub fn list_spending(conn: &Connection, card_id: Option<i64>) -> Result<Vec<Spending>> {
    let mut results = Vec::new();

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<Spending> {
        Ok(Spending {
            id: row.get(0)?,
            card_id: row.get(1)?,
            amount: row.get(2)?,
            category: row.get(3)?,
            date: row.get(4)?,
            miles_earned: row.get(5)?,
        })
    };

    if let Some(id) = card_id {
        let mut stmt = conn.prepare(
            "SELECT id, card_id, amount, category, date, miles_earned
             FROM spending WHERE card_id = ?1 ORDER BY date DESC",
        )?;
        let rows = stmt.query_map(params![id], map_row)?;
        for row in rows {
            results.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, card_id, amount, category, date, miles_earned
             FROM spending ORDER BY date DESC",
        )?;
        let rows = stmt.query_map([], map_row)?;
        for row in rows {
            results.push(row?);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DEFAULT_CATEGORIES, DEFAULT_PAYMENT_CATEGORIES};

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        conn
    }

    fn all_categories() -> Vec<String> {
        DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect()
    }

    fn all_payment_categories() -> Vec<String> {
        DEFAULT_PAYMENT_CATEGORIES.iter().map(|s| s.to_string()).collect()
    }

    /// Shorthand for tests: add a card with default payment categories
    fn add_test_card(
        conn: &Connection,
        name: &str,
        categories: &[String],
        miles_per_dollar: f64,
        block_size: f64,
        renewal: i32,
        max_limit: Option<f64>,
        min_spend: Option<f64>,
    ) -> i64 {
        add_card(conn, name, categories, &all_payment_categories(), miles_per_dollar, None, block_size, renewal, max_limit, min_spend).unwrap()
    }

    // ── Card tests ───────────────────────────────────────────────

    #[test]
    fn test_add_card() {
        let conn = test_db();

        let cats = vec!["dining".to_string(), "travel".to_string()];
        let pay_cats = vec!["contactless".to_string(), "online".to_string()];
        let id = add_card(&conn, "DBS Altitude", &cats, &pay_cats, 3.0, Some(2.0), 1.0, 15, Some(5000.0), Some(800.0)).unwrap();
        assert_eq!(id, 1);

        let cards = list_cards(&conn).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].name, "DBS Altitude");
        assert_eq!(cards[0].categories, r#"["dining","travel"]"#);
        assert_eq!(cards[0].payment_categories, r#"["contactless","online"]"#);
        assert_eq!(cards[0].miles_per_dollar, 3.0);
        assert_eq!(cards[0].block_size, 1.0);
        assert_eq!(cards[0].statement_renewal_date, 15);
        assert_eq!(cards[0].miles_per_dollar_foreign, Some(2.0));
        assert_eq!(cards[0].max_reward_limit, Some(5000.0));
        assert_eq!(cards[0].min_spend, Some(800.0));
    }

    #[test]
    fn test_add_card_default_categories() {
        let conn = test_db();

        let cats = all_categories();
        add_card(&conn, "Generic Card", &cats, &all_payment_categories(), 1.0, None, 1.0, 1, None, None).unwrap();

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 1);
        let results = best_card_for_category(&conn, "entertainment").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_cards_empty() {
        let conn = test_db();
        let cards = list_cards(&conn).unwrap();
        assert!(cards.is_empty());
    }

    #[test]
    fn test_list_cards_multiple() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        add_test_card(&conn, "Card B", &vec!["travel".into()], 2.0, 1.0, 15, Some(1000.0), Some(500.0));
        add_test_card(&conn, "Card C", &vec!["groceries".into()], 10.0, 5.0, 20, None, None);

        let cards = list_cards(&conn).unwrap();
        assert_eq!(cards.len(), 3);
    }

    #[test]
    fn test_remove_card() {
        let conn = test_db();

        let id = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        assert!(remove_card(&conn, id).unwrap());

        let cards = list_cards(&conn).unwrap();
        assert!(cards.is_empty());
    }

    #[test]
    fn test_remove_card_deletes_spending() {
        let conn = test_db();

        let id = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        add_spending(&conn, id, 50.0, "dining", "2026-02-19").unwrap();

        remove_card(&conn, id).unwrap();

        let spending = list_spending(&conn, None).unwrap();
        assert!(spending.is_empty());
    }

    #[test]
    fn test_remove_card_nonexistent() {
        let conn = test_db();
        assert!(!remove_card(&conn, 999).unwrap());
    }

    // ── Best card tests ──────────────────────────────────────────

    #[test]
    fn test_best_card_single_match() {
        let conn = test_db();

        add_test_card(&conn, "DBS Altitude", &vec!["dining".into(), "travel".into()], 3.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].card_name, "DBS Altitude");
        assert_eq!(results[0].effective_rate, 3.0);
    }

    #[test]
    fn test_best_card_ranked_by_effective_rate() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        add_test_card(&conn, "Card B", &vec!["dining".into()], 10.0, 5.0, 1, None, None);
        add_test_card(&conn, "Card C", &vec!["dining".into()], 4.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].card_name, "Card C");
        assert_eq!(results[1].card_name, "Card A");
        assert_eq!(results[2].card_name, "Card B");
    }

    #[test]
    fn test_best_card_case_insensitive() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["Dining".into()], 3.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_best_card_no_match() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "travel").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_best_card_multi_category_card() {
        let conn = test_db();

        add_test_card(&conn, "Multi Card", &vec!["dining".into(), "travel".into()], 2.0, 1.0, 1, None, None);
        add_test_card(&conn, "Dining Card", &vec!["dining".into()], 4.0, 1.0, 1, None, None);

        let dining = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(dining.len(), 2);
        assert_eq!(dining[0].card_name, "Dining Card");

        let travel = best_card_for_category(&conn, "travel").unwrap();
        assert_eq!(travel.len(), 1);
        assert_eq!(travel[0].card_name, "Multi Card");
    }

    // ── Spending tests ───────────────────────────────────────────

    #[test]
    fn test_add_spending_block_size_1() {
        let conn = test_db();

        // 3 miles per $1 block
        let card_id = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);

        let (id, miles) = add_spending(&conn, card_id, 42.50, "dining", "2026-02-19").unwrap();
        assert_eq!(id, 1);
        // floor(42.50 / 1.0) * 3.0 = 42 * 3 = 126
        assert_eq!(miles, 126.0);
    }

    #[test]
    fn test_add_spending_block_size_5() {
        let conn = test_db();

        // 10 miles per $5 block
        let card_id = add_test_card(&conn, "Card B", &vec!["dining".into()], 10.0, 5.0, 1, None, None);

        let (_, miles) = add_spending(&conn, card_id, 42.50, "dining", "2026-02-19").unwrap();
        // floor(42.50 / 5.0) * 10.0 = 8 * 10 = 80
        assert_eq!(miles, 80.0);
    }

    #[test]
    fn test_add_spending_below_block_size() {
        let conn = test_db();

        // 10 miles per $5 block, spend only $3
        let card_id = add_test_card(&conn, "Card B", &vec!["dining".into()], 10.0, 5.0, 1, None, None);

        let (_, miles) = add_spending(&conn, card_id, 3.0, "dining", "2026-02-19").unwrap();
        // floor(3.0 / 5.0) * 10.0 = 0 * 10 = 0
        assert_eq!(miles, 0.0);
    }

    #[test]
    fn test_list_spending_all() {
        let conn = test_db();

        let card_a = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        let card_b = add_test_card(&conn, "Card B", &vec!["travel".into()], 2.0, 1.0, 1, None, None);

        add_spending(&conn, card_a, 50.0, "dining", "2026-02-18").unwrap();
        add_spending(&conn, card_b, 100.0, "travel", "2026-02-19").unwrap();

        let all = list_spending(&conn, None).unwrap();
        assert_eq!(all.len(), 2);
        // Ordered by date DESC
        assert_eq!(all[0].date, "2026-02-19");
        assert_eq!(all[1].date, "2026-02-18");
    }

    #[test]
    fn test_list_spending_by_card() {
        let conn = test_db();

        let card_a = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        let card_b = add_test_card(&conn, "Card B", &vec!["travel".into()], 2.0, 1.0, 1, None, None);

        add_spending(&conn, card_a, 50.0, "dining", "2026-02-18").unwrap();
        add_spending(&conn, card_b, 100.0, "travel", "2026-02-19").unwrap();

        let card_a_spending = list_spending(&conn, Some(card_a)).unwrap();
        assert_eq!(card_a_spending.len(), 1);
        assert_eq!(card_a_spending[0].amount, 50.0);
    }

    #[test]
    fn test_spending_miles_stored_correctly() {
        let conn = test_db();

        let card_id = add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        add_spending(&conn, card_id, 100.0, "dining", "2026-02-19").unwrap();

        let spending = list_spending(&conn, Some(card_id)).unwrap();
        assert_eq!(spending[0].miles_earned, 300.0);
    }
}
