// SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDX-License-Identifier: BSD-3-Clause

use super::subscript::Subscript;
use crate::{
    Feeder, ShellCore,
    error::{exec::ExecError, parse::ParseError},
    utils::arg,
};

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub text:   String,
    pub name:   String,
    pub index:  Option<Subscript>,
    pub lineno: usize,
}

impl Variable {
    pub fn get_index(
        &mut self,
        core: &mut ShellCore,
        right_is_array: bool,
        append: bool,
    ) -> Result<Option<String>, ExecError> {
        if let Some(mut s) = self.index.clone() {
            if s.text == "[]" {
                return Err(ExecError::ArrayIndexInvalid("".to_string()));
            }
            if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                if core.db.is_assoc(&self.name) {
                    let mut index = s.text.clone();
                    index.remove(0);
                    index.pop();
                    return Ok(Some(index));
                }
                return Ok(Some("0".to_string()));
            }
            let index = s.eval(core, &self.name)?;
            return Ok(Some(index));
        }

        if core.db.is_array(&self.name) && !append && !right_is_array { Ok(Some("0".to_string())) } else { Ok(None) }
    }

    pub fn is_array(&mut self) -> bool {
        self.is_pos_param_array() || self.is_var_array()
    }

    pub fn is_pos_param_array(&mut self) -> bool {
        self.name == "@" || self.name == "*"
    }

    pub fn is_var_array(&mut self) -> bool {
        if self.index.is_none() {
            return false;
        }
        let sub = &self.index.as_ref().unwrap().text;
        sub == "[*]" || sub == "[@]"
    }

    pub fn set_value(&mut self, value: &String, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.index.is_none() {
            return core.db.set_param(&self.name, value, None);
        }

        let index = self.index.clone().unwrap().eval(core, &self.name)?;
        core.db.set_param2(&self.name, &index, value, None)
    }

    pub fn parse_and_set(arg: &str, value: &str, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut f = Feeder::new(arg);
        match Self::parse(&mut f, core)? {
            Some(mut v) => {
                if !f.is_empty() {
                    return Err(ExecError::InvalidName(arg.to_string()));
                }
                v.set_value(&value.to_string(), core)
            },
            None => Err(ExecError::InvalidName(arg.to_string())),
        }
    }

    pub fn init_variable(
        &self,
        core: &mut ShellCore,
        layer: Option<usize>,
        args: &mut Vec<String>,
    ) -> Result<(), ExecError> {
        let mut prev = vec![];

        if (layer.is_none() && core.db.exist(&self.name)) || core.db.params[layer.unwrap()].get(&self.name).is_some() {
            prev = vec![core.db.get_param(&self.name)?];
        }

        let i_opt = arg::consume_option("-i", args);
        if arg::consume_option("-a", args) || (!args.contains(&"-A".to_string()) && self.index.is_some()) {
            if prev.is_empty() {
                // TODO: ^ Maybe, there is a case where an assoc must be
                // prepared.
                return match i_opt {
                    true => core.db.set_int_array(&self.name, None, layer),
                    false => core.db.set_array(&self.name, None, layer),
                };
            }

            return match i_opt {
                true => core.db.set_int_array(&self.name, Some(prev), layer),
                false => core.db.set_array(&self.name, Some(prev), layer),
            };
        } else if arg::consume_option("-A", args) {
            match i_opt {
                true => core.db.set_int_assoc(&self.name, layer)?,
                false => core.db.set_assoc(&self.name, layer, false)?,
            }
            if !prev.is_empty() {
                core.db.set_assoc_elem(&self.name, &"0".to_string(), &prev[0], layer)?;
            }
            return Ok(());
        }

        let value = match prev.len() {
            0 => "".to_string(),
            _ => prev[0].clone(),
        };

        match i_opt {
            true => core.db.init_as_num(&self.name, &value, layer),
            false => core.db.set_param(&self.name, &value, layer),
        }
    }

    pub fn exist(&self, core: &mut ShellCore) -> Result<bool, ExecError> {
        // used in value_check.rs
        if core.db.is_array(&self.name) || core.db.is_assoc(&self.name) {
            if core.db.get_vec(&self.name, false)?.is_empty() {
                return Ok(false);
            }

            if self.index.is_none() {
                return Ok(core.db.has_key(&self.name, "0")?);
            }
        }

        if let Some(sub) = self.index.clone().as_mut() {
            let index = sub.eval(core, &self.name)?;
            return Ok(core.db.has_key(&self.name, &index)?);
        }

        Ok(core.db.exist(&self.name))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        let name = feeder.consume(len);
        ans.name = name.clone();
        ans.text += &name;

        if let Some(s) = Subscript::parse(feeder, core)? {
            ans.text += &s.text.clone();
            ans.index = Some(s);
        };

        Ok(Some(ans))
    }
}
