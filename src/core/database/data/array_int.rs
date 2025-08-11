// SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

use super::{Data, array::ArrayData, array_uninit::UninitArray};
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct IntArrayData {
    body: HashMap<usize, isize>,
}

impl Data for IntArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = "(".to_string();
        for i in self.keys() {
            formatted += &format!("[{}]=\"{}\" ", i, &self.body[&i]);
        }
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn clear(&mut self) {
        self.body.clear();
    }
    fn is_initialized(&self) -> bool {
        true
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        let n = self.to_index(key)?;
        Ok(self.body.contains_key(&n))
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        let n = super::to_int(value)?;
        self.body.insert(0, n);
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        let n = match value.parse::<isize>() {
            Ok(n) => n,
            Err(e) => return Err(ExecError::Other(e.to_string())),
        };

        if let Some(v) = self.body.get(&0) {
            self.body.insert(0, v + n);
        } else {
            self.body.insert(0, n);
        }
        Ok(())
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let key = self.to_index(key)?;
        let n = super::to_int(value)?;
        self.body.insert(key, n);
        Ok(())
    }

    // fn push_elems(&mut self, values: Vec<String>) -> Result<(), ExecError> {
    // let mut index = match self.body.is_empty() {
    // true  => 0,
    // false => *self.keys().iter().max().unwrap(),
    // };
    //
    // for v in values {
    // self.body.insert(index, v);
    // index += 1;
    // }
    // Ok(())
    // }

    fn append_to_array_elem(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let key = self.to_index(key)?;
        let n = super::to_int(value)?;

        if let Some(prev) = self.body.get(&key) {
            self.body.insert(key, prev + n);
        } else {
            self.body.insert(key, n);
        }
        Ok(())
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok(self.values().join(" "));
        }
        if key == "@" {
            return Ok(self.values().join(ifs));
        }

        let n = key.parse::<usize>().map_err(|_| ExecError::ArrayIndexInvalid(key.to_string()))?;

        Ok(self.body.get(&n).unwrap_or(&0).to_string())
    }

    fn get_all_as_array(&mut self, skip_none: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap() as usize;
        let mut ans = vec![];
        for i in 0..(max + 1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.to_string()),
                None => {
                    if !skip_none {
                        ans.push("".to_string());
                    }
                },
            }
        }
        Ok(ans)
    }

    fn get_vec_from(&mut self, pos: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap() as usize;
        let mut ans = vec![];
        for i in pos..(max + 1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.to_string()),
                None => {
                    if !skip_non {
                        ans.push("".to_string());
                    }
                },
            }
        }
        Ok(ans)
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().iter().map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        self.body.get(&0).map(|v| Ok(v.to_string())).ok_or(ExecError::Other("No entry".to_string()))?
    }

    fn get_str_type(&self) -> Box<dyn Data> {
        let mut hash = HashMap::new();
        for d in &self.body {
            hash.insert(*d.0, d.1.to_string());
        }

        Box::new(ArrayData::from(hash))
    }

    fn is_array(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        self.body.len()
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = key.parse::<usize>().map_err(|_| ExecError::ArrayIndexInvalid(key.to_string()))?;
        let s = self.body.get(&n).unwrap_or(&0).to_string();

        Ok(s.chars().count())
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            self.body.clear();
            return Ok(());
        }

        if let Ok(n) = key.parse::<usize>() {
            self.body.remove(&n);
            return Ok(());
        }
        Err(ExecError::Other("invalid index".to_string()))
    }
}

impl IntArrayData {
    pub fn set_elem(
        db_layer: &mut HashMap<String, Box<dyn Data>>,
        name: &str,
        pos: isize,
        val: &String,
    ) -> Result<(), ExecError> {
        if let Some(d) = db_layer.get_mut(name) {
            if d.is_array() {
                if !d.is_initialized() {
                    *d = IntArrayData::default().boxed_clone();
                }

                return d.set_as_array(&pos.to_string(), val);
            } else if d.is_assoc() {
                return d.set_as_assoc(&pos.to_string(), val);
            } else if d.is_single() {
                let data = d.get_as_single()?;
                IntArrayData::set_new_entry(db_layer, name)?;

                if data != "" {
                    Self::set_elem(db_layer, name, 0, &data)?;
                }
                Self::set_elem(db_layer, name, pos, val)
            } else {
                IntArrayData::set_new_entry(db_layer, name)?;
                Self::set_elem(db_layer, name, pos, val)
            }
        } else {
            IntArrayData::set_new_entry(db_layer, name)?;
            Self::set_elem(db_layer, name, pos, val)
        }
    }

    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str) -> Result<(), ExecError> {
        db_layer.insert(name.to_string(), UninitArray {}.boxed_clone());
        Ok(())
    }

    pub fn values(&self) -> Vec<String> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys.iter().map(|i| self.body[i].to_string()).collect()
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys
    }

    fn to_index(&mut self, key: &str) -> Result<usize, ExecError> {
        let mut index = match key.parse::<isize>() {
            Ok(i) => i,
            _ => return Err(ExecError::ArrayIndexInvalid(key.to_string())),
        };

        if index >= 0 {
            return Ok(index as usize);
        }

        let keys = self.keys();
        let max = match keys.iter().max() {
            Some(n) => *n as isize,
            None => -1,
        };
        index += max + 1;

        if index < 0 {
            return Err(ExecError::ArrayIndexInvalid(key.to_string()));
        }

        Ok(index as usize)
    }
}
