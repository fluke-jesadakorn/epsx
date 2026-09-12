use super::*;
use sha2::{Digest, Sha256};

#[derive(Debug, PartialEq)]
pub enum AdminCreditError {
    Invalid,
    Conflict,
    InsufficientBalance,
    Unavailable,
}
impl From<sqlx::Error> for AdminCreditError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "Admin credit transaction failed");
        Self::Unavailable
    }
}
#[derive(Debug, PartialEq)]
pub struct AdminCreditResult {
    pub transaction_id: Uuid,
    pub balance_after: BigDecimal,
    pub replayed: bool,
}
pub fn valid_wallet(wallet: &str) -> bool {
    wallet.len() == 42
        && wallet.starts_with("0x")
        && wallet[2..].bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn valid_amount(amount: &BigDecimal) -> bool {
    let normalized = amount.normalized();
    let (_, scale) = normalized.as_bigint_and_exponent();
    (-8..=2).contains(&scale)
        && normalized > 0
        && normalized <= BigDecimal::from_str("99999999.99").unwrap()
}
impl CreditRepositoryAdapter {
    pub async fn admin_adjust(
        &self,
        actor: &str,
        key: &str,
        wallet: &str,
        amount: BigDecimal,
        grant: bool,
        reason: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<AdminCreditResult, AdminCreditError> {
        if !valid_wallet(actor)
            || !valid_wallet(wallet)
            || !valid_amount(&amount)
            || key.is_empty()
            || key.len() > 128
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
            || reason.is_some_and(|s| s.len() > 500 || s.chars().any(char::is_control))
            || (!grant && expires_at.is_some())
        {
            return Err(AdminCreditError::Invalid);
        }
        let actor = actor.to_ascii_lowercase();
        let wallet = wallet.to_ascii_lowercase();
        let payload = serde_json::json!({"wallet":wallet,"amount":amount.normalized().to_string(),"grant":grant,"reason":reason,"expires_at":expires_at});
        let hash = format!("{:x}", Sha256::digest(payload.to_string().as_bytes()));
        let mut tx = self.db_pool.begin().await?;
        sqlx::query("INSERT INTO public.admin_credit_commands (actor,idempotency_key,payload_hash) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING")
            .bind(&actor).bind(key).bind(&hash).execute(&mut *tx).await?;
        let (existing_hash, transaction_id, balance_after): (String,Option<Uuid>,Option<BigDecimal>) =
            sqlx::query_as("SELECT payload_hash,transaction_id,balance_after FROM public.admin_credit_commands WHERE actor=$1 AND idempotency_key=$2 FOR UPDATE")
            .bind(&actor).bind(key).fetch_one(&mut *tx).await?;
        if existing_hash != hash {
            return Err(AdminCreditError::Conflict);
        }
        if let (Some(transaction_id), Some(balance_after)) = (transaction_id, balance_after) {
            tx.commit().await?;
            return Ok(AdminCreditResult {
                transaction_id,
                balance_after,
                replayed: true,
            });
        }
        if expires_at.is_some_and(|value| value <= Utc::now()) {
            return Err(AdminCreditError::Invalid);
        }
        sqlx::query("INSERT INTO public.wallet_credits (wallet_address,balance) VALUES ($1,0) ON CONFLICT DO NOTHING")
            .bind(&wallet).execute(&mut *tx).await?;
        let (balance,pending): (BigDecimal,BigDecimal) = sqlx::query_as("SELECT balance,pending_balance FROM public.wallet_credits WHERE wallet_address=$1 FOR UPDATE")
            .bind(&wallet).fetch_one(&mut *tx).await?;
        if !grant && &balance - &pending < amount {
            return Err(AdminCreditError::InsufficientBalance);
        }
        if grant && &balance + &amount > BigDecimal::from_str("99999999.99").unwrap() {
            return Err(AdminCreditError::Invalid);
        }
        let signed = if grant { amount } else { -amount };
        let transaction_id: Uuid = sqlx::query_scalar("SELECT public.add_credit_transaction($1,$2,$3,NULL,'admin_action',$4,$5,$6,'{}'::jsonb)")
            .bind(&wallet).bind(signed).bind(if grant {"grant"} else {"revoke"}).bind(reason).bind(&actor).bind(expires_at)
            .fetch_one(&mut *tx).await?;
        let balance_after: BigDecimal =
            sqlx::query_scalar("SELECT balance_after FROM public.credit_transactions WHERE id=$1")
                .bind(transaction_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query("UPDATE public.admin_credit_commands SET transaction_id=$3,balance_after=$4 WHERE actor=$1 AND idempotency_key=$2")
            .bind(&actor).bind(key).bind(transaction_id).bind(&balance_after).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(AdminCreditResult {
            transaction_id,
            balance_after,
            replayed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credit_amounts_never_round_or_overflow_the_existing_ledger() {
        for value in ["0.01", "1.2500", "99999999.99"] {
            assert!(
                valid_amount(&BigDecimal::from_str(value).unwrap()),
                "{value}"
            );
        }
        for value in ["0", "-1", "0.001", "100000000", "1e100000", "1e-100000"] {
            assert!(
                !valid_amount(&BigDecimal::from_str(value).unwrap()),
                "{value}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires isolated EPSX_CREDIT_TEST_DATABASE"]
    async fn admin_credit_commands_replay_serialize_and_rollback_with_the_ledger() {
        let pool = Arc::new(
            sqlx::PgPool::connect(&std::env::var("EPSX_CREDIT_TEST_DATABASE").unwrap())
                .await
                .unwrap(),
        );
        let name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
        assert!(name.starts_with("epsx_credit_check_"));
        let repo = CreditRepositoryAdapter::new(pool.clone());
        let actor = format!("0x00000000{}", Uuid::new_v4().simple());
        let wallet = format!("0x00000000{}", Uuid::new_v4().simple());
        let amount = BigDecimal::from_str("12.50").unwrap();
        let (a, b) = tokio::join!(
            repo.admin_adjust(
                &actor,
                "grant-1",
                &wallet,
                amount.clone(),
                true,
                Some("Rehearsal"),
                None
            ),
            repo.admin_adjust(
                &actor,
                "grant-1",
                &wallet,
                amount.clone(),
                true,
                Some("Rehearsal"),
                None
            )
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        assert_eq!(a.transaction_id, b.transaction_id);
        assert_ne!(a.replayed, b.replayed);
        assert_eq!(a.balance_after, amount);
        let ledger_actor: String =
            sqlx::query_scalar("SELECT granted_by FROM public.credit_transactions WHERE id=$1")
                .bind(a.transaction_id)
                .fetch_one(pool.as_ref())
                .await
                .unwrap();
        assert_eq!(ledger_actor, actor);
        assert_eq!(
            repo.admin_adjust(
                &actor,
                "grant-1",
                &wallet,
                BigDecimal::from(1),
                true,
                Some("Rehearsal"),
                None
            )
            .await,
            Err(AdminCreditError::Conflict)
        );
        sqlx::query("UPDATE public.wallet_credits SET pending_balance=10 WHERE wallet_address=$1")
            .bind(&wallet)
            .execute(pool.as_ref())
            .await
            .unwrap();
        assert_eq!(
            repo.admin_adjust(
                &actor,
                "revoke-1",
                &wallet,
                BigDecimal::from(3),
                false,
                None,
                None
            )
            .await,
            Err(AdminCreditError::InsufficientBalance)
        );
        let amount = BigDecimal::from_str("2.50").unwrap();
        let revoke = repo
            .admin_adjust(
                &actor,
                "revoke-1",
                &wallet,
                amount.clone(),
                false,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(revoke.balance_after, BigDecimal::from(10));
        let replay = repo
            .admin_adjust(&actor, "revoke-1", &wallet, amount, false, None, None)
            .await
            .unwrap();
        assert_eq!(replay.transaction_id, revoke.transaction_id);
        assert!(replay.replayed);
        // A failed completion write must roll back the balance and ledger insert.
        sqlx::raw_sql("CREATE FUNCTION public.reject_credit_command_finish() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'rehearsal completion failure'; END $$; CREATE TRIGGER reject_credit_command_finish BEFORE UPDATE ON public.admin_credit_commands FOR EACH ROW EXECUTE FUNCTION public.reject_credit_command_finish();").execute(pool.as_ref()).await.unwrap();
        assert_eq!(
            repo.admin_adjust(
                &actor,
                "grant-failure",
                &wallet,
                BigDecimal::from(1),
                true,
                None,
                None
            )
            .await,
            Err(AdminCreditError::Unavailable)
        );
        assert_eq!(
            repo.get_balance(&wallet).await.unwrap().unwrap().balance,
            BigDecimal::from(10)
        );
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM public.credit_transactions WHERE wallet_address=$1",
        )
        .bind(&wallet)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();
        assert_eq!(count, 2);
        sqlx::raw_sql("DROP TRIGGER reject_credit_command_finish ON public.admin_credit_commands; DROP FUNCTION public.reject_credit_command_finish();").execute(pool.as_ref()).await.unwrap();
        pool.close().await;
    }
}
