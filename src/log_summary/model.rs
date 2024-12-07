use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RcSectionEntry {
    pub indent_level: i32,
    pub message: String,
    pub children: Vec<Rc<RefCell<RcSectionEntry>>>,
}

// Start Generation Here
pub fn convert_rc_section_entries(entries: Vec<Rc<RefCell<RcSectionEntry>>>) -> Vec<SectionEntry> {
    entries
        .into_iter()
        .map(|rc_entry| {
            let RcSectionEntry {
                indent_level,
                message,
                children,
            } = Rc::try_unwrap(rc_entry)
                .expect("Multiple references exist")
                .into_inner();

            SectionEntry {
                indent_level,
                message,
                children: convert_rc_section_entries(children),
            }
        })
        .collect()
}

// Start of Selection
pub fn build_structure_by_priority(entries: Vec<Message>) -> Vec<SectionEntry> {
    let mut result: Vec<Rc<RefCell<RcSectionEntry>>> = Vec::new();
    let mut stack: Vec<Rc<RefCell<RcSectionEntry>>> = Vec::new();

    for entry in entries {
        // 현재 엔트리보다 들여쓰기가 크거나 같은 스택의 엔트리들 제거
        while let Some(last_entry) = stack.last() {
            if last_entry.borrow().indent_level >= entry.indent_level {
                stack.pop();
            } else {
                break;
            }
        }

        let section_entry = Rc::new(RefCell::new(RcSectionEntry {
            indent_level: entry.indent_level,
            message: entry.message.clone(),
            children: Vec::new(),
        }));

        // 스택이 비어있지 않으면 현재 엔트리를 스택 최상위 엔트리의 자식으로 추가
        if let Some(parent) = stack.last() {
            // 자식으로 추가하기 위해 Rc 복제
            parent.borrow_mut().children.push(Rc::clone(&section_entry));
        } else {
            // 스택이 비어있으면 최상위 레벨이므로 결과 목록에 추가
            result.push(Rc::clone(&section_entry));
        }

        // 현재 엔트리를 스택에 추가
        stack.push(Rc::clone(&section_entry));
    }

    stack.clear(); // clear all references
    convert_rc_section_entries(result)
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
    pub message: String,
    pub children: Vec<SectionEntry>,
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
    pub entries: Vec<SectionEntry>,
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
