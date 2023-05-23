fn percentage(cur: f32, tot: f32) -> Option<f32> {
    // TODO Consider Result<Option<f32>>:
    //      NaNs are essentially options, but other cases are more like errors.
    match () {
        _ if cur.is_nan() || tot.is_nan() => {
            tracing::error!(
                "one of the values is NaN. cur:{:?}, tot:{:?}",
                cur,
                tot
            );
            None
        }
        _ if tot == 0.0 => {
            tracing::error!(
                "the total value is zero. cur:{:?}, tot:{:?}.",
                cur,
                tot
            );
            None
        }
        _ if cur > tot => {
            tracing::error!(
                "the current value exceeds total. cur:{:?}, tot:{:?}.",
                cur,
                tot
            );
            None
        }
        _ => Some((cur / tot) * 100.0),
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn percentage_floor(cur: f32, max: f32) -> Option<u64> {
    percentage(cur, max).map(|pct| pct.floor() as u64)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn percentage_round(cur: f32, max: f32) -> Option<u64> {
    percentage(cur, max).map(|pct| pct.round() as u64)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn percentage_ceiling(cur: f32, max: f32) -> Option<u64> {
    percentage(cur, max).map(|pct| pct.ceil() as u64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn t_percentage() {
        // TODO prop tests?
        assert_eq!(None, super::percentage(1.0, f32::NAN));
        assert_eq!(None, super::percentage(f32::NAN, 1.0));
        assert_eq!(None, super::percentage(2.0, 1.0));
        assert_eq!(Some(50.0), super::percentage(1.0, 2.0));
    }
}
