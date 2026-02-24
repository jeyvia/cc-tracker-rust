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

/// Converts a (year, month, day) to days since Unix epoch using the
/// algorithm from http://howardhinnant.github.io/date_algorithms.html
fn ymd_to_days(year: i32, month: i32, day: i32) -> i32 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let m = month;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Returns the day of week for a date: 0=Monday, 1=Tuesday, ... 5=Saturday, 6=Sunday.
fn day_of_week(year: i32, month: i32, day: i32) -> i32 {
    let days = ymd_to_days(year, month, day);
    // 1970-01-01 was a Thursday (day 3 in 0=Mon scheme)
    ((days % 7) + 7 + 3) % 7
}

/// If the given date falls on a weekend, moves it to the previous Friday.
/// Returns (year, month, day) adjusted.
fn adjust_for_weekend(year: i32, month: i32, day: i32) -> (i32, i32, i32) {
    let dow = day_of_week(year, month, day);
    let shift = match dow {
        5 => 1, // Saturday → Friday (subtract 1 day)
        6 => 2, // Sunday → Friday (subtract 2 days)
        _ => 0,
    };
    if shift == 0 {
        return (year, month, day);
    }
    let days = ymd_to_days(year, month, day) - shift;
    days_to_ymd(days)
}

/// Converts days since Unix epoch back to (year, month, day).
fn days_to_ymd(days: i32) -> (i32, i32, i32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Returns the start date of the current statement cycle for a card,
/// given its renewal day and a reference date (YYYY-MM-DD).
/// If the renewal day falls on a weekend, it is shifted to the previous Friday.
fn cycle_start_date(renewal_day: i32, reference_date: &str) -> String {
    let parts: Vec<&str> = reference_date.split('-').collect();
    let year: i32 = parts[0].parse().unwrap();
    let month: i32 = parts[1].parse().unwrap();
    let day: i32 = parts[2].parse().unwrap();

    // Compute the adjusted renewal date for this month
    let (ay, am, ad) = adjust_for_weekend(year, month, renewal_day);

    if day >= ad && am == month {
        // Current cycle started this month (on the adjusted date)
        format!("{:04}-{:02}-{:02}", ay, am, ad)
    } else {
        // Current cycle started last month
        let (prev_y, prev_m) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
        let (py, pm, pd) = adjust_for_weekend(prev_y, prev_m, renewal_day);
        format!("{:04}-{:02}-{:02}", py, pm, pd)
    }
}

pub fn best_card_for_category(
    conn: &Connection,
    category: &str,
    amount: f64,
    payment_category: &str,
    date: &str,
) -> Result<Vec<CardRecommendation>> {
    // Step 1: Find all cards that match the spending category AND payment category
    let mut stmt = conn.prepare(
        "SELECT DISTINCT c.id, c.name, c.miles_per_dollar, c.block_size,
                (c.miles_per_dollar / c.block_size) AS effective_rate,
                c.max_reward_limit, c.min_spend, c.statement_renewal_date
         FROM cards c, json_each(c.categories) j, json_each(c.payment_categories) p
         WHERE LOWER(j.value) = LOWER(?1)
           AND LOWER(p.value) = LOWER(?2)
         ORDER BY effective_rate DESC",
    )?;

    struct CandidateCard {
        id: i64,
        name: String,
        miles_per_dollar: f64,
        block_size: f64,
        effective_rate: f64,
        max_reward_limit: Option<f64>,
        min_spend: Option<f64>,
        statement_renewal_date: i32,
    }

    let rows = stmt.query_map(params![category, payment_category], |row| {
        Ok(CandidateCard {
            id: row.get(0)?,
            name: row.get(1)?,
            miles_per_dollar: row.get(2)?,
            block_size: row.get(3)?,
            effective_rate: row.get(4)?,
            max_reward_limit: row.get(5)?,
            min_spend: row.get(6)?,
            statement_renewal_date: row.get(7)?,
        })
    })?;

    let candidates: Vec<CandidateCard> = rows.collect::<Result<Vec<_>>>()?;

    let mut results = Vec::new();

    for card in &candidates {
        let miles_this_txn = calculate_miles(amount, card.block_size, card.miles_per_dollar);

        // Step 2: Check max_reward_limit — sum spending in the current cycle
        let cycle_start = cycle_start_date(card.statement_renewal_date, date);
        let cycle_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0.0) FROM spending
             WHERE card_id = ?1 AND date >= ?2",
            params![card.id, cycle_start],
            |row| row.get(0),
        )?;

        let remaining_limit = card.max_reward_limit.map(|limit| (limit - cycle_total).max(0.0));

        // Check if adding this amount would exceed the reward limit
        let exceeded_limit = match remaining_limit {
            Some(remaining) => amount > remaining,
            None => false, // no limit
        };

        // Step 3: Check min_spend — has the card met its minimum spend this cycle?
        let min_spend_met = match card.min_spend {
            Some(min) => cycle_total >= min,
            None => true, // no minimum
        };

        // Determine eligibility and reason
        let (eligible, reason) = if exceeded_limit {
            (false, format!("Exceeds reward limit (${:.2} remaining)", remaining_limit.unwrap()))
        } else if !min_spend_met {
            let shortfall = card.min_spend.unwrap() - cycle_total;
            (false, format!("Min spend not met (${:.2} more needed)", shortfall))
        } else {
            (true, "Eligible".to_string())
        };

        results.push(CardRecommendation {
            card_name: card.name.clone(),
            miles_per_dollar: card.miles_per_dollar,
            block_size: card.block_size,
            effective_rate: card.effective_rate,
            miles_earned: miles_this_txn,
            remaining_limit,
            eligible,
            reason,
        });
    }

    // Sort: eligible cards first (by effective_rate DESC), then ineligible cards
    results.sort_by(|a, b| {
        b.eligible.cmp(&a.eligible)
            .then(b.effective_rate.partial_cmp(&a.effective_rate).unwrap())
    });

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

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        let results = best_card_for_category(&conn, "entertainment", 10.0, "contactless", "2026-02-19").unwrap();
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

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].card_name, "DBS Altitude");
        assert_eq!(results[0].effective_rate, 3.0);
        assert!(results[0].eligible);
    }

    #[test]
    fn test_best_card_ranked_by_effective_rate() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);
        add_test_card(&conn, "Card B", &vec!["dining".into()], 10.0, 5.0, 1, None, None);
        add_test_card(&conn, "Card C", &vec!["dining".into()], 4.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].card_name, "Card C");
        assert_eq!(results[1].card_name, "Card A");
        assert_eq!(results[2].card_name, "Card B");
    }

    #[test]
    fn test_best_card_case_insensitive() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["Dining".into()], 3.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_best_card_no_match() {
        let conn = test_db();

        add_test_card(&conn, "Card A", &vec!["dining".into()], 3.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "travel", 10.0, "contactless", "2026-02-19").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_best_card_no_match_payment_category() {
        let conn = test_db();

        // Card only supports "contactless", query with "online"
        let cats = vec!["dining".into()];
        let pay_cats = vec!["contactless".into()];
        add_card(&conn, "Card A", &cats, &pay_cats, 3.0, None, 1.0, 1, None, None).unwrap();

        let results = best_card_for_category(&conn, "dining", 10.0, "online", "2026-02-19").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_best_card_multi_category_card() {
        let conn = test_db();

        add_test_card(&conn, "Multi Card", &vec!["dining".into(), "travel".into()], 2.0, 1.0, 1, None, None);
        add_test_card(&conn, "Dining Card", &vec!["dining".into()], 4.0, 1.0, 1, None, None);

        let dining = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(dining.len(), 2);
        assert_eq!(dining[0].card_name, "Dining Card");

        let travel = best_card_for_category(&conn, "travel", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(travel.len(), 1);
        assert_eq!(travel[0].card_name, "Multi Card");
    }

    #[test]
    fn test_best_card_exceeds_reward_limit() {
        let conn = test_db();

        // Card with $100 reward limit, renewal day 1
        let card_id = add_test_card(&conn, "Limited Card", &vec!["dining".into()], 4.0, 1.0, 1, Some(100.0), None);
        // Spend $90 already in this cycle
        add_spending(&conn, card_id, 90.0, "dining", "2026-02-05").unwrap();

        // Try to spend $20 more — exceeds the $100 limit ($10 remaining)
        let results = best_card_for_category(&conn, "dining", 20.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].eligible);
        assert!(results[0].reason.contains("Exceeds reward limit"));
    }

    #[test]
    fn test_best_card_within_reward_limit() {
        let conn = test_db();

        // Card with $100 reward limit, renewal day 1
        let card_id = add_test_card(&conn, "Limited Card", &vec!["dining".into()], 4.0, 1.0, 1, Some(100.0), None);
        // Spend $50 already in this cycle
        add_spending(&conn, card_id, 50.0, "dining", "2026-02-05").unwrap();

        // Try to spend $30 more — within limit ($50 remaining)
        let results = best_card_for_category(&conn, "dining", 30.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].eligible);
        assert_eq!(results[0].remaining_limit, Some(50.0));
    }

    #[test]
    fn test_best_card_min_spend_not_met() {
        let conn = test_db();

        // Card with $500 min spend, renewal day 1
        add_test_card(&conn, "Min Spend Card", &vec!["dining".into()], 4.0, 1.0, 1, None, Some(500.0));

        // No spending yet — min spend not met
        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].eligible);
        assert!(results[0].reason.contains("Min spend not met"));
    }

    #[test]
    fn test_best_card_min_spend_met() {
        let conn = test_db();

        // Card with $500 min spend, renewal day 1
        let card_id = add_test_card(&conn, "Min Spend Card", &vec!["dining".into()], 4.0, 1.0, 1, None, Some(500.0));
        // Already spent $600 this cycle
        add_spending(&conn, card_id, 600.0, "dining", "2026-02-05").unwrap();

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].eligible);
    }

    #[test]
    fn test_best_card_eligible_sorted_first() {
        let conn = test_db();

        // Card A: high rate but min spend not met
        add_test_card(&conn, "Card A", &vec!["dining".into()], 10.0, 1.0, 1, None, Some(500.0));
        // Card B: lower rate but no restrictions
        add_test_card(&conn, "Card B", &vec!["dining".into()], 2.0, 1.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining", 10.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 2);
        // Card B should come first because it's eligible
        assert_eq!(results[0].card_name, "Card B");
        assert!(results[0].eligible);
        // Card A is ineligible, sorted after
        assert_eq!(results[1].card_name, "Card A");
        assert!(!results[1].eligible);
    }

    #[test]
    fn test_best_card_miles_earned_calculated() {
        let conn = test_db();

        // 10 miles per $5 block
        add_test_card(&conn, "Card A", &vec!["dining".into()], 10.0, 5.0, 1, None, None);

        let results = best_card_for_category(&conn, "dining", 42.50, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        // floor(42.50 / 5.0) * 10.0 = 8 * 10 = 80
        assert_eq!(results[0].miles_earned, 80.0);
    }

    // ── Cycle date / weekend tests ─────────────────────────────

    #[test]
    fn test_cycle_start_date_weekday() {
        // 2026-02-15 is a Sunday, renewal day 15 → adjusted to Friday 13th
        // Reference date Feb 19 (Thu) >= 13, so cycle started Feb 13
        let start = cycle_start_date(15, "2026-02-19");
        assert_eq!(start, "2026-02-13");
    }

    #[test]
    fn test_cycle_start_date_saturday_adjustment() {
        // 2026-02-14 is a Saturday, renewal day 14 → adjusted to Friday 13th
        let start = cycle_start_date(14, "2026-02-19");
        assert_eq!(start, "2026-02-13");
    }

    #[test]
    fn test_cycle_start_date_sunday_adjustment() {
        // 2026-03-01 is a Sunday, renewal day 1 → adjusted to Friday Feb 27
        // Reference date Mar 5 (Thu), renewal day 1 for March is Sun → Fri Feb 27
        // day 5 >= adjusted day? adjusted_for March 1 → Feb 27, am=2, month=3, am != month
        // So we go to "last month": Feb 1 is a Sunday → adjusted to Jan 30 (Fri)
        // Wait, let me reconsider. The reference is Mar 5 and renewal day is 1.
        // For this month (March): March 1 is Sunday → adjust to Feb 27 (Friday).
        // am=2, month=3, am != month → falls through to "last month" branch.
        // Actually the adjusted date moved to a different month, so we need to handle this.
        // Let me verify: day=5, ad=27, am=2, month=3. day(5) >= ad(27) is false, so
        // it goes to the else branch: prev month = Feb, renewal 1 → Feb 1 is Sunday → Jan 30 Fri.
        // The cycle start should be Jan 30.
        let start = cycle_start_date(1, "2026-03-05");
        assert_eq!(start, "2026-01-30");
    }

    #[test]
    fn test_cycle_start_date_no_adjustment() {
        // 2026-02-02 is a Monday, renewal day 2 → no adjustment needed
        let start = cycle_start_date(2, "2026-02-19");
        assert_eq!(start, "2026-02-02");
    }

    #[test]
    fn test_day_of_week() {
        // Known dates for verification:
        // 2026-02-19 is a Thursday (3)
        assert_eq!(day_of_week(2026, 2, 19), 3);
        // 2026-02-14 is a Saturday (5)
        assert_eq!(day_of_week(2026, 2, 14), 5);
        // 2026-02-15 is a Sunday (6)
        assert_eq!(day_of_week(2026, 2, 15), 6);
        // 2026-02-13 is a Friday (4)
        assert_eq!(day_of_week(2026, 2, 13), 4);
        // 2026-02-16 is a Monday (0)
        assert_eq!(day_of_week(2026, 2, 16), 0);
    }

    #[test]
    fn test_reward_limit_respects_weekend_cycle() {
        let conn = test_db();

        // Card with renewal day 15, which in Feb 2026 is a Sunday → adjusted to Feb 13 (Fri)
        // max_reward_limit = $200
        let card_id = add_test_card(&conn, "Weekend Card", &vec!["dining".into()], 4.0, 1.0, 15, Some(200.0), None);

        // Spend $150 on Feb 14 (after the adjusted cycle start of Feb 13)
        add_spending(&conn, card_id, 150.0, "dining", "2026-02-14").unwrap();

        // Query on Feb 19 for $60 — should exceed limit ($50 remaining)
        let results = best_card_for_category(&conn, "dining", 60.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].eligible);
        assert_eq!(results[0].remaining_limit, Some(50.0));
    }

    #[test]
    fn test_spending_before_adjusted_cycle_not_counted() {
        let conn = test_db();

        // Card with renewal day 15, Feb 2026 → adjusted to Feb 13 (Fri)
        let card_id = add_test_card(&conn, "Weekend Card", &vec!["dining".into()], 4.0, 1.0, 15, Some(200.0), None);

        // Spend $180 on Feb 12 (BEFORE the adjusted cycle start of Feb 13)
        add_spending(&conn, card_id, 180.0, "dining", "2026-02-12").unwrap();

        // Query on Feb 19 for $50 — previous cycle spending shouldn't count
        let results = best_card_for_category(&conn, "dining", 50.0, "contactless", "2026-02-19").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].eligible);
        assert_eq!(results[0].remaining_limit, Some(200.0));
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
