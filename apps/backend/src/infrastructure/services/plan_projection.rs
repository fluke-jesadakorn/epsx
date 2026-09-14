//! Maintain the payments read projection from the canonical core catalog.
//! Prices, metadata and numeric precision are transferred as PostgreSQL JSON text.
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};
use std::time::Duration;
type Error = Box<dyn std::error::Error + Send + Sync>;
static HEALTHY: OnceLock<AtomicBool> = OnceLock::new();
pub fn healthy() -> Option<bool> {
    HEALTHY.get().map(|v| v.load(Ordering::Acquire))
}
async fn columns(pool: &PgPool, schema: &str) -> Result<Vec<(String, String)>, Error> {
    Ok(sqlx::query_as("SELECT column_name::text, concat_ws(':',data_type,udt_name,is_nullable,character_maximum_length,numeric_precision,numeric_scale) FROM information_schema.columns WHERE table_schema=$1 AND table_name='plans' ORDER BY column_name")
        .bind(schema).fetch_all(pool).await?)
}
fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
pub async fn reconcile(core: &PgPool, payments: &PgPool) -> Result<(), Error> {
    // Serialize before reading core, so overlapping runs cannot apply an older
    // snapshot after a newer one. The lock also coordinates with manual repair.
    let mut target = payments.begin().await?;
    sqlx::query("LOCK TABLE payments.plans IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *target)
        .await?;
    let source_columns = columns(core, "public").await?;
    if source_columns.is_empty() || source_columns != columns(payments, "payments").await? {
        return Err("core/payments plan schemas differ; explicit migration required".into());
    }
    let mut source = core.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *source)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM public.plans")
        .fetch_one(&mut *source)
        .await?;
    if !(1..=10_000).contains(&count) {
        return Err("canonical plan catalog must contain 1..10000 records".into());
    }
    let snapshot: String =
        sqlx::query_scalar("SELECT jsonb_agg(to_jsonb(p) ORDER BY id)::text FROM public.plans p")
            .fetch_one(&mut *source)
            .await?;
    source.commit().await?;
    if snapshot.len() > 16 * 1024 * 1024 {
        return Err("plan snapshot exceeds size limit".into());
    }
    let names = source_columns
        .iter()
        .map(|(name, _)| identifier(name))
        .collect::<Vec<_>>();
    let updates = source_columns
        .iter()
        .filter(|(name, _)| name != "id")
        .map(|(name, _)| format!("{0}=EXCLUDED.{0}", identifier(name)))
        .collect::<Vec<_>>();
    let sql = format!("INSERT INTO payments.plans ({0}) SELECT {0} FROM jsonb_populate_recordset(NULL::payments.plans,$1::jsonb) ON CONFLICT (id) DO UPDATE SET {1}",names.join(","),updates.join(","));
    sqlx::query(&sql)
        .bind(&snapshot)
        .execute(&mut *target)
        .await?;
    // Keep historical rows referenced by old payments after a catalog deletion.
    sqlx::query("UPDATE payments.plans SET is_active=false, is_public=false, is_promoted=false WHERE (is_active OR is_public OR is_promoted) AND id NOT IN (SELECT (p->>'id')::uuid FROM jsonb_array_elements($1::jsonb) p)").bind(&snapshot).execute(&mut *target).await?;
    let equal: bool = sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM jsonb_array_elements($1::jsonb) s LEFT JOIN payments.plans p ON p.id=(s->>'id')::uuid WHERE to_jsonb(p) IS DISTINCT FROM s)").bind(&snapshot).fetch_one(&mut *target).await?;
    if !equal {
        return Err("plan projection differs after upsert; transaction rolled back".into());
    }
    target.commit().await?;
    Ok(())
}
async fn checked(core: &PgPool, payments: &PgPool) -> Result<(), Error> {
    tokio::time::timeout(Duration::from_secs(15), reconcile(core, payments))
        .await
        .map_err(|_| "plan projection synchronization timed out")?
}
pub async fn start(core: PgPool, payments: PgPool) -> Result<(), Error> {
    let health = HEALTHY.get_or_init(|| AtomicBool::new(false));
    checked(&core, &payments).await?;
    health.store(true, Ordering::Release);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let result = checked(&core, &payments).await;
            health.store(result.is_ok(), Ordering::Release);
            if let Err(error) = result {
                tracing::error!(%error,"Plan projection unavailable");
            }
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn isolated(variable: &str) -> PgPool {
        let pool = PgPool::connect(&std::env::var(variable).expect(variable))
            .await
            .unwrap();
        let name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(
            name.starts_with("epsx_projection_check_"),
            "isolated clone required"
        );
        pool
    }
    async fn snapshot(pool: &PgPool) -> String {
        sqlx::query_scalar("SELECT jsonb_agg(to_jsonb(p) ORDER BY id)::text FROM payments.plans p")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires fresh EPSX_PROJECTION_CORE/PAYMENTS isolated production clones"]
    async fn imported_catalog_updates_are_exact_atomic_and_preserve_history() {
        let core = isolated("EPSX_PROJECTION_CORE").await;
        let payments = isolated("EPSX_PROJECTION_PAYMENTS").await;
        reconcile(&core, &payments).await.unwrap();
        let id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM public.plans ORDER BY id LIMIT 1")
            .fetch_one(&core)
            .await
            .unwrap();
        sqlx::query("UPDATE public.plans SET price=12345.67, display_order=91, name='Projection rehearsal', plan_metadata=plan_metadata || '{\"projection_rehearsal\":9007199254740993}'::jsonb WHERE id=$1")
            .bind(id).execute(&core).await.unwrap();
        // Simulate an old purchased plan retained only in the payments database.
        let historical = uuid::Uuid::new_v4();
        sqlx::query("INSERT INTO payments.plans SELECT (jsonb_populate_record(NULL::payments.plans, to_jsonb(p) || jsonb_build_object('id',$2::uuid,'slug',$2::text,'name','Historical rehearsal','is_active',true,'is_public',true,'is_promoted',true))).* FROM payments.plans p WHERE id=$1")
            .bind(id).bind(historical).execute(&payments).await.unwrap();
        reconcile(&core, &payments).await.unwrap();
        let source: String =
            sqlx::query_scalar("SELECT to_jsonb(p)::text FROM public.plans p WHERE id=$1")
                .bind(id)
                .fetch_one(&core)
                .await
                .unwrap();
        let target: String =
            sqlx::query_scalar("SELECT to_jsonb(p)::text FROM payments.plans p WHERE id=$1")
                .bind(id)
                .fetch_one(&payments)
                .await
                .unwrap();
        assert_eq!(source, target, "every column and numeric value preserved");
        assert!(source.contains("9007199254740993"));
        let flags: (bool, bool, bool) = sqlx::query_as(
            "SELECT is_active,is_public,is_promoted FROM payments.plans WHERE id=$1",
        )
        .bind(historical)
        .fetch_one(&payments)
        .await
        .unwrap();
        assert_eq!(flags, (false, false, false));
        let before = snapshot(&payments).await;
        reconcile(&core, &payments).await.unwrap();
        assert_eq!(before, snapshot(&payments).await, "retries are idempotent");

        sqlx::query("UPDATE public.plans SET description='New source revision' WHERE id=$1")
            .bind(id)
            .execute(&core)
            .await
            .unwrap();
        sqlx::query("ALTER TABLE payments.plans ADD COLUMN rehearsal_schema_drift text")
            .execute(&payments)
            .await
            .unwrap();
        assert!(reconcile(&core, &payments)
            .await
            .unwrap_err()
            .to_string()
            .contains("schemas differ"));
        sqlx::query("ALTER TABLE payments.plans DROP COLUMN rehearsal_schema_drift")
            .execute(&payments)
            .await
            .unwrap();
        assert_eq!(
            before,
            snapshot(&payments).await,
            "schema drift must not write rows"
        );

        // A trigger changing a projected field must roll back the entire batch.
        sqlx::query("CREATE FUNCTION payments.rehearsal_corrupt_projection() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN NEW.description='Corrupted projection'; RETURN NEW; END $$")
            .execute(&payments).await.unwrap();
        sqlx::query("CREATE TRIGGER rehearsal_corrupt BEFORE UPDATE ON payments.plans FOR EACH ROW EXECUTE FUNCTION payments.rehearsal_corrupt_projection()")
            .execute(&payments).await.unwrap();
        assert!(reconcile(&core, &payments)
            .await
            .unwrap_err()
            .to_string()
            .contains("differs after upsert"));
        assert_eq!(
            before,
            snapshot(&payments).await,
            "failed verification must roll back all rows"
        );
        sqlx::query("DROP TRIGGER rehearsal_corrupt ON payments.plans")
            .execute(&payments)
            .await
            .unwrap();
        reconcile(&core, &payments).await.unwrap();
        core.close().await;
        payments.close().await;
    }
}
