# Quick Start

## Prerequisites

- Rust 1.93+
- Node.js 18+
- (Optional) ngrok for Telegram testing

## 1. Run the Backend

```bash
cargo run --bin backend
```

Server starts on `http://127.0.0.1:3000`. Verify with:

```bash
curl http://127.0.0.1:3000/api/health
```

## 2. Run the Frontend

```bash
cd src/frontend
npm install
npm run dev
```

Opens on `http://localhost:5173`.

## 3. Try It Out

Add a card:

```bash
curl -X POST http://127.0.0.1:3000/api/cards \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Chase Sapphire",
    "categories": ["dining", "travel"],
    "miles_per_dollar": 2.0,
    "block_size": 1.0,
    "renewal_date": 1
  }'
```

Get a recommendation:

```bash
curl "http://127.0.0.1:3000/api/best-card?category=dining&amount=50&payment_category=contactless"
```

## 4. Connect to Telegram (Optional)

### Create a bot

1. Message [@BotFather](https://t.me/botfather) on Telegram
2. Send `/newbot` and follow the prompts
3. Save the bot token

### Expose your frontend with ngrok

```bash
ngrok http 5173
```

Copy the HTTPS URL.

### Set the Mini App URL

1. Send `/mybots` to BotFather
2. Select your bot
3. Bot Settings > Menu Button > Configure Menu Button
4. Paste the ngrok HTTPS URL

### Test

Open your bot in Telegram, click the menu button, and your app loads inside Telegram.

## Development

```bash
# Run tests
cargo test

# Auto-reload backend (requires cargo-watch)
cargo watch -x 'run --bin backend'

# Build for release
cargo build --release --bin backend
```

## Deployment

### Backend

Build the release binary and deploy to any VPS or container host:

```bash
cargo build --release --bin backend
```

### Frontend

```bash
cd src/frontend
npm run build
```

Deploy the `dist/` folder to Vercel, Netlify, or Cloudflare Pages. Update `VITE_API_URL` in `.env` to point to your production backend.

### Update Telegram

In BotFather, update the Menu Button URL to your production frontend URL.

## Troubleshooting

- **"Failed to fetch" in frontend** — make sure the backend is running on port 3000
- **CORS errors** — the backend enables CORS for all origins in dev; restrict in production
- **Telegram app not loading** — ensure ngrok is running and you used the HTTPS URL
- **Database issues** — delete `cc_tracker.db` and restart the backend to reset
