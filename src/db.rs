use rusqlite::{Connection, Result, params};

use crate::models::{Card, CardRecommendation};

/// Creates tables on the given connection.
pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cards (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT,
            name                    TEXT NOT NULL,
            categories              TEXT NOT NULL,
            miles_per_dollar        REAL NOT NULL,
            block_size              REAL NOT NULL DEFAULT 1.0,
            statement_renewal_date  INTEGER NOT NULL DEFAULT 1,
            max_reward_limit        REAL NOT NULL DEFAULT 0.0,
            min_spend               REAL NOT NULL DEFAULT 0.0
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

pub fn add_card(
    conn: &Connection,
    name: &str,
    categories: &[String],
    miles_per_dollar: f64,
    block_size: f64,
    statement_renewal_date: i32,
    max_reward_limit: f64,
    min_spend: f64,
) -> Result<i64> {
    let categories_json = serde_json::to_string(categories).unwrap();
    conn.execute(
        "INSERT INTO cards (name, categories, miles_per_dollar, block_size, statement_renewal_date, max_reward_limit, min_spend)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![name, categories_json, miles_per_dollar, block_size, statement_renewal_date, max_reward_limit, min_spend],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, categories, miles_per_dollar, block_size,
                statement_renewal_date, max_reward_limit, min_spend
         FROM cards",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Card {
            id: row.get(0)?,
            name: row.get(1)?,
            categories: row.get(2)?,
            miles_per_dollar: row.get(3)?,
            block_size: row.get(4)?,
            statement_renewal_date: row.get(5)?,
            max_reward_limit: row.get(6)?,
            min_spend: row.get(7)?,
        })
    })?;

    let mut cards = Vec::new();
    for card in rows {
        cards.push(card?);
    }
    Ok(cards)
}

pub fn remove_card(conn: &Connection, id: i64) -> Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DEFAULT_CATEGORIES;

    /// Helper: creates an in-memory DB with tables ready to go.
    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        conn
    }

    fn all_categories() -> Vec<String> {
        DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_add_card() {
        let conn = test_db();

        let cats = vec!["dining".to_string(), "travel".to_string()];
        let id = add_card(&conn, "DBS Altitude", &cats, 3.0, 1.0, 15, 5000.0, 800.0).unwrap();
        assert_eq!(id, 1);

        let cards = list_cards(&conn).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].name, "DBS Altitude");
        assert_eq!(cards[0].categories, r#"["dining","travel"]"#);
        assert_eq!(cards[0].miles_per_dollar, 3.0);
        assert_eq!(cards[0].block_size, 1.0);
        assert_eq!(cards[0].statement_renewal_date, 15);
        assert_eq!(cards[0].max_reward_limit, 5000.0);
        assert_eq!(cards[0].min_spend, 800.0);
    }

    #[test]
    fn test_add_card_default_categories() {
        let conn = test_db();

        let cats = all_categories();
        add_card(&conn, "Generic Card", &cats, 1.0, 1.0, 1, 0.0, 0.0).unwrap();

        // Should match any category
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

        add_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();
        add_card(&conn, "Card B", &vec!["travel".into()], 2.0, 1.0, 15, 1000.0, 500.0).unwrap();
        add_card(&conn, "Card C", &vec!["groceries".into()], 10.0, 5.0, 20, 0.0, 0.0).unwrap();

        let cards = list_cards(&conn).unwrap();
        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].name, "Card A");
        assert_eq!(cards[1].name, "Card B");
        assert_eq!(cards[2].name, "Card C");
    }

    #[test]
    fn test_remove_card() {
        let conn = test_db();

        let id = add_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();
        assert!(remove_card(&conn, id).unwrap());

        let cards = list_cards(&conn).unwrap();
        assert!(cards.is_empty());
    }

    #[test]
    fn test_remove_card_nonexistent() {
        let conn = test_db();
        assert!(!remove_card(&conn, 999).unwrap());
    }

    #[test]
    fn test_best_card_single_match() {
        let conn = test_db();

        add_card(&conn, "DBS Altitude", &vec!["dining".into(), "travel".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].card_name, "DBS Altitude");
        assert_eq!(results[0].effective_rate, 3.0);
    }

    #[test]
    fn test_best_card_ranked_by_effective_rate() {
        let conn = test_db();

        // Card A: 3 miles per $1 → effective 3.0
        add_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();
        // Card B: 10 miles per $5 → effective 2.0
        add_card(&conn, "Card B", &vec!["dining".into()], 10.0, 5.0, 1, 0.0, 0.0).unwrap();
        // Card C: 4 miles per $1 → effective 4.0
        add_card(&conn, "Card C", &vec!["dining".into()], 4.0, 1.0, 1, 0.0, 0.0).unwrap();

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].card_name, "Card C");
        assert_eq!(results[1].card_name, "Card A");
        assert_eq!(results[2].card_name, "Card B");
    }

    #[test]
    fn test_best_card_case_insensitive() {
        let conn = test_db();

        add_card(&conn, "Card A", &vec!["Dining".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();

        let results = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_best_card_no_match() {
        let conn = test_db();

        add_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, 0.0, 0.0).unwrap();

        let results = best_card_for_category(&conn, "travel").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_best_card_multi_category_card() {
        let conn = test_db();

        // One card covers both dining and travel
        add_card(&conn, "Multi Card", &vec!["dining".into(), "travel".into()], 2.0, 1.0, 1, 0.0, 0.0).unwrap();
        // Dining-only card with better rate
        add_card(&conn, "Dining Card", &vec!["dining".into()], 4.0, 1.0, 1, 0.0, 0.0).unwrap();

        let dining = best_card_for_category(&conn, "dining").unwrap();
        assert_eq!(dining.len(), 2);
        assert_eq!(dining[0].card_name, "Dining Card");
        assert_eq!(dining[1].card_name, "Multi Card");

        let travel = best_card_for_category(&conn, "travel").unwrap();
        assert_eq!(travel.len(), 1);
        assert_eq!(travel[0].card_name, "Multi Card");
    }
}
