use super::super::{
    ChainId, LeaseDuration, LeaseFence, LeaseGrant, LeaseOwner, SelectedChainRepositoryError,
    SelectionBoundaryError, SelectionConflict,
};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};

#[derive(Debug, FromRow)]
struct LockedLeaseRow {
    lease_owner: Option<String>,
    lease_fence: i64,
    lease_expires_at: Option<DateTime<Utc>>,
    database_now: DateTime<Utc>,
}

#[derive(Debug)]
struct StoredLease {
    owner: Option<LeaseOwner>,
    fence: Option<LeaseFence>,
    expires_at: Option<DateTime<Utc>>,
    database_now: DateTime<Utc>,
}

pub(super) async fn acquire(
    pool: &PgPool,
    chain_id: ChainId,
    owner: LeaseOwner,
    duration: LeaseDuration,
) -> Result<LeaseGrant, SelectedChainRepositoryError> {
    let chain_id_value = chain_id_value(chain_id)?;
    let duration_micros = duration_micros(duration)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| map_sqlx_error("begin lease acquisition", error))?;

    sqlx::query(
        r#"
        INSERT INTO public.indexer_chain_state (chain_id)
        VALUES ($1)
        ON CONFLICT (chain_id) DO NOTHING
        "#,
    )
    .bind(chain_id_value)
    .execute(&mut *transaction)
    .await
    .map_err(|error| map_sqlx_error("ensure chain lease row", error))?;

    let stored = lock_lease_row(&mut transaction, chain_id_value).await?;
    if stored.owner.is_some()
        && stored
            .expires_at
            .is_some_and(|expires_at| expires_at > stored.database_now)
    {
        // The in-memory protocol rejects all live reacquisition, including by
        // the current owner. An expired grant may be taken over by any owner.
        return Err(SelectionConflict::LeaseHeld { chain_id }.into());
    }

    let fence = LeaseFence::successor(stored.fence)?;
    let expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        UPDATE public.indexer_chain_state
        SET lease_owner = $2,
            lease_fence = $3,
            lease_expires_at = clock_timestamp()
                + ($4::double precision * INTERVAL '1 microsecond'),
            updated_at = clock_timestamp()
        WHERE chain_id = $1
        RETURNING lease_expires_at
        "#,
    )
    .bind(chain_id_value)
    .bind(owner.as_str())
    .bind(fence_value(fence)?)
    .bind(duration_micros)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| map_sqlx_error("write acquired chain lease", error))?;

    transaction
        .commit()
        .await
        .map_err(|error| map_sqlx_error("commit lease acquisition", error))?;

    Ok(LeaseGrant::new(chain_id, owner, fence, expires_at))
}

pub(super) async fn renew(
    pool: &PgPool,
    grant: &LeaseGrant,
    duration: LeaseDuration,
) -> Result<LeaseGrant, SelectedChainRepositoryError> {
    let chain_id_value = chain_id_value(grant.chain_id())?;
    let fence_value = fence_value(grant.fence())?;
    let duration_micros = duration_micros(duration)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| map_sqlx_error("begin lease renewal", error))?;

    let Some(stored) = lock_optional_lease_row(&mut transaction, chain_id_value).await? else {
        return Err(SelectionConflict::StaleLease.into());
    };
    require_exact_live_grant(&stored, grant)?;

    let expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        UPDATE public.indexer_chain_state
        SET lease_expires_at = clock_timestamp()
                + ($4::double precision * INTERVAL '1 microsecond'),
            updated_at = clock_timestamp()
        WHERE chain_id = $1
          AND lease_owner = $2
          AND lease_fence = $3
          AND lease_expires_at > clock_timestamp()
        RETURNING lease_expires_at
        "#,
    )
    .bind(chain_id_value)
    .bind(grant.owner().as_str())
    .bind(fence_value)
    .bind(duration_micros)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| map_sqlx_error("renew chain lease", error))?
    .ok_or(SelectionConflict::StaleLease)?;

    transaction
        .commit()
        .await
        .map_err(|error| map_sqlx_error("commit lease renewal", error))?;

    Ok(LeaseGrant::new(
        grant.chain_id(),
        grant.owner().clone(),
        grant.fence(),
        expires_at,
    ))
}

pub(super) async fn release(
    pool: &PgPool,
    grant: &LeaseGrant,
) -> Result<(), SelectedChainRepositoryError> {
    let chain_id_value = chain_id_value(grant.chain_id())?;
    let fence_value = fence_value(grant.fence())?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| map_sqlx_error("begin lease release", error))?;

    let Some(stored) = lock_optional_lease_row(&mut transaction, chain_id_value).await? else {
        return Err(SelectionConflict::StaleLease.into());
    };
    require_exact_live_grant(&stored, grant)?;

    let released = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE public.indexer_chain_state
        SET lease_owner = NULL,
            lease_expires_at = NULL,
            updated_at = clock_timestamp()
        WHERE chain_id = $1
          AND lease_owner = $2
          AND lease_fence = $3
          AND lease_expires_at > clock_timestamp()
        RETURNING lease_fence
        "#,
    )
    .bind(chain_id_value)
    .bind(grant.owner().as_str())
    .bind(fence_value)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| map_sqlx_error("release chain lease", error))?;

    if released != Some(fence_value) {
        return Err(SelectionConflict::StaleLease.into());
    }

    transaction
        .commit()
        .await
        .map_err(|error| map_sqlx_error("commit lease release", error))?;
    Ok(())
}

async fn lock_lease_row(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: i64,
) -> Result<StoredLease, SelectedChainRepositoryError> {
    lock_optional_lease_row(transaction, chain_id)
        .await?
        .ok_or_else(|| {
            SelectedChainRepositoryError::CorruptState(
                "chain lease row disappeared while locked".into(),
            )
        })
}

async fn lock_optional_lease_row(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: i64,
) -> Result<Option<StoredLease>, SelectedChainRepositoryError> {
    let row = sqlx::query_as::<_, LockedLeaseRow>(
        r#"
        SELECT lease_owner,
               lease_fence,
               lease_expires_at,
               clock_timestamp() AS database_now
        FROM public.indexer_chain_state
        WHERE chain_id = $1
        FOR UPDATE
        "#,
    )
    .bind(chain_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| map_sqlx_error("lock chain lease row", error))?;

    row.map(parse_stored_lease).transpose()
}

fn parse_stored_lease(row: LockedLeaseRow) -> Result<StoredLease, SelectedChainRepositoryError> {
    if row.lease_fence < 0 {
        return Err(corrupt_state("stored lease fence is negative"));
    }
    let fence = if row.lease_fence == 0 {
        None
    } else {
        Some(
            LeaseFence::new(row.lease_fence as u64)
                .map_err(|_| corrupt_state("stored lease fence is outside its domain"))?,
        )
    };

    let (owner, expires_at) = match (row.lease_owner, row.lease_expires_at) {
        (None, None) => (None, None),
        (Some(owner), Some(expires_at)) => {
            if fence.is_none() {
                return Err(corrupt_state("stored live lease has a zero fence"));
            }
            let owner = LeaseOwner::new(owner)
                .map_err(|_| corrupt_state("stored lease owner is outside its domain"))?;
            (Some(owner), Some(expires_at))
        }
        _ => {
            return Err(corrupt_state(
                "stored lease owner and expiration are not paired",
            ));
        }
    };

    Ok(StoredLease {
        owner,
        fence,
        expires_at,
        database_now: row.database_now,
    })
}

fn require_exact_live_grant(
    stored: &StoredLease,
    grant: &LeaseGrant,
) -> Result<(), SelectedChainRepositoryError> {
    if stored.owner.as_ref() != Some(grant.owner())
        || stored.fence != Some(grant.fence())
        || stored
            .expires_at
            .is_none_or(|expires_at| expires_at <= stored.database_now)
    {
        return Err(SelectionConflict::StaleLease.into());
    }
    Ok(())
}

fn duration_micros(duration: LeaseDuration) -> Result<i64, SelectedChainRepositoryError> {
    let duration = duration.get();
    let seconds_micros = duration
        .as_secs()
        .checked_mul(1_000_000)
        .ok_or(SelectionBoundaryError::InvalidLeaseDuration)?;
    let subsecond_micros = u64::from(duration.subsec_nanos()).div_ceil(1_000);
    let micros = seconds_micros
        .checked_add(subsecond_micros)
        .ok_or(SelectionBoundaryError::InvalidLeaseDuration)?
        .max(1);
    i64::try_from(micros).map_err(|_| SelectionBoundaryError::InvalidLeaseDuration.into())
}

fn chain_id_value(chain_id: ChainId) -> Result<i64, SelectedChainRepositoryError> {
    i64::try_from(chain_id.get()).map_err(|_| {
        SelectedChainRepositoryError::CorruptState("validated chain id exceeds BIGINT".into())
    })
}

fn fence_value(fence: LeaseFence) -> Result<i64, SelectedChainRepositoryError> {
    i64::try_from(fence.get()).map_err(|_| {
        SelectedChainRepositoryError::CorruptState("validated lease fence exceeds BIGINT".into())
    })
}

fn map_sqlx_error(operation: &'static str, error: sqlx::Error) -> SelectedChainRepositoryError {
    match error {
        error @ (sqlx::Error::ColumnDecode { .. } | sqlx::Error::Decode(_)) => {
            corrupt_state(format!("{operation}: {error}"))
        }
        error => SelectedChainRepositoryError::Unavailable(format!("{operation}: {error}")),
    }
}

fn corrupt_state(message: impl Into<String>) -> SelectedChainRepositoryError {
    SelectedChainRepositoryError::CorruptState(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn duration_conversion_rounds_up_to_database_microseconds() {
        let one_nanosecond = LeaseDuration::new(Duration::from_nanos(1)).unwrap();
        let exact_microsecond = LeaseDuration::new(Duration::from_micros(1)).unwrap();
        let fractional_microsecond = LeaseDuration::new(Duration::from_nanos(1_001)).unwrap();

        assert_eq!(duration_micros(one_nanosecond).unwrap(), 1);
        assert_eq!(duration_micros(exact_microsecond).unwrap(), 1);
        assert_eq!(duration_micros(fractional_microsecond).unwrap(), 2);
    }

    #[test]
    fn duration_conversion_preserves_the_maximum_domain_value() {
        let maximum = LeaseDuration::new(Duration::from_secs(24 * 60 * 60)).unwrap();
        assert_eq!(duration_micros(maximum).unwrap(), 86_400_000_000);
    }

    #[test]
    fn transport_errors_map_to_unavailable() {
        let error = map_sqlx_error("test operation", sqlx::Error::RowNotFound);
        assert!(matches!(
            error,
            SelectedChainRepositoryError::Unavailable(_)
        ));
    }

    #[test]
    fn decoded_database_values_map_to_corrupt_state() {
        let source = std::io::Error::other("malformed database value");
        let error = map_sqlx_error("test operation", sqlx::Error::Decode(Box::new(source)));
        assert!(matches!(
            error,
            SelectedChainRepositoryError::CorruptState(_)
        ));
    }
}
