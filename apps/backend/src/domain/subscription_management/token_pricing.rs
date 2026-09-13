//! Exact catalog pricing shared by public projections, quotes and new orders.
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
pub struct TokenPricing {
    pub original_price: String,
    pub price: String,
    pub savings: String,
    pub promotion_active: bool,
    pub promotion_status: String,
    pub promotion_discount: f64,
    pub promotion_ends_at: Option<String>,
}

fn decimal(value: &Value) -> Result<BigDecimal, &'static str> {
    let text = value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string());
    // Bound exponents/scale before arithmetic on administrator input.
    if text.len() > 78 || !text.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Err("invalid_promotion_amount");
    }
    BigDecimal::from_str(&text).map_err(|_| "invalid_promotion_amount")
}

fn money(value: BigDecimal) -> String {
    if value == 0 {
        return "0.00".into();
    }
    let value = value.normalized();
    if value.fractional_digit_count() < 2 {
        value.with_scale(2).to_string()
    } else {
        value.to_string()
    }
}

fn date(p: &Value, key: &str) -> Result<Option<DateTime<Utc>>, &'static str> {
    let Some(value) = p.get(key).filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let text = value.as_str().ok_or("invalid_promotion_date")?;
    if text.is_empty() {
        return Ok(None);
    }
    DateTime::parse_from_rfc3339(text)
        .map(|v| Some(v.with_timezone(&Utc)))
        .map_err(|_| "invalid_promotion_date")
}

pub fn calculate(
    metadata: &Value,
    token: &str,
    now: DateTime<Utc>,
) -> Result<TokenPricing, &'static str> {
    if !["USDT", "USDC"].contains(&token) {
        return Err("unsupported_token");
    }
    let base = decimal(&metadata["pay_prices"][token])?;
    if base <= 0 {
        return Err("invalid_token_price");
    }
    let p = &metadata["promotion"];
    let enabled = metadata["pay_use_catalog_promotion"] == true && p["enabled"] == true;
    let mut status = "disabled";
    let mut end = None;
    let mut price = base.clone();
    if enabled {
        let start = date(p, "start_date")?;
        end = date(p, "end_date")?;
        if start.zip(end).is_some_and(|(s, e)| s >= e) {
            return Err("invalid_promotion_window");
        }
        let custom = p.get("price").map(decimal).transpose()?.unwrap_or_default();
        let value = p.get("value").map(decimal).transpose()?.unwrap_or_default();
        let discounted = if custom > 0 {
            custom
        } else {
            match p["type"].as_str() {
                Some("percentage") if value > 0 && value < 100 => {
                    &base * (BigDecimal::from(100) - value) / BigDecimal::from(100)
                }
                Some("fixed") if value > 0 => &base - value,
                _ => return Err("invalid_promotion_discount"),
            }
        };
        if discounted.normalized().fractional_digit_count() > 18 {
            return Err("token_price_precision_exceeded");
        }
        if discounted <= 0 || discounted >= base {
            return Err("invalid_promotion_discount");
        }
        status = if start.is_some_and(|s| now < s) {
            "upcoming"
        } else if end.is_some_and(|e| now >= e) {
            "expired"
        } else {
            "active"
        };
        if status == "active" {
            price = discounted;
        }
    }
    if price.normalized().fractional_digit_count() > 18
        || base.normalized().fractional_digit_count() > 18
    {
        return Err("token_price_precision_exceeded");
    }
    let savings = &base - &price;
    let discount = (&savings * BigDecimal::from(100) / &base)
        .to_string()
        .parse()
        .unwrap_or_default();
    Ok(TokenPricing {
        original_price: money(base),
        price: money(price),
        savings: money(savings),
        promotion_active: status == "active",
        promotion_status: status.into(),
        promotion_discount: discount,
        promotion_ends_at: end.map(|e| e.to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn metadata() -> Value {
        json!({"pay_prices":{"USDT":"99","USDC":"99"},"pay_use_catalog_promotion":true,"promotion":{"enabled":true,"type":"percentage","value":90,"start_date":"2026-09-01T00:00:00Z","end_date":"2026-10-01T00:00:00Z"}})
    }
    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }
    #[test]
    fn exact_shared_token_pricing_and_schedule() {
        for token in ["USDT", "USDC"] {
            let m = metadata();
            let sale = calculate(&m, token, at("2026-09-09T00:00:00Z")).unwrap();
            assert_eq!(
                (
                    sale.original_price.as_str(),
                    sale.price.as_str(),
                    sale.savings.as_str()
                ),
                ("99.00", "9.90", "89.10")
            );
            for (time, status) in [
                ("2026-08-31T00:00:00Z", "upcoming"),
                ("2026-10-01T00:00:00Z", "expired"),
            ] {
                let p = calculate(&m, token, at(time)).unwrap();
                assert_eq!(p.price, "99.00");
                assert_eq!(p.promotion_status, status);
                assert_eq!(p.savings, "0.00");
            }
        }
    }
    #[test]
    fn fixed_custom_disabled_and_independent_prices() {
        let now = at("2026-09-09T00:00:00Z");
        let mut m = metadata();
        m["promotion"]["type"] = json!("fixed");
        m["promotion"]["value"] = json!(89.1);
        assert_eq!(calculate(&m, "USDT", now).unwrap().price, "9.90");
        m["promotion"]["price"] = json!(7.25);
        assert_eq!(calculate(&m, "USDT", now).unwrap().price, "7.25");
        m["promotion"]["enabled"] = json!(false);
        assert_eq!(calculate(&m, "USDT", now).unwrap().price, "99.00");
        m["promotion"]["enabled"] = json!(true);
        m["pay_use_catalog_promotion"] = json!(false);
        assert_eq!(calculate(&m, "USDT", now).unwrap().price, "99.00");
    }
    #[test]
    fn malformed_or_excessive_discount_is_rejected() {
        for (key, value) in [
            ("start_date", json!("bad")),
            ("end_date", json!(123)),
            ("end_date", json!("2026-08-01T00:00:00Z")),
            ("value", json!(100)),
            ("value", json!(-1)),
            ("price", json!(100)),
        ] {
            let mut m = metadata();
            m["promotion"][key] = value;
            assert!(calculate(&m, "USDT", at("2026-09-09T00:00:00Z")).is_err());
        }
    }
}
