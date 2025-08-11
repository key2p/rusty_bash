// SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDX-License-Identifier: BSD-3-Clause

use crate::{
    Feeder, ShellCore,
    elements::{expr::arithmetic::ArithmeticExpr, word::Word},
    error::{exec::ExecError, parse::ParseError},
};

#[derive(Debug, Clone, Default)]
pub struct Subscript {
    pub text: String,
    data:     SubscriptType,
}

#[derive(Debug, Clone, Default)]
enum SubscriptType {
    #[default]
    None,
    Arith(ArithmeticExpr),
    // Evaluated(String),
    Array(String),
}

impl Subscript {
    pub fn eval(&mut self, core: &mut ShellCore, param_name: &str) -> Result<String, ExecError> {
        if let SubscriptType::Array(a) = &self.data {
            return Ok(a.clone());
        }
        // if let SubscriptType::Evaluated(s) = &self.data {
        // return Ok(s.clone());
        // }

        if let SubscriptType::Arith(mut a) = self.data.clone() {
            if a.text.is_empty() {
                return Err(ExecError::ArrayIndexInvalid(a.text.clone()));
            }
            if !core.db.is_assoc(param_name) {
                return a.eval(core);
            }

            if core.valid_assoc_expand_once && core.shopts.query("assoc_expand_once") {
                return Ok(a.text.clone());
            }

            let mut f = Feeder::new(&a.text);
            if let Some(w) = Word::parse(&mut f, core, None)? {
                return w.eval_as_assoc_index(core);
            } else {
                return Ok(a.text.clone());
            }
        }

        Err(ExecError::ArrayIndexInvalid("".to_string()))
    }

    pub fn reparse(&mut self, core: &mut ShellCore, param_name: &str) -> Result<(), ExecError> {
        if let SubscriptType::Array(_) = &self.data {
            return Ok(());
        }

        let mut text = self.eval(core, param_name)?;
        text.insert(0, '[');
        text.push(']');

        let mut f = Feeder::new(&text);
        match Self::parse(&mut f, core) {
            Ok(Some(s)) => *self = s,
            _ => return Err(ExecError::InvalidName(text.clone())),
        }

        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("[") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        if feeder.starts_withs(&["@", "*"]) {
            let s = feeder.consume(1);
            ans.text += &s.clone();
            ans.data = SubscriptType::Array(s);
        } else if let Some(a) = ArithmeticExpr::parse(feeder, core, true, "[")? {
            ans.text += &a.text.clone();
            ans.data = SubscriptType::Arith(a);
        }

        if !feeder.starts_with("]") {
            return Ok(None);
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
