use std::io::stdout;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use sensor_tui::tui::readings::render::chart;
use sensor_tui::tui::readings::sensor_info::ChartParts;
use sensor_tui::tui::readings::UiState;

fn data_with_gap(buffer: &mut Vec<(f64, f64)>) -> ChartParts<'_> {
    buffer.extend((0..10).into_iter().map(|i| (i as f64, 2.0)));
    buffer.extend((30..35).into_iter().map(|i| (i as f64, 5.0)));
    buffer.extend((50..55).into_iter().map(|i| (i as f64, 1.0)));

    let info = protocol::reading::Info {
        val: 0.0,
        device: protocol::Device::SmallBedroom(protocol::small_bedroom::Device::Bed(
            protocol::small_bedroom::bed::Device::Bme680,
        )),
        resolution: 1.0,
        range: 0.0..5.0,
        unit: protocol::Unit::Ohm,
        description: "test data with gaps",
        branch_id: 0,
    };

    ChartParts {
        reading: info,
        data: buffer.as_mut_slice(),
    }
}

fn chart(frame: &mut Frame) {
    let layout = frame.area();
    let mut tab = UiState::default();
    let mut buffer = Vec::new();
    chart::render(frame, layout, &mut tab, &mut [data_with_gap(&mut buffer)]);
}

fn main() {
    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    terminal.clear().unwrap();

    terminal.draw(chart).unwrap();
    event::read().unwrap();

    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
