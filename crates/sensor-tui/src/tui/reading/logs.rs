use std::iter;

use jiff::Unit;
use log_store::api::{self, ErrorEvent};

#[derive(Debug)]
pub(crate) struct Logs {
    current: Option<ErrorEvent>,
    /// newest items at the end
    history: Vec<api::ErrorEvent>,
}

impl Logs {
    pub(crate) fn new_from(error: &protocol::Error) -> Self {
        Self {
            current: Some(ErrorEvent {
                start: jiff::Timestamp::now(),
                end: None,
                error: error.clone(),
            }),
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
        if let Some(ErrorEvent { start, error, .. }) = self.current.take() {
            self.history.push(api::ErrorEvent {
                start,
                end: Some(jiff::Timestamp::now()),
                error,
            })
        }

        self.current = Some(ErrorEvent {
            start: jiff::Timestamp::now(),
            end: None,
            error: error.clone(),
        })
    }

    pub(crate) fn density<const N: usize>(&self, buckets: [jiff::Span; N]) -> [f32; N] {
        let now = jiff::Timestamp::now();
        let mut buckets = buckets.map(|bound| (bound, 0.0));
        for ErrorEvent { start, end, .. } in self.history.iter().rev() {
            for (bound, count) in &mut buckets {
                let bound_start = now - *bound;
                let start = start.max(&bound_start);
                let end = if let Some(end) = end { end } else { &now };
                let error_time = end.since(*start).expect("duration should fit type");
                if !error_time.is_negative() {
                    *count += error_time.total(Unit::Second).expect("smaller then day")
                        / bound.total(Unit::Second).expect("smaller then day");
                }
            }
        }
        buckets.map(|(_, count)| count as f32)
    }

    pub(crate) fn list(&self) -> impl Iterator<Item = &api::ErrorEvent> {
        self.history.iter().chain(self.current.iter())
    }
}
