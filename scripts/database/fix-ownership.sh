#!/usr/bin/env bash
# =============================================================================
# Fix database ownership — make epsx_user the consistent owner of all dev DBs
# =============================================================================
# Run as superuser (e.g., your OS user that has PostgreSQL superuser access).
#
# Usage: ./scripts/database/fix-ownership.sh
# =============================================================================
set -euo pipefail

R='\033[0;31m' G='\033[0;32m' Y='\033[1;33m' B='\033[0;34m' NC='\033[0m'
ok()   { echo -e "${G}[OK]${NC}   $*"; }
info() { echo -e "${B}[INFO]${NC} $*"; }
warn() { echo -e "${Y}[WARN]${NC} $*"; }
fail() { echo -e "${R}[FAIL]${NC} $*"; exit 1; }

DATABASES=("epsx_dev" "epsx_analytics_dev" "epsx_payments_dev" "epsx_notifications_dev")

# ─── Preflight ────────────────────────────────────────────────────────────
pg_isready -q 2>/dev/null || fail "PostgreSQL is not running"

# ─── 1. Ensure epsx_user role exists with CREATEDB ────────────────────────
info "Ensuring epsx_user role exists with CREATEDB..."

psql postgres -v ON_ERROR_STOP=1 <<-'EOSQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'epsx_user') THEN
    CREATE ROLE epsx_user LOGIN CREATEDB PASSWORD 'password';
    RAISE NOTICE 'Created role epsx_user';
  ELSE
    ALTER ROLE epsx_user CREATEDB;
    RAISE NOTICE 'Role epsx_user already exists — ensured CREATEDB';
  END IF;
END $$;
EOSQL
ok "epsx_user role ready"

# ─── 2. Transfer database ownership ──────────────────────────────────────
info "Transferring database ownership to epsx_user..."

for db in "${DATABASES[@]}"; do
  if psql postgres -t -c "SELECT 1 FROM pg_database WHERE datname = '$db'" | grep -q 1; then
    psql postgres -c "ALTER DATABASE $db OWNER TO epsx_user;" 2>/dev/null
    ok "Database $db → epsx_user"
  else
    warn "Database $db does not exist — skipping"
  fi
done

# ─── 3. Reassign object ownership inside each database ───────────────────
info "Reassigning object ownership inside each database..."

for db in "${DATABASES[@]}"; do
  if psql postgres -t -c "SELECT 1 FROM pg_database WHERE datname = '$db'" | grep -q 1; then
    psql -d "$db" -v ON_ERROR_STOP=0 <<-EOSQL
      -- Transfer ownership of all tables, sequences, and functions to epsx_user
      DO \$\$
      DECLARE
        obj RECORD;
      BEGIN
        -- Tables
        FOR obj IN SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tableowner <> 'epsx_user'
        LOOP
          EXECUTE format('ALTER TABLE public.%I OWNER TO epsx_user', obj.tablename);
        END LOOP;

        -- Sequences
        FOR obj IN SELECT sequencename FROM pg_sequences WHERE schemaname = 'public' AND sequenceowner <> 'epsx_user'
        LOOP
          EXECUTE format('ALTER SEQUENCE public.%I OWNER TO epsx_user', obj.sequencename);
        END LOOP;

        -- Functions
        FOR obj IN
          SELECT p.proname, pg_get_function_identity_arguments(p.oid) AS args
          FROM pg_proc p JOIN pg_namespace n ON p.pronamespace = n.oid
          WHERE n.nspname = 'public' AND pg_get_userbyid(p.proowner) <> 'epsx_user'
        LOOP
          EXECUTE format('ALTER FUNCTION public.%I(%s) OWNER TO epsx_user', obj.proname, obj.args);
        END LOOP;
      END \$\$;

      -- Set default privileges so future objects are owned by epsx_user
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO epsx_user;
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO epsx_user;
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON FUNCTIONS TO epsx_user;
EOSQL
    ok "Objects in $db reassigned"
  fi
done

# ─── 4. Verification ─────────────────────────────────────────────────────
echo ""
info "Verification:"

echo ""
echo "  Role attributes:"
psql postgres -c "\du epsx_user" 2>/dev/null

echo ""
echo "  Database ownership:"
psql postgres -t -c "SELECT datname, pg_get_userbyid(datdba) AS owner FROM pg_database WHERE datname LIKE 'epsx%' ORDER BY datname;"

echo ""
echo "  Tables NOT owned by epsx_user (should be empty):"
for db in "${DATABASES[@]}"; do
  if psql postgres -t -c "SELECT 1 FROM pg_database WHERE datname = '$db'" | grep -q 1; then
    RESULT=$(psql -d "$db" -t -c "SELECT count(*) FROM pg_tables WHERE schemaname='public' AND tableowner <> 'epsx_user';" 2>/dev/null | tr -d ' ')
    if [ "$RESULT" = "0" ]; then
      ok "$db: all tables owned by epsx_user"
    else
      warn "$db: $RESULT table(s) NOT owned by epsx_user"
      psql -d "$db" -c "SELECT tablename, tableowner FROM pg_tables WHERE schemaname='public' AND tableowner <> 'epsx_user';"
    fi
  fi
done

echo ""
ok "Done! epsx_user is now the consistent database owner."
