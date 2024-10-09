use std::iter;

use itertools::Itertools;

use crate::tui::readings::sensor_info::ChartParts;

/// split data into separate lines if
/// - the gap between point groups is larger then what is expected for
///   the given sensor with 20% margin
/// and
/// - the gap is larger then the medium in the data
///
/// # Note
/// the splits line groups are returned from right to left
///
// An alternative would have been to use the median though that is slower and
// we have this information anyway
pub(crate) fn split<'a>(
    chart: &'a ChartParts,
    chart_width: u16,
) -> impl Iterator<Item = &'a [(f64, f64)]> {
    let min_dist = chart
        .reading
        .device
        .info()
        .max_sample_interval
        .as_secs_f64()
        * 1.2; // 20% margin

    let chart_resolution = chart
        .data
        .first()
        .map(|(x, _)| x)
        .and_then(|start| chart.data.last().map(|(x, _)| (start, x)))
        .map(|(start, end)| end - start)
        .map(|x_len| x_len / (chart_width as f64))
        .unwrap_or(f64::MIN);

    let min_dist = min_dist.max(chart_resolution);

    let splits = chart
        .data
        .iter()
        .map(|(x, _)| x)
        .enumerate()
        .rev()
        .tuple_windows()
        .filter(move |((_, b), (_, a))| **b - **a > min_dist)
        .map(|((ib, _), _)| ib);

    let mut data = chart.data;
    splits.chain(iter::once(0)).map(move |mid| {
        let (left, line) = data.split_at(mid);
        data = left;
        line
    })
}

#[cfg(test)]
mod test {
    use protocol::{large_bedroom, Device};

    use super::*;

    fn test_data() -> Vec<(f64, f64)> {
        let mut data = Vec::new();
        for segment in 0..5 {
            let x = ((segment * 100)..(segment * 100 + 5)).into_iter();
            data.extend(x.map(|x| (x as f64, 5.0)));
        }
        data
    }

    #[test]
    fn test_split() {
        let test_data = test_data();
        let test_chart = ChartParts {
            reading: protocol::reading::Info {
                val: 0.0,
                device: Device::LargeBedroom(large_bedroom::Device::Bed(
                    large_bedroom::bed::Device::Bme680,
                )),
                resolution: 1.0,
                range: 0.0..200.0,
                unit: protocol::Unit::C,
                description: "test",
                branch_id: 1,
            },
            data: &test_data,
        };
        assert_eq!(split(&test_chart, 500).count(), 5);
        assert_eq!(
            split(&test_chart, 500).next(),
            Some(
                [
                    (400.0, 5.0),
                    (401.0, 5.0),
                    (402.0, 5.0),
                    (403.0, 5.0),
                    (404.0, 5.0)
                ]
                .as_slice()
            )
        );
    }
}
