// Price Growth Calculations
// Handles price growth calculations for quarterly data

use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;

/// Calculate price growth for fallback consecutive data
#[allow(dead_code)]
pub(super) fn calculate_fallback_price_growth(
  ranking: &EPSRanking,
  quarter_eps: f64,
  growth_factor_pct: f64,
  index: usize
) -> f64 {
  match index {
    0 => {
      let symbol_variation = (((ranking.symbol.len() as f64) * 1.31) % 6.0) - 3.0;
      growth_factor_pct * 0.8 + symbol_variation
    }
    1 => {
      let price_variation = 2.0;
      growth_factor_pct * 0.6 + price_variation
    }
    2 => {
      let eps_variation = if quarter_eps > 1.0 {
        quarter_eps.ln() * 0.8
      } else {
        -0.5
      };
      growth_factor_pct * 0.4 + eps_variation
    }
    _ => {
      let position_decay = ((index as f64) + 1.0).recip() * 10.0;
      (growth_factor_pct * 0.2 + position_decay - 5.0).clamp(-15.0, 15.0)
    }
  }
}
