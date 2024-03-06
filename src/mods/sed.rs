use regex::Regex;
use std::collections::VecDeque;

pub struct SedCommand {
    pattern: Regex,
    replacement: String,
    global: bool,
}

impl SedCommand {
    pub fn parse(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split('/').collect();
        if parts.len() >= 4 {
            let pattern = match Regex::new(parts[1]) {
                Ok(regex) => regex,
                Err(_) => return None,
            };
            let replacement = parts[2].to_string();
            let global = parts.get(3).map_or(false, |&flag| flag.contains('g'));

            Some(SedCommand {
                pattern,
                replacement,
                global,
            })
        } else {
            None
        }
    }

    pub fn apply_to(&self, message: &str) -> String {
        if self.global {
            self.pattern.replace_all(message, self.replacement.as_str()).to_string()
        } else {
            self.pattern.replace(message, self.replacement.as_str()).to_string()
        }
    }
}

pub struct MessageBuffer {
    buffer: VecDeque<String>,
    capacity: usize,
}

impl MessageBuffer {
    pub fn new(capacity: usize) -> Self {
        MessageBuffer {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_message(&mut self, message: String) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(message);
    }

    pub fn apply_sed_command(&mut self, command: &SedCommand) -> Option<String> {
        for message in self.buffer.iter_mut() {
            if command.pattern.is_match(message) {
                *message = command.apply_to(message);
                return Some(message.clone());
            }
        }
        None
    }
}

