# Credit Card Miles Tracker

A Rust + TypeScript application for tracking credit card miles and getting smart card recommendations for every purchase.

## Features

- **Smart card recommendations** — finds the best card for any purchase based on category, payment method, and reward rates
- **Spending tracking** — log transactions and miles earned per card
- **Card management** — add, list, and delete credit cards with full reward configuration
- **Reward-aware algorithm** that considers:
  - Spending category and payment method matching
  - Maximum reward limits per statement cycle
  - Minimum spend thresholds
  - Weekend-adjusted statement cycles
  - Miles calculation: `floor(amount / block_size) * miles_per_dollar`
- **Telegram Mini App** frontend with theme matching and haptic feedback

## Architecture

```
Telegram Mini App (React + TypeScript)
        │ HTTP/REST
Rust Backend (Axum) — port 3000
        │
  SQLite Database
```

## Tech Stack

| Layer    | Technology                                  |
|----------|---------------------------------------------|
| Backend  | Rust, Axum, Tokio, rusqlite, serde          |
| Frontend | TypeScript, React, Vite, @twa-dev/sdk       |
| Database | SQLite                                      |

## Project Structure

```
cc-tracker-rust/
├── src/
│   ├── backend/
│   │   ├── main.rs        # Axum REST API server
│   │   ├── db.rs          # Database operations + tests
│   │   └── models.rs      # Data structures
│   └── frontend/          # React Telegram Mini App
│       ├── src/
│       │   ├── App.tsx        # Main component
│       │   ├── api.ts         # Backend API client
│       │   └── telegram.ts    # Telegram SDK integration
│       └── package.json
├── Cargo.toml
├── QUICKSTART.md
└── README.md
```

## API Endpoints

| Method | Endpoint         | Description                        |
|--------|------------------|------------------------------------|
| GET    | `/api/health`    | Health check                       |
| POST   | `/api/cards`     | Add a new card                     |
| GET    | `/api/cards`     | List all cards                     |
| DELETE | `/api/cards?id=` | Delete a card                      |
| GET    | `/api/best-card` | Get card recommendations           |
| POST   | `/api/spending`  | Record a spending transaction      |
| GET    | `/api/spending`  | List spending (optional `card_id`) |

### Best Card Query Parameters

```
GET /api/best-card?category=dining&amount=50&payment_category=contactless&date=2026-02-24
```

- `category` — spending category (dining, travel, groceries, transport, shopping, entertainment)
- `amount` — purchase amount
- `payment_category` — contactless, mobile contactless, or online
- `date` — optional, defaults to today

### Add Card Request Body

```json
{
  "name": "Card Name",
  "categories": ["dining", "travel"],
  "payment_categories": ["contactless"],
  "miles_per_dollar": 2.0,
  "miles_per_dollar_foreign": 2.5,
  "block_size": 1.0,
  "renewal_date": 1,
  "max_reward_limit": 1000.0,
  "min_spend": 100.0
}
```

All fields except `name`, `miles_per_dollar`, `block_size`, and `renewal_date` are optional. Categories and payment categories default to all if omitted.

### Add Spending Request Body

```json
{
  "card_id": 1,
  "amount": 50.0,
  "category": "dining",
  "date": "2026-02-24"
}
```

## Database Schema

**cards** — credit card details, categories, reward rates, and limits

**spending** — transactions linked to cards with amount, category, date, and miles earned

## Testing

```bash
cargo test
```

32 tests covering card CRUD, spending tracking, recommendation algorithm, statement cycle calculations, and weekend adjustments.

## License

MIT