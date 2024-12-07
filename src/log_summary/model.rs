use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RcSectionEntry {
    pub message: Message,
    pub children: Vec<Rc<RefCell<RcSectionEntry>>>,
}

impl From<RcSectionEntry> for SectionEntry {
    fn from(entry: RcSectionEntry) -> Self {
        SectionEntry {
            entry: Entry::from(entry.message),
            children: RcSectionEntryVec(entry.children).into(),
        }
    }
}

// 새로운 타입 정의
pub struct RcSectionEntryVec(Vec<Rc<RefCell<RcSectionEntry>>>);

impl From<RcSectionEntryVec> for Vec<SectionEntry> {
    fn from(entries: RcSectionEntryVec) -> Self {
        entries
            .0
            .into_iter()
            .map(|rc_entry| {
                Rc::try_unwrap(rc_entry)
                    .expect("Multiple references exist")
                    .into_inner()
                    .into()
            })
            .collect()
    }
}

// Start of Selection
pub fn build_structure_by_priority(entries: Vec<Message>) -> Vec<SectionEntry> {
    let mut result: Vec<Rc<RefCell<RcSectionEntry>>> = Vec::new();
    let mut stack: Vec<Rc<RefCell<RcSectionEntry>>> = Vec::new();

    for entry in entries {
        // 현재 엔트리보다 들여쓰기가 크거나 같은 스택의 엔트리들 제거
        while let Some(last_entry) = stack.last() {
            if last_entry.borrow().message.indent_level >= entry.indent_level {
                stack.pop();
            } else {
                break;
            }
        }

        let section_entry = Rc::new(RefCell::new(RcSectionEntry {
            message: entry,
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
    RcSectionEntryVec(result).into()
}

fn reduce_dict(
    result_dict: Vec<HashMap<String, serde_value::Value>>,
) -> HashMap<String, serde_value::Value> {
    let mut merged_dict = HashMap::new();
    for d in result_dict {
        for (key, value) in d {
            if key == "note" {
                let notes = merged_dict.entry(key).or_insert_with(|| Value::Seq(vec![]));
                if let Value::Seq(arr) = notes {
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
    fn to_dict(&self) -> HashMap<String, serde_value::Value> {
        let mut map = HashMap::new();
        map.insert(
            "message".to_string(),
            serde_value::Value::String(self.message.clone()),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entry {
    KeyValue(KeyValueEntry),
    Message(MessageEntry),
}

impl Entry {
    fn priority(&self) -> i32 {
        match self {
            Entry::Message(m) => m.indent_level,
            Entry::KeyValue(k) => k.indent_level,
        }
    }

    fn to_dict(
        &self,
        details: Option<HashMap<String, serde_value::Value>>,
    ) -> HashMap<String, serde_value::Value> {
        match self {
            Entry::KeyValue(k) => k.to_dict(details),
            Entry::Message(m) => m.to_dict(details),
        }
    }
}

impl From<Message> for Entry {
    fn from(msg: Message) -> Self {
        // key:value 형식인지 확인
        if msg.message.contains(':') {
            let parts: Vec<&str> = msg.message.splitn(2, ':').collect();
            let key = parts[0].trim();
            let entry = if parts.len() == 2 {
                let value = parts[1].trim();
                // 유효한 key:value 쌍이면 KeyValue 반환
                Entry::KeyValue(KeyValueEntry {
                    indent_level: msg.indent_level,
                    key: key.to_string(),
                    value: Some(value.to_string()),
                })
            } else {
                Entry::KeyValue(KeyValueEntry {
                    indent_level: msg.indent_level,
                    key: key.to_string(),
                    value: None,
                })
            };
            return entry;
        }

        // key:value 형식이 아니면 Message 반환
        Entry::Message(MessageEntry {
            indent_level: msg.indent_level,
            message: msg.message,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionEntry {
    pub entry: Entry,
    pub children: Vec<SectionEntry>,
}

impl SectionEntry {
    fn to_dict(&self) -> HashMap<String, serde_value::Value> {
        if self.children.len() > 0 {
            let details = reduce_dict(
                self.children
                    .iter()
                    .map(|child: &SectionEntry| child.to_dict())
                    .collect(),
            );
            self.entry.to_dict(Some(details))
        } else {
            self.entry.to_dict(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: Option<String>,
    pub indent_level: i32,
}

impl KeyValueEntry {
    fn normalize_snakecase_key(&self) -> String {
        self.key
            .trim()
            .to_lowercase()
            .replace(".", "_")
            .replace(" ", "_")
            .replace("(", "")
            .replace(")", "")
    }

    fn parse_value(&self) -> serde_value::Value {
        debug!("         value: {:?}", self.value);
        match &self.value {
            None => serde_value::to_value(None::<String>).unwrap(),
            Some(v) => {
                let low_value = v.to_lowercase();
                if low_value == "yes" {
                    Value::Bool(true)
                } else if low_value == "no" {
                    Value::Bool(false)
                } else if let Ok(num) = low_value.parse::<i64>() {
                    Value::I64(num)
                } else if let Ok(f) = low_value.parse::<f64>() {
                    Value::F64(f)
                } else {
                    Value::String(v.clone())
                }
            }
        }
    }

    fn to_dict(
        &self,
        details: Option<HashMap<String, serde_value::Value>>,
    ) -> HashMap<String, serde_value::Value> {
        let key = self.normalize_snakecase_key();
        let value = self.parse_value();
        if let Some(details) = details {
            let value_map: HashMap<String, Value> = HashMap::from([
                ("value".to_string(), value),
                (
                    "details".to_string(),
                    serde_value::to_value(details).unwrap(),
                ),
            ]);

            // {"key": {"value": value, "details": details}}
            HashMap::from([(key, serde_value::to_value(value_map).unwrap())])
        } else {
            HashMap::from([(key, value)])
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    pub message: String,
    pub indent_level: i32,
}

impl MessageEntry {
    fn to_dict(
        &self,
        details: Option<HashMap<String, serde_value::Value>>,
    ) -> HashMap<String, serde_value::Value> {
        if let Some(details) = details {
            // {"note": {"value": "message", "details": details}}
            HashMap::from([(
                "note".to_string(),
                serde_value::to_value(HashMap::from([
                    ("value".to_string(), Value::String(self.message.clone())),
                    (
                        "details".to_string(),
                        serde_value::to_value(details).unwrap(),
                    ),
                ]))
                .unwrap(),
            )])
        } else {
            // {"note": "message"}
            HashMap::from([("note".to_string(), Value::String(self.message.clone()))])
        }
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

    fn to_dict(&self) -> HashMap<String, serde_value::Value> {
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
            serde_value::to_value(merged_dict).unwrap(),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionNote {
    pub message: Message,
}

impl SectionNote {
    fn to_dict(&self) -> HashMap<String, serde_value::Value> {
        let mut map = HashMap::new();
        map.insert(
            "note".to_string(),
            serde_value::Value::String(self.message.message.clone()),
        );
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub sections: Vec<Section>,
}

impl Document {
    pub fn to_dict(&self) -> HashMap<String, serde_value::Value> {
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
