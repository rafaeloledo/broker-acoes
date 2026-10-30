use b3_analytics::CoreMultiples;
use b3_core::quote::brt_hour_minute;

#[derive(Clone, Copy)]
pub enum MetricId {
    Pl,
    Pvp,
    DividendYield,
    Graham,
    Bazin,
    Lpa,
    Vpa,
    Dpa,
    Outside,
}

pub fn price_caption(fetched_at_ms: Option<i64>) -> String {
    match fetched_at_ms {
        Some(ms) => {
            let (hour, minute) = brt_hour_minute(ms);
            format!("Mercado · {hour:02}:{minute:02}")
        }
        None => "Cotação local".to_string(),
    }
}

pub fn fundamentals_caption(cvm_year: Option<i32>) -> String {
    match cvm_year {
        Some(year) => format!("CVM {year}"),
        None => "Cenário local".to_string(),
    }
}

pub fn change_percent(price: f64, previous_close: f64) -> Option<f64> {
    if previous_close <= 0.0 || !price.is_finite() {
        None
    } else {
        Some((price - previous_close) / previous_close * 100.0)
    }
}

pub fn format_metric(core: Option<&CoreMultiples>, metric: MetricId) -> String {
    let Some(core) = core else {
        return "indisponível".to_string();
    };
    let value = match metric {
        MetricId::Pl => core.pl,
        MetricId::Pvp => core.pvp,
        MetricId::DividendYield => core.dividend_yield,
        MetricId::Graham => core.graham_fair_value,
        MetricId::Bazin => core.bazin_ceiling_price,
        MetricId::Lpa => core.lpa,
        MetricId::Vpa => core.vpa,
        MetricId::Dpa => core.dpa,
        MetricId::Outside => None,
    };
    match (metric, value) {
        (MetricId::Outside, _) | (_, None) => "indisponível".to_string(),
        (MetricId::DividendYield, Some(value)) => format!("{value:.2}%"),
        (MetricId::Graham | MetricId::Bazin | MetricId::Lpa | MetricId::Vpa | MetricId::Dpa, Some(value)) => {
            format!("R$ {value:.2}")
        }
        (_, Some(value)) => format!("{value:.2}"),
    }
}

#[cfg(test)]
mod tests {
    use super::change_percent;

    #[test]
    fn change_uses_previous_close() {
        let change = change_percent(110.0, 100.0).unwrap();
        assert!((change - 10.0).abs() < 1e-9);
    }

    #[test]
    fn change_without_previous_close_is_absent() {
        assert!(change_percent(110.0, 0.0).is_none());
    }
}
