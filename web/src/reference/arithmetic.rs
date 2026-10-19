//! Total, checked relations on integer quantities. No floats or implicit unit conversion.
use crate::{Claim, Computation, Error, Result, Term};

pub(crate) fn operator(claim: &Claim) -> Option<Computation> {
    Some(match claim.atom.as_str() {
        "oathkeeper.quantity.add" => Computation::QuantityAdd,
        "oathkeeper.quantity.subtract" => Computation::QuantitySubtract,
        "oathkeeper.quantity.allocate" => Computation::QuantityAllocate,
        _ => return None,
    })
}

pub(crate) fn allocate(amount: i64, basis: i64, total: i64) -> Result<i64> {
    if amount < 0 || basis < 0 || total <= 0 || amount > total {
        return Err(Error(
            "[E_ARITHMETIC] allocation requires 0 <= amount <= total, basis >= 0, total > 0".into(),
        ));
    }
    // i64 nonnegative product and half-divisor fit in i128, including i64::MAX.
    let numerator = i128::from(amount) * i128::from(basis.min(total));
    Ok(((numerator + i128::from(total) / 2) / i128::from(total)) as i64)
}

pub(crate) fn compute(claim: &Claim, op: &Computation) -> Result<()> {
    let (name, arity) = match op {
        Computation::QuantityAdd => ("oathkeeper.quantity.add", 3),
        Computation::QuantitySubtract => ("oathkeeper.quantity.subtract", 3),
        Computation::QuantityAllocate => ("oathkeeper.quantity.allocate", 4),
    };
    if claim.atom != name || claim.negative || claim.arguments.len() != arity {
        return Err(Error(
            "[E_ARITHMETIC] wrong predicate, polarity, or arity".into(),
        ));
    }
    let mut unit = None;
    let mut values = Vec::new();
    for term in &claim.arguments {
        match term {
            Term::Quantity { value, unit: u } if unit.is_none() || unit == Some(u) => {
                unit = Some(u);
                values.push(*value);
            }
            _ => return Err(Error("[E_ARITHMETIC] quantities must share a unit".into())),
        }
    }
    let expected = match op {
        Computation::QuantityAdd => values[0].checked_add(values[1]),
        Computation::QuantitySubtract => values[0].checked_sub(values[1]),
        Computation::QuantityAllocate => Some(allocate(values[0], values[1], values[2])?),
    }
    .ok_or_else(|| Error("[E_ARITHMETIC] integer overflow".into()))?;
    if Some(&expected) != values.last() {
        return Err(Error("[E_ARITHMETIC] incorrect arithmetic witness".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(atom: &str, values: &[i64]) -> Claim {
        Claim {
            atom: format!("oathkeeper.quantity.{atom}"),
            negative: false,
            arguments: values
                .iter()
                .map(|v| Term::Quantity {
                    value: *v,
                    unit: "usd_cent".into(),
                })
                .collect(),
        }
    }
    #[test]
    fn overflow_zero_denominator_and_negative_allocations_are_rejected() {
        assert!(compute(
            &c("add", &[i64::MAX, 1, i64::MIN]),
            &Computation::QuantityAdd
        )
        .is_err());
        assert!(compute(
            &c("subtract", &[i64::MIN, 1, i64::MAX]),
            &Computation::QuantitySubtract
        )
        .is_err());
        for (a, b, d) in [(1, 0, 0), (-1, 1, 2), (1, -1, 2), (3, 1, 2)] {
            assert!(allocate(a, b, d).is_err());
        }
        assert_eq!(allocate(i64::MAX, i64::MAX, i64::MAX).unwrap(), i64::MAX);
        assert_eq!(allocate(1, 1, 2).unwrap(), 1);
        assert_eq!(allocate(1, 1, 3).unwrap(), 0);
    }
    #[test]
    fn bounded_allocation_preserves_amount_and_basis() {
        for amount in 0..30 {
            for basis in 0..30 {
                for total in amount.max(1)..30 {
                    let n = allocate(amount, basis, total).unwrap();
                    assert!(n >= 0 && n <= amount && n <= basis);
                    let exact = i128::from(amount) * i128::from(basis.min(total));
                    let error = (i128::from(n) * i128::from(total) - exact).abs();
                    assert!(2 * error <= i128::from(total));
                }
            }
        }
    }
}
