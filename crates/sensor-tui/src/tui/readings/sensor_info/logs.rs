use jiff::Unit;
use log_store::api::{self, ErrorEvent};

#[derive(Debug)]
pub(crate) struct FromStore {
    pub(crate) list: Vec<api::ErrorEvent>,
    pub(crate) since: jiff::Timestamp,
}

#[derive(Debug)]
pub(crate) struct Local {
    pub(crate) list: Vec<api::ErrorEvent>,
    pub(crate) since: jiff::Timestamp,
}

#[derive(Debug)]
pub(crate) struct Logs {
    current: Option<ErrorEvent>,
    pub local: Local,
    pub from_store: Option<FromStore>,
}

impl Logs {
    pub(crate) fn new_from(error: &protocol::Error) -> Self {
        Self {
            current: Some(ErrorEvent {
                start: jiff::Timestamp::now(),
                end: None,
                error: error.clone(),
            }),
            local: Local {
                list: Vec::new(),
                since: jiff::Timestamp::now(),
            },
            from_store: None,
        }
    }

    pub(crate) fn new_empty() -> Self {
        Self {
            current: None,
            local: Local {
                list: Vec::new(),
                since: jiff::Timestamp::now(),
            },
            from_store: None,
        }
    }

    pub(crate) fn add(&mut self, new_error: &protocol::Error) {
        if let Some(ErrorEvent { start, error, .. }) = self.current.take() {
            if &error == new_error {
                return;
            }

            self.local.list.push(api::ErrorEvent {
                start,
                end: Some(jiff::Timestamp::now()),
                error,
            })
        }

        self.current = Some(ErrorEvent {
            start: jiff::Timestamp::now(),
            end: None,
            error: new_error.clone(),
        })
    }

    pub(crate) fn density<const N: usize>(&self, buckets: [jiff::Span; N]) -> [f32; N] {
        let now = jiff::Timestamp::now();
        let mut buckets = buckets.map(|bound| (bound, 0.0));
        for ErrorEvent { start, end, .. } in self.local.list.iter().rev() {
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

    pub(crate) fn list(&self) -> Vec<api::ErrorEvent> {
        let last = self
            .from_store
            .as_ref()
            .map(|FromStore { list, .. }| list)
            .map(|list| list.last())
            .flatten()
            .map(|ErrorEvent { start, .. }| *start)
            .unwrap_or(jiff::Timestamp::from_second(0).unwrap());
        let without_duplicates = self
            .local
            .list
            .iter()
            .skip_while(|ErrorEvent { start, .. }| start < &last)
            .cloned();
        let mut logs = self
            .from_store
            .as_ref()
            .map(|FromStore { list, .. }| list)
            .cloned()
            .unwrap_or(Vec::new());
        logs.extend(without_duplicates);
        logs
    }

    pub(crate) fn covers_from(&self) -> jiff::Timestamp {
        *self
            .from_store
            .as_ref()
            .map(|FromStore { since, .. }| since)
            .unwrap_or(&self.local.since)
    }
}
