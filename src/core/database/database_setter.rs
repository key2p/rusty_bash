// SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDXLicense-Identifier: BSD-3-Clause

// The methods of DataBase are distributed in database/database_*.rs files.

use std::{env, process};

use nix::unistd;

use super::{
    ArrayData, AssocData, Data, IntArrayData, IntAssocData, IntData, SingleData, UninitArray, UninitAssoc,
    data::{
        epochrealtime::EpochRealTime, epochseconds::EpochSeconds, random::RandomVar, seconds::Seconds,
        srandom::SRandomVar,
    },
};
use crate::{core::DataBase, error::exec::ExecError, utils::restricted_shell};

impl DataBase {
    pub fn init_as_num(&mut self, name: &str, value: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![value.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        match self.param_options[layer].get_mut(name) {
            Some(e) => *e += "i",
            None => {
                self.param_options[layer].insert(name.to_string(), "i".to_string());
            },
        }
        let db_layer = &mut self.params[layer];

        let mut data = IntData { body: 0 };

        if value != "" {
            match value.parse::<isize>() {
                Ok(n) => data.body = n,
                Err(e) => {
                    return Err(ExecError::Other(e.to_string()));
                },
            }
        }

        db_layer.insert(name.to_string(), Box::new(data));
        Ok(())
    }

    pub fn set_param(&mut self, name: &str, val: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] = val.to_string();
        }

        if !self.flags.contains('r') && (self.flags.contains('a') || self.has_flag(name, 'x')) {
            env::set_var(name, "");
        }

        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };
        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if env::var(name).is_ok() {
            env::set_var(name, &val);
        }

        if db_layer.get(name).is_none() {
            db_layer.insert(name.to_string(), Box::new(SingleData::from("")));
        }

        let d = db_layer.get_mut(name).unwrap();

        if d.is_array() {
            if !d.is_initialized() {
                *d = ArrayData::default().boxed_clone();
            }
            return d.set_as_array("0", &val);
        }

        if d.is_assoc() {
            if !d.is_initialized() {
                *d = AssocData::default().boxed_clone();
            }
            return d.set_as_assoc("0", &val);
        }

        d.set_as_single(&val)
    }

    pub fn append_param(&mut self, name: &str, val: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] += val;
        }

        if !self.flags.contains('r')
            && (self.flags.contains('a') || self.has_flag(name, 'x'))
            && !env::var(name).is_ok()
        {
            env::set_var(name, "");
        }

        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };
        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if let Ok(v) = env::var(name) {
            env::set_var(name, v + &val);
        }

        if db_layer.get(name).is_none() {
            db_layer.insert(name.to_string(), Box::new(SingleData::from("")));
        }

        let d = db_layer.get_mut(name).unwrap();

        if d.is_array() {
            return d.append_to_array_elem("0", &val);
        }

        d.append_as_single(&val)
    }

    pub fn set_param2(
        &mut self,
        name: &str,
        index: &String,
        val: &String,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.set_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.set_array_elem(&name, val, n, layer)?;
            }
        } else if self.is_assoc(name) {
            self.set_assoc_elem(&name, &index, val, layer)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => {
                    self.set_array_elem(&name, val, n, layer)?;
                },
                _ => {
                    self.set_assoc_elem(&name, &index, val, layer)?;
                },
            }
        }
        Ok(())
    }

    pub fn append_param2(
        &mut self,
        name: &str,
        index: &String,
        val: &String,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.append_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.append_to_array_elem(&name, val, n, layer)?;
            }
        } else if self.is_assoc(name) {
            self.append_to_assoc_elem(&name, &index, val, layer)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => {
                    self.append_to_array_elem(&name, val, n, layer)?;
                },
                _ => {
                    self.append_to_assoc_elem(&name, &index, val, layer)?;
                },
            }
        }
        Ok(())
    }

    pub fn set_array_elem(
        &mut self,
        name: &str,
        val: &String,
        pos: isize,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };

        if self.has_flag(name, 'i') {
            IntArrayData::set_elem(&mut self.params[layer], name, pos, &val)
        } else {
            ArrayData::set_elem(&mut self.params[layer], name, pos, &val)
        }
    }

    pub fn append_to_array_elem(
        &mut self,
        name: &str,
        val: &String,
        pos: isize,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };

        let layer = self.get_target_layer(name, layer);
        ArrayData::append_elem(&mut self.params[layer], name, pos, &val)
    }

    pub fn set_assoc_elem(
        &mut self,
        name: &str,
        key: &String,
        val: &String,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };
        let i_flag = self.has_flag(name, 'i');
        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        match db_layer.get_mut(name) {
            Some(v) => {
                if !v.is_initialized() {
                    *v = match i_flag {
                        true => IntAssocData::default().boxed_clone(),
                        false => AssocData::default().boxed_clone(),
                    };
                }

                v.set_as_assoc(key, &val)
            },
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn append_to_assoc_elem(
        &mut self,
        name: &str,
        key: &String,
        val: &String,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &Some(vec![val.to_string()]))?;

        let val = match self.has_flag(name, 'l') {
            true => val.to_string().to_lowercase(),
            false => val.to_string(),
        };

        let layer = self.get_target_layer(name, layer);
        AssocData::append_elem(&mut self.params[layer], name, key, &val)
    }

    pub fn set_array(&mut self, name: &str, v: Option<Vec<String>>, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &v)?;

        let l_flag = self.has_flag(name, 'l');
        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if v.is_none() {
            db_layer.insert(name.to_string(), UninitArray {}.boxed_clone());
        } else {
            let v = match l_flag {
                true => v.unwrap().iter().map(|e| e.to_lowercase()).collect(),
                false => v.unwrap(),
            };

            db_layer.insert(name.to_string(), Box::new(ArrayData::from(Some(v))));
        }
        Ok(())
    }

    pub fn set_int_array(&mut self, name: &str, v: Option<Vec<String>>, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &v)?;

        let layer = self.get_target_layer(name, layer);

        match self.param_options[layer].get_mut(name) {
            Some(e) => *e += "i",
            None => {
                self.param_options[layer].insert(name.to_string(), "i".to_string());
            },
        }

        let db_layer = &mut self.params[layer];
        db_layer.insert(name.to_string(), IntArrayData::default().boxed_clone());

        if v.is_some() {
            for (i, e) in v.unwrap().into_iter().enumerate() {
                self.set_array_elem(&name, &e, i as isize, Some(layer))?;
            }
        }

        Ok(())
    }

    pub fn set_int_assoc(&mut self, name: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &None)?;

        let layer = self.get_target_layer(name, layer);

        match self.param_options[layer].get_mut(name) {
            Some(e) => *e += "i",
            None => {
                self.param_options[layer].insert(name.to_string(), "i".to_string());
            },
        }

        let db_layer = &mut self.params[layer];
        db_layer.insert(name.to_string(), IntAssocData::default().boxed_clone());
        Ok(())
    }

    pub fn set_assoc(&mut self, name: &str, layer: Option<usize>, set_array: bool) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        restricted_shell::check(self, name, &None)?;

        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if set_array {
            db_layer.insert(name.to_string(), Box::new(AssocData::default()));
        } else {
            db_layer.insert(name.to_string(), UninitAssoc {}.boxed_clone());
        }
        Ok(())
    }

    pub fn set_flag(&mut self, name: &str, flag: char, layer: Option<usize>) {
        let layer = match layer {
            None => self.position_parameters.len() - 1,
            Some(lay) => lay,
        };

        let rf = &mut self.param_options[layer];
        match rf.get_mut(name) {
            Some(d) => d.push(flag),
            None => {
                rf.insert(name.to_string(), flag.to_string());
            },
        }
    }
}

pub fn initialize(db: &mut DataBase) -> Result<(), String> {
    db.exit_status = 0;

    db.set_param("$", &process::id().to_string(), None)?;
    db.set_param("BASHPID", &process::id().to_string(), None)?;
    db.set_param("BASH_SUBSHELL", "0", None)?;
    db.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
    db.set_param("OPTIND", "1", None)?;
    db.set_param("IFS", " \t\n", None)?;

    db.init_as_num("UID", &unistd::getuid().to_string(), None)?;
    db.param_options[0].insert("UID".to_string(), "ir".to_string());

    db.params[0].insert("RANDOM".to_string(), Box::new(RandomVar::new()));
    db.param_options[0].insert("RANDOM".to_string(), "i".to_string());

    db.params[0].insert("SRANDOM".to_string(), Box::new(SRandomVar::new()));
    db.param_options[0].insert("SRANDOM".to_string(), "i".to_string());

    db.params[0].insert("SECONDS".to_string(), Box::new(Seconds::new()));
    db.params[0].insert("EPOCHSECONDS".to_string(), Box::new(EpochSeconds {}));
    db.params[0].insert("EPOCHREALTIME".to_string(), Box::new(EpochRealTime {}));

    db.set_array("FUNCNAME", None, None)?;
    db.set_array("BASH_SOURCE", Some(vec![]), None)?;
    db.set_array("BASH_ARGC", Some(vec![]), None)?;
    db.set_array("BASH_ARGV", Some(vec![]), None)?;
    db.set_array("BASH_LINENO", Some(vec![]), None)?;
    db.set_array("DIRSTACK", Some(vec![]), None)?;
    db.set_assoc("BASH_CMDS", None, false)?;
    Ok(())
}
