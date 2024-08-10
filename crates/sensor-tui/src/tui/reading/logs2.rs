use jiff::Unit;
use log_store::api::{self, ErrorEvent};

#[derive(Debug)]
pub(crate) struct Logs {
    current: Option<(jiff::Timestamp, protocol::Error)>,
    history: Vec<api::ErrorEvent>,
}

impl Logs {
    pub(crate) fn new_from(error: &protocol::Error) -> Self {
        Self {
            current: Some((jiff::Timestamp::now(), error.clone())),
            history: Vec::new(),
        }
    }

    pub(crate) fn new_empty() -> Self {
        Self {
            current: None,
            history: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, error: &protocol::Error) {
        if let Some((started, event)) = self.current.take() {
            self.history.push(api::ErrorEvent {
                start: started,
                end: jiff::Timestamp::now(),
                error: event,
            })
        }

        self.current = Some((jiff::Timestamp::now(), error.clone()))
    }

    pub(crate) fn density<const N: usize>(&self, buckets: [jiff::Span; N]) -> [f32; N] {
        let now = jiff::Timestamp::now();
        let mut buckets = buckets.map(|bound| (bound, 0.0));
        for ErrorEvent { start, end, .. } in self.history.iter().rev() {
            for (bound, count) in &mut buckets {
                let bound_start = now - *bound;
                let start = start.max(&bound_start);
                let error_time = end.since(*start).expect("duration should fit type");
                if !error_time.is_negative() {
                    *count += error_time.total(Unit::Second).expect("smaller then day")
                        / bound.total(Unit::Second).expect("smaller then day");
                }
            }
        }
        buckets.map(|(_, count)| count as f32)
    }
}
