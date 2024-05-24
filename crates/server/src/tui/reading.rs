use histogram::Histogram;
use protocol::{
    large_bedroom::{Error, LargeBedroomDiscriminants},
    Sensor,
};
use ratatui::{text::Line, widgets::Bar};
use std::{collections::VecDeque, time::Instant};

pub struct SensorInfo {
    timing: Histogram,
    history: VecDeque<(Instant, f32)>,
    condition: Result<(), Error>,
}

impl SensorInfo {
    fn last(&self) -> Result<f32, Error> {
        self.condition.clone()?;

        let last = self
            .history
            .front()
            .expect("Items are put in the map when they arrive with a value");
        Ok(last.1)
    }

    fn last_at(&self) -> Result<Instant, Error> {
        self.condition.clone()?;

        let last = self
            .history
            .front()
            .expect("Items are put in the map when they arrive with a value");
        Ok(last.0)
    }
}

pub struct Readings {
    pub map: Vec<(LargeBedroomDiscriminants, SensorInfo)>,
}

impl Readings {
    pub fn add(&mut self, data: Sensor) {
        match data {
            Sensor::LargeBedroom(data) => {
                let key = LargeBedroomDiscriminants::from(data);
                if let Some((_, info)) = self.map.iter_mut().find(|p| p.0 == key) {
                    if let Ok(last_reading) = info.last_at() {
                        info.timing
                            .increment(last_reading.elapsed().as_millis() as u64)
                            .unwrap();
                    }
                    info.history.push_front((Instant::now(), data.into()));
                    info.condition = Ok(());
                } else {
                    let mut history = VecDeque::new();
                    history.push_front((Instant::now(), data.into()));
                    self.map.push((
                        key,
                        SensorInfo {
                            timing: Histogram::new(4, 24).unwrap(),
                            history,
                            condition: Ok(()),
                        },
                    ));
                }
            }
            Sensor::LargeBedroomError(err) => {
                for key in err.broken_readings() {
                    if let Some((_, info)) = self.map.iter_mut().find(|p| p.0 == *key) {
                        info.condition = Err(err.clone());
                    } else {
                        self.map.push((
                            *key,
                            SensorInfo {
                                timing: Histogram::new(4, 24).unwrap(),
                                history: VecDeque::new(),
                                condition: Err(err.clone()),
                            },
                        ))
                    }
                }
            }
            _ => todo!(),
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.map
            .iter()
            .map(|(key, val)| {
                let val = val.last();
                format!("{key:?}: {val:?}\n")
            })
            .collect()
    }

    pub fn histogram_all(&self) -> Vec<Bar> {
        let mut all = Histogram::new(4, 24).unwrap();
        for (_, val) in self.map.iter() {
            all = all.checked_add(&val.timing).unwrap();
        }
        histogram_bars(&all)
    }

    pub fn histogram(&self, key: LargeBedroomDiscriminants) -> Vec<Bar> {
        let hist = &self.map.iter().find(|p| p.0 == key).unwrap().1.timing;
        histogram_bars(hist)
    }

    pub fn chart<'a>(
        &mut self,
        key: LargeBedroomDiscriminants,
        plot_buf: &'a mut Vec<(f64, f64)>,
    ) -> Option<ChartParts<'a>> {
        let data = &self.map.iter_mut().find(|p| p.0 == key).unwrap().1;

        plot_buf.clear();
        for xy in data
            .history
            .iter()
            .map(|(x, y)| (x.elapsed().as_secs_f64(), *y as f64))
        {
            plot_buf.push(xy);
        }

        Some(ChartParts {
            name: format!("{key:?}"),
            data: plot_buf,
        })
    }
}

pub struct ChartParts<'a> {
    pub name: String,
    pub data: &'a [(f64, f64)],
}

fn histogram_bars(hist: &Histogram) -> Vec<Bar<'static>> {
    let percentiles = hist.percentiles(&[25.0, 50.0, 75.0, 90.0, 95.0, 100.0]).unwrap();
    percentiles
        .into_iter()
        .map(|(p, bucket)| {
            Bar::default()
                .value(bucket.count())
                .text_value(format!("p{p}: {}", bucket.count()))
                .label(Line::from(format!("{}..{}", bucket.start(), bucket.end())))
        })
        .collect()
}
