use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn build_structure_by_priority(entries: Vec<Message>) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    let mut stack: Vec<Message> = Vec::new();

    for entry in entries {
        // Remove entries from stack with priority >= current entry
        while !stack.is_empty() && stack.last().unwrap().indent_level >= entry.indent_level {
            stack.pop();
        }

        // If stack is not empty, add current entry as child of top entry
        if !stack.is_empty() {
            if let Some(last) = stack.last_mut() {
                last.message.push_str("\n");
                last.message.push_str(&entry.message);
            }
        } else {
            // If stack is empty, this is top level so add to result list
            result.push(entry.clone());
        }

        // Add current entry to stack
        stack.push(entry);
    }

    result
}

fn reduce_dict(
    result_dict: Vec<HashMap<String, serde_json::Value>>,
) -> HashMap<String, serde_json::Value> {
    let mut merged_dict = HashMap::new();
    for d in result_dict {
        for (key, value) in d {
            if key == "note" {
                let notes = merged_dict
                    .entry(key)
                    .or_insert_with(|| serde_json::Value::Array(vec![]));
                if let serde_json::Value::Array(arr) = notes {
                    arr.push(value);
                }
            } else {
                merged_dict.insert(key, value);
            }
        }
    }
    merged_dict
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub indent_level: i32,
    pub message: String,
}

impl Message {
    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "message".to_string(),
            serde_json::Value::String(self.message.clone()),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entry {
    Section(SectionEntry),
    SectionHeader(SectionHeader),
    KeyValue(KeyValueEntry),
    Message(MessageEntry),
}

impl Entry {
    fn priority(&self) -> i32 {
        match self {
            Entry::Message(m) => m.indent_level,
            _ => 0,
        }
    }

    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        match self {
            Entry::Section(s) => s.to_dict(),
            Entry::SectionHeader(h) => h.to_dict(),
            Entry::KeyValue(k) => k.to_dict(),
            Entry::Message(m) => m.to_dict(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionEntry {
    pub indent_level: i32,
    pub children: Vec<Entry>,
}

impl SectionEntry {
    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        let details = reduce_dict(self.children.iter().map(|child| child.to_dict()).collect());
        map.insert(
            "details".to_string(),
            serde_json::to_value(details).unwrap(),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionHeader {
    pub title: String,
}

impl SectionHeader {
    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "title".to_string(),
            serde_json::Value::String(self.title.clone()),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: Option<String>,
    pub value: Option<String>,
    pub indent_level: i32,
    pub children: Vec<Entry>,
}

impl KeyValueEntry {
    fn normalize_snakecase_key(&self) -> String {
        self.key
            .as_ref()
            .map(|k| {
                k.trim()
                    .to_lowercase()
                    .replace(".", "_")
                    .replace(" ", "_")
                    .replace("(", "")
                    .replace(")", "")
            })
            .unwrap_or_default()
    }

    fn parse_value(&self) -> serde_json::Value {
        match &self.value {
            None => serde_json::Value::Null,
            Some(v) => {
                let low_value = v.to_lowercase();
                if low_value == "yes" {
                    serde_json::Value::Bool(true)
                } else if low_value == "no" {
                    serde_json::Value::Bool(false)
                } else if low_value.chars().all(|c| c.is_digit(10)) {
                    serde_json::Value::Number(low_value.parse().unwrap())
                } else if let Ok(f) = low_value.parse::<f64>() {
                    serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
                } else {
                    serde_json::Value::String(v.clone())
                }
            }
        }
    }

    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        let key = self.normalize_snakecase_key();

        if !self.children.is_empty() {
            let mut inner_map = HashMap::new();
            if self.value.is_some() {
                inner_map.insert("value".to_string(), self.parse_value());
            }
            let details = reduce_dict(self.children.iter().map(|child| child.to_dict()).collect());
            inner_map.insert(
                "details".to_string(),
                serde_json::to_value(details).unwrap(),
            );
            map.insert(key, serde_json::to_value(inner_map).unwrap());
        } else {
            map.insert(key, self.parse_value());
        }
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    pub message: Option<String>,
    pub indent_level: i32,
    pub children: Vec<Entry>,
}

impl MessageEntry {
    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        if let Some(msg) = &self.message {
            map.insert("note".to_string(), serde_json::Value::String(msg.clone()));
        }
        if !self.children.is_empty() {
            let details = reduce_dict(self.children.iter().map(|child| child.to_dict()).collect());
            map.insert(
                "details".to_string(),
                serde_json::to_value(details).unwrap(),
            );
        }
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Section {
    Table(SectionTable),
    Note(SectionNote),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionTable {
    pub title: String,
    pub entries: Vec<Message>,
}

impl SectionTable {
    fn normalize_title(&self, title: &str) -> String {
        title
            .trim()
            .to_lowercase()
            .replace(" ", "_")
            .replace("(", "")
            .replace(")", "")
    }

    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        let entry_dicts: Vec<_> = self.entries.iter().map(|entry| entry.to_dict()).collect();

        let merged_dict = entry_dicts
            .into_iter()
            .fold(HashMap::new(), |mut acc, map| {
                acc.extend(map);
                acc
            });

        map.insert(
            self.normalize_title(&self.title),
            serde_json::to_value(merged_dict).unwrap(),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionNote {
    pub message: Message,
}

impl SectionNote {
    fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "note".to_string(),
            serde_json::Value::String(self.message.message.clone()),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub sections: Vec<Section>,
}

impl Document {
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        reduce_dict(
            self.sections
                .iter()
                .map(|section| match section {
                    Section::Table(t) => t.to_dict(),
                    Section::Note(n) => n.to_dict(),
                })
                .collect(),
        )
    }
}
