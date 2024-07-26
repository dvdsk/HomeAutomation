struct FmtOption {
    unit: &'static str,
    factor: usize,
    /// at a max display this numb of items
    /// before going to the bigger FmtOption
    next: usize,
}

impl FmtOption {
    fn scale(&self, seconds: f64) -> usize {
        (seconds / self.factor as f64).floor() as usize
    }
}

pub fn fmt_seconds(secs: f64) -> String {
    #[rustfmt::skip]
    let mut options = [
        // next is purposefully set wrong, we set it in a loop below
        FmtOption{unit: "s", factor: 1, next: 0},
        FmtOption{unit: "m", factor: 60, next: 0},
        FmtOption{unit: "h", factor: 60 * 60, next: 0},
        FmtOption{unit: "d", factor: 60 * 60 * 24, next: 0},
        FmtOption{unit: "w", factor: 60 * 60 * 24 * 7, next: 0},
        FmtOption{unit: "y", factor: 60 * 60 * 24 * 365, next: 0},
    ];

    let mut next = usize::MAX;
    for fmt in options.iter_mut().rev() {
        fmt.next = next;
        next = fmt.factor;
    }

    let mut small = &options[0];
    for big in &options[1..] {
        if big.scale(secs) == 0 {
            return format!("{}{}", small.scale(secs), small.unit);
        }

        if secs >= big.next as f64 {
            small = big;
            continue;
        }

        if small.scale(secs % small.next as f64) == 0 {
            return format!("{}{}", big.scale(secs), big.unit);
        }

        return format!(
            "{}{}{}{}",
            big.scale(secs),
            big.unit,
            small.scale(secs % small.next as f64),
            small.unit
        );
    }

    unreachable!()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_x() {
        let cases = [
            (5, "5s"),
            (92, "1m32s"),
            (60 * 60 * 12, "12h"),
            (60 * 60 * 24 * 2, "2d"),
            (60 * 60 * 24 * 7, "1w"),
            (60 * 60 * 24 * 366, "1y"),
            (60 * 60 * 24 * 365 * 14 + 60 * 60 * 24 * 7 * 6, "14y6w"),
        ];

        for (input, correct_output) in cases {
            assert_eq!(fmt_seconds(input as f64), correct_output);
        }
    }
}
