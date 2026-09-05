use std::{collections::VecDeque, sync::Arc};

use chrono::Local;
use iced::widget::text_editor::{self, Action, Cursor, Edit, Motion, Position};

const MAX_LINES: usize = 2_000;
const RETAIN_LINES_AFTER_TRIM: usize = 1_500;

#[derive(Clone)]
struct Entry {
    timestamp: String,
    text: String,
}

pub(super) struct SerialLog {
    entries: VecDeque<Entry>,
    content: text_editor::Content,
    show_timestamps: bool,
    pending_text: String,
}

impl SerialLog {
    pub(super) fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            content: text_editor::Content::new(),
            show_timestamps: true,
            pending_text: String::new(),
        }
    }

    pub(super) fn push(&mut self, text: String) {
        let entry = Entry {
            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
            text,
        };
        format_entry(&entry, self.show_timestamps, &mut self.pending_text);
        self.entries.push_back(entry);
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.content = text_editor::Content::new();
        self.pending_text.clear();
    }

    pub(super) fn set_show_timestamps(&mut self, enabled: bool) {
        if self.show_timestamps != enabled {
            self.show_timestamps = enabled;
            self.refresh_content();
        }
    }

    pub(super) fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    pub(super) fn content(&self) -> &text_editor::Content {
        &self.content
    }

    pub(super) fn perform(&mut self, action: Action) {
        self.content.perform(action);
    }

    pub(super) fn flush(&mut self) -> bool {
        if self.pending_text.is_empty() && self.entries.len() <= MAX_LINES {
            return false;
        }

        let mut cursor = self.content.cursor();
        if !self.pending_text.is_empty() {
            self.content.perform(Action::Move(Motion::DocumentEnd));
            self.content
                .perform(Action::Edit(Edit::Paste(Arc::new(std::mem::take(
                    &mut self.pending_text,
                )))));
        }

        if self.entries.len() > MAX_LINES {
            let removed = self.entries.len() - RETAIN_LINES_AFTER_TRIM;
            self.entries.drain(..removed);
            cursor = cursor_after_trim(cursor, removed);
            self.content.move_to(Cursor {
                position: Position {
                    line: removed,
                    column: 0,
                },
                selection: Some(Position { line: 0, column: 0 }),
            });
            self.content.perform(Action::Edit(Edit::Delete));
        }

        self.content.move_to(cursor);
        true
    }

    fn refresh_content(&mut self) {
        let mut text = String::new();
        for entry in &self.entries {
            format_entry(entry, self.show_timestamps, &mut text);
        }
        self.content = text_editor::Content::with_text(&text);
        self.pending_text.clear();
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub(super) fn last_text(&self) -> Option<&str> {
        self.entries.back().map(|entry| entry.text.as_str())
    }
}

fn cursor_after_trim(cursor: Cursor, removed: usize) -> Cursor {
    Cursor {
        position: position_after_trim(cursor.position, removed),
        selection: cursor
            .selection
            .map(|position| position_after_trim(position, removed)),
    }
}

fn position_after_trim(position: Position, removed: usize) -> Position {
    if position.line < removed {
        Position { line: 0, column: 0 }
    } else {
        Position {
            line: position.line - removed,
            column: position.column,
        }
    }
}

fn format_entry(entry: &Entry, show_timestamps: bool, output: &mut String) {
    if show_timestamps {
        output.push('[');
        output.push_str(&entry.timestamp);
        output.push_str("] ");
    }
    output.push_str(&entry.text);
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_appends_until_flush() {
        let mut log = SerialLog::new();
        for index in 0..200 {
            log.push(format!("RX {index}"));
        }

        assert_eq!(log.content().text(), "");
        assert!(log.flush());
        assert_eq!(log.content().line_count(), 201);
        assert!(log.content().text().contains("RX 199"));
    }

    #[test]
    fn trims_presented_prefix_without_rebuilding_content() {
        let mut log = SerialLog::new();
        for index in 0..MAX_LINES {
            log.push(format!("RX {index}"));
        }
        assert!(log.flush());

        log.content.move_to(Cursor {
            position: Position {
                line: 1_000,
                column: 2,
            },
            selection: Some(Position {
                line: 1_001,
                column: 3,
            }),
        });

        log.push(format!("RX {MAX_LINES}"));
        assert_eq!(log.entries.len(), MAX_LINES + 1);
        assert!(log.flush());
        assert_eq!(log.entries.len(), RETAIN_LINES_AFTER_TRIM);

        let content = log.content().text();
        assert_eq!(log.content().line_count(), RETAIN_LINES_AFTER_TRIM + 1);
        assert!(content.contains("RX 2000"));
        assert!(!content.contains("RX 0\n"));
        assert_eq!(
            log.content().cursor(),
            Cursor {
                position: Position {
                    line: 499,
                    column: 2,
                },
                selection: Some(Position {
                    line: 500,
                    column: 3,
                }),
            }
        );
    }

    #[test]
    fn sustained_traffic_stays_bounded() {
        let mut log = SerialLog::new();
        for index in 0..10_000 {
            log.push(format!("RX {index}"));
            if index % 256 == 255 {
                log.flush();
            }
        }
        log.flush();

        assert!(log.entries.len() <= MAX_LINES);
        assert!(log.content().line_count() <= MAX_LINES + 1);
        let content = log.content().text();
        assert!(content.contains("RX 9999"));
        assert!(!content.contains("RX 0\n"));
    }

    #[test]
    fn timestamp_toggle_and_clear_refresh_explicitly() {
        let mut log = SerialLog::new();
        log.push("RX one".into());
        log.push("RX two".into());
        log.flush();

        log.set_show_timestamps(false);
        assert_eq!(log.content().text(), "RX one\nRX two\n");
        log.set_show_timestamps(true);
        assert!(log.content().text().starts_with('['));

        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.content().text(), "");
    }
}
