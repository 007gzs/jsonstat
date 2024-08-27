use std::{collections::BTreeMap, fmt::Display};

use serde::Serialize;
use serde_json::{json, Map, Value};

#[derive(Serialize, Clone)]
struct MaxMinCount<T> {
    count: usize,
    max: T,
    min: T,
}
impl<T: PartialOrd + Default + Clone> MaxMinCount<T> {
    fn new() -> Self {
        Self {
            count: 0,
            max: T::default(),
            min: T::default(),
        }
    }
    fn add(&mut self, new_value: &T) {
        if self.count == 0 {
            self.max = new_value.clone();
            self.min = new_value.clone();
        } else {
            if new_value > &self.max {
                self.max = new_value.clone();
            }
            if new_value < &self.min {
                self.min = new_value.clone();
            }
        }
        self.count += 1;
    }
    fn merge(&mut self, other: &Self) {
        if self.count == 0 {
            self.max = other.max.clone();
            self.min = other.min.clone();
        } else {
            if other.max > self.max {
                self.max = other.max.clone();
            }
            if other.min < self.min {
                self.min = other.min.clone();
            }
        }
        self.count += other.count;
    }
}

#[derive(Serialize, Clone)]
struct Count {
    count: usize,
}
impl Count {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn add(&mut self) {
        self.count += 1;
    }
    fn merge(&mut self, other: &Self) {
        self.count += other.count;
    }
}

#[derive(Serialize, Clone)]
struct JsonStatItem {
    string: MaxMinCount<usize>,
    int: MaxMinCount<i64>,
    float: MaxMinCount<f64>,
    bool: Count,
    null: Count,
    object: Count,
    array: MaxMinCount<usize>,
}
impl JsonStatItem {
    fn new() -> Self {
        JsonStatItem {
            string: MaxMinCount::new(),
            int: MaxMinCount::new(),
            float: MaxMinCount::new(),
            bool: Count::new(),
            null: Count::new(),
            object: Count::new(),
            array: MaxMinCount::new(),
        }
    }
    fn merge(&mut self, other: &Self) {
        self.string.merge(&other.string);
        self.int.merge(&other.int);
        self.float.merge(&other.float);
        self.bool.merge(&other.bool);
        self.null.merge(&other.null);
        self.object.merge(&other.object);
        self.array.merge(&other.array);
    }
    fn stat(&mut self, key: &str, data: &Value) -> Vec<(String, Value)> {
        let mut ret = Vec::new();
        match data {
            Value::String(s) => {
                self.string.add(&s.len());
            }
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    self.int.add(&num);
                } else if let Some(num) = n.as_f64() {
                    self.float.add(&num);
                }
            }
            Value::Null => {
                self.null.add();
            }
            Value::Bool(_) => {
                self.bool.add();
            }
            Value::Array(arr) => {
                self.array.add(&arr.len());
                let k = format!("{}[]", key);
                for item in arr {
                    ret.push((k.clone(), item.clone()));
                }
            }
            Value::Object(obj) => {
                self.object.add();
                for (k, v) in obj {
                    ret.push((format!("{}.{}", key, k), v.clone()));
                }
            }
        }
        ret
    }
    fn to_json_value(&self) -> Value {
        let mut ret = Map::new();
        if self.null.count > 0 {
            ret.insert("null".to_string(), json!({"count": self.null.count}));
        }
        if self.bool.count > 0 {
            ret.insert("bool".to_string(), json!({"count": self.bool.count}));
        }
        if self.int.count > 0 {
            ret.insert(
                "int".to_string(),
                json!({
                    "count": self.int.count,
                    "min": self.int.min,
                    "max": self.int.max,
                }),
            );
        }
        if self.float.count > 0 {
            ret.insert(
                "float".to_string(),
                json!({
                    "count": self.float.count,
                    "min": self.float.min,
                    "max": self.float.max,
                }),
            );
        }
        if self.string.count > 0 {
            ret.insert(
                "string".to_string(),
                json!({
                    "count": self.string.count,
                    "min": self.string.min,
                    "max": self.string.max,
                }),
            );
        }
        if self.array.count > 0 {
            ret.insert(
                "array".to_string(),
                json!({
                    "count": self.array.count,
                    "min": self.array.min,
                    "max": self.array.max,
                }),
            );
        }
        if self.object.count > 0 {
            ret.insert("object".to_string(), json!({"count": self.object.count}));
        }
        Value::Object(ret)
    }
}

impl Display for JsonStatItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.null.count > 0 {
            write!(f, "null:{};", self.null.count)?;
        }
        if self.bool.count > 0 {
            write!(f, "bool:{};", self.bool.count)?;
        }
        if self.int.count > 0 {
            write!(
                f,
                "int:{}({}~{});",
                self.int.count, self.int.min, self.int.max
            )?;
        }
        if self.float.count > 0 {
            write!(
                f,
                "float:{}({}~{});",
                self.float.count, self.float.min, self.float.max
            )?;
        }
        if self.string.count > 0 {
            write!(
                f,
                "string:{}({}~{});",
                self.string.count, self.string.min, self.string.max
            )?;
        }
        if self.array.count > 0 {
            write!(
                f,
                "array:{}({}~{});",
                self.array.count, self.array.min, self.array.max
            )?;
        }
        if self.object.count > 0 {
            write!(f, "object:{}", self.object.count)?;
        }
        Ok(())
    }
}
pub struct JsonStat {
    items: BTreeMap<String, JsonStatItem>,
    group_key: Option<String>,
}
impl JsonStat {
    pub fn new() -> Self {
        JsonStat {
            items: BTreeMap::new(),
            group_key: None,
        }
    }
    pub fn new_by_group(group_key: &str) -> Self {
        JsonStat {
            items: BTreeMap::new(),
            group_key: Some(group_key.to_string()),
        }
    }
    pub fn stat_str(&mut self, line: &str) -> bool {
        if let Ok(value) = serde_json::from_str(line) {
            self.stat_value(&value)
        } else {
            false
        }
    }
    fn get_group_key(&self, value: &Value) -> String {
        if let Some(key) = &self.group_key {
            let mut val = value;
            for k in key.split('.') {
                val = match val.as_object() {
                    Some(v) => match v.get(k) {
                        Some(_v) => _v,
                        None => {
                            return "".to_string();
                        }
                    },
                    None => {
                        return "".to_string();
                    }
                }
            }
            if let Some(r) = val.as_str() {
                r.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
    pub fn stat_value(&mut self, value: &Value) -> bool {
        let mut todo_list = Vec::new();
        todo_list.push((self.get_group_key(value), value.clone()));
        while let Some((k, v)) = todo_list.pop() {
            let (item, list) = self.stat_key_value(&k, &v);
            if !list.is_empty() {
                todo_list.extend(list);
            }
            if let Some(v) = self.items.get_mut(&k) {
                v.merge(&item);
            } else {
                self.items.insert(k, item);
            }
        }
        true
    }
    fn stat_key_value(&self, key: &str, value: &Value) -> (JsonStatItem, Vec<(String, Value)>) {
        let mut item = JsonStatItem::new();
        let ret = item.stat(key, value);
        (item, ret)
    }
    pub fn merge(&mut self, other: &Self) {
        for (k, v) in other.items.iter() {
            if let Some(v1) = self.items.get_mut(k) {
                v1.merge(v);
            } else {
                self.items.insert(k.clone(), v.clone());
            }
        }
    }
    pub fn to_json_str(&self, full: bool) -> String {
        if full {
            serde_json::to_string(&self.items).unwrap()
        } else {
            let mut map = Map::new();
            for (k, v) in self.items.iter() {
                map.insert(k.clone(), v.to_json_value());
            }
            Value::Object(map).to_string()
        }
    }
}
impl Display for JsonStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.items.iter() {
            writeln!(f, "{} : {}", if k.is_empty() { "." } else { k }, v)?;
        }
        Ok(())
    }
}
impl Default for JsonStat {
    fn default() -> Self {
        Self::new()
    }
}
