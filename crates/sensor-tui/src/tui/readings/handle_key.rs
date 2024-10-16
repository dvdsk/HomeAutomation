use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::UiState;

impl UiState {
    pub(crate) fn handle_key_normal_mode(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('b') if self.reading_selected => {
                self.history_length.start_editing();
                self.input_mode.editing_bounds = true;
            }
            KeyCode::Char('h') if self.reading_selected => {
                self.show_histogram = !self.show_histogram;
            }
            KeyCode::Char('l') if self.reading_selected => {
                self.show_logs = !self.show_logs;
            }
            KeyCode::Char('x') if self.reading_selected => {
                self.chart_cursor.toggle();
                self.input_mode.chart_cursor = self.chart_cursor.is_enabled();
            }
            KeyCode::Char('c') if self.reading_selected => {
                let id = *self
                    .tree_state
                    .selected()
                    .last()
                    .expect("reading_selected is true");
                if !self.comparing.remove(&id) {
                    self.comparing.insert(id);
                }
            }
            KeyCode::Char('?') => {
                self.show_complete_help = !self.show_complete_help;
            }
            _ => return Some(key),
        }

        None
    }

    pub(crate) fn handle_key_cursor(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        let offset = match key.code {
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => -5,
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => 5,
            KeyCode::Left => -1,
            KeyCode::Right => 1,
            _ => return Some(key),
        };

        self.chart_cursor.shift(offset);
        None
    }

    pub(crate) fn handle_key_bounds(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.history_length.exit_editing();
                self.input_mode.editing_bounds = false;
                None
            }
            _ => self.history_length.process(key),
        }
    }

    pub(crate) fn handle_key_all(&mut self, key: KeyEvent) -> Option<KeyEvent> {
        match key.code {
            KeyCode::Down => {
                self.tree_state.key_down();
            }
            KeyCode::Up => {
                self.tree_state.key_up();
            }
            KeyCode::Enter => {
                self.tree_state.toggle_selected();
            }
            _ => return Some(key),
        }
        None
    }
}

