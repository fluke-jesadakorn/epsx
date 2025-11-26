# Backend Helper Scripts

Helper scripts to run common backend tasks with proper environment variable loading.

## Prerequisites

1. Create `.env` file in `/apps/backend/` directory
2. Ensure all required environment variables are set

## Scripts

### check-env.sh

Check environment configuration and test connections.

```bash
./scripts/check-env.sh
```

**What it does:**
- Verifies all required environment variables are set
- Tests database connection
- Tests Redis connection (if available)
- Shows configuration summary

**Use this:**
- Before running migrations or starting the backend
- To troubleshoot environment issues
- After setting up a new development environment

---

### migrate.sh

Run database migrations with environment loaded from .env.

```bash
./scripts/migrate.sh
```

**What it does:**
- Loads environment from `.env` file
- Validates DATABASE_URL is set
- Runs `sqlx migrate run`

**Use this instead of:**
```bash
# Don't do this - may not load .env properly
sqlx migrate run

# Use this instead
./scripts/migrate.sh
```

---

### run.sh

Start the backend server with environment loaded from .env.

```bash
./scripts/run.sh
```

**What it does:**
- Loads environment from `.env` file
- Validates required variables (DATABASE_URL, REDIS_URL)
- Starts backend with `cargo run --release`

**Use this instead of:**
```bash
# Don't do this - may not load .env properly
cargo run

# Use this instead
./scripts/run.sh
```

## Environment Variables

### Required

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string (for notifications)
- `OIDC_CLIENT_SECRET` - OIDC client secret
- `OIDC_ADMIN_CLIENT_SECRET` - OIDC admin client secret
- `FIREBASE_PROJECT_ID` - Firebase project ID

### Optional

- `BACKEND_URL` - Backend URL (default: http://localhost:8080)
- `FRONTEND_URL` - Frontend URL (default: http://localhost:3000)
- `RUST_LOG` - Logging level (default: info)

## Troubleshooting

### .env file not found

Create `.env` file in `/apps/backend/`:

```bash
cd apps/backend
cp ../../.env .env
# Or copy only required variables
```

### DATABASE_URL not working

Check format:
```
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
```

### Redis connection failed

Check Redis is running:
```bash
redis-cli ping
```

Or check REDIS_URL format:
```
REDIS_URL=redis://localhost:6379
REDIS_URL=rediss://user:password@host:port  # For TLS
```

### Migrations fail with "relation already exists"

This means the table/index already exists. The migration system will handle it automatically with `IF NOT EXISTS` clauses.

## Development Workflow

```bash
# 1. Check environment
./scripts/check-env.sh

# 2. Run migrations
./scripts/migrate.sh

# 3. Start backend
./scripts/run.sh

# OR for development with hot reload
cargo watch -x run
```

## Production Note

These scripts are for local development only. In production:
- Use proper secret management (not .env files)
- Use environment variables from deployment platform
- Use managed database and Redis services
