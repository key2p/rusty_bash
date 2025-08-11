// SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDXLicense-Identifier: BSD-3-Clause

mod data;
mod database_checker;
mod database_getter;
mod database_setter;

use std::collections::HashMap;

use self::data::{
    Data, array::ArrayData, array_int::IntArrayData, array_uninit::UninitArray, assoc::AssocData,
    assoc_int::IntAssocData, assoc_uninit::UninitAssoc, single::SingleData, single_int::IntData,
};
use crate::{elements::command::function_def::FunctionDefinition, env, error::exec::ExecError, exit};
// use self::data::special::SpecialData;

#[derive(Debug, Default)]
pub struct DataBase {
    pub flags:               String,
    pub params:              Vec<HashMap<String, Box<dyn Data>>>,
    pub param_options:       Vec<HashMap<String, String>>,
    pub position_parameters: Vec<Vec<String>>,
    pub functions:           HashMap<String, FunctionDefinition>,
    pub exit_status:         i32,
    pub last_arg:            String,
    pub hash_counter:        HashMap<String, usize>,
}

impl DataBase {
    pub fn new() -> DataBase {
        let mut data = DataBase {
            params: vec![HashMap::new()],
            param_options: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            flags: "B".to_string(),
            ..Default::default()
        };

        database_setter::initialize(&mut data).unwrap();
        data
    }

    pub fn get_target_layer(&mut self, name: &str, layer: Option<usize>) -> usize {
        match layer {
            Some(n) => n,
            None => self.solve_layer(name),
        }
    }

    fn solve_layer(&mut self, name: &str) -> usize {
        self.get_layer_pos(name).unwrap_or(0)
    }

    pub fn push_local(&mut self) {
        self.params.push(HashMap::new());
        match self.param_options.last() {
            Some(e) => self.param_options.push(e.clone()),
            None => exit::internal("error: DataBase::push_local"),
        }
    }

    pub fn pop_local(&mut self) {
        self.params.pop();
        self.param_options.pop();
    }

    pub fn init(&mut self, name: &str, layer: usize) {
        if let Some(d) = self.params[layer].get_mut(name) {
            d.clear();
        }
    }

    pub fn unset_var(&mut self, name: &str) {
        env::remove_var(name);

        for layer in &mut self.params {
            layer.remove(name);
        }
        for layer in &mut self.param_options {
            layer.remove(name);
        }
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(&mut self, name: &str) {
        self.unset_var(name);
        self.unset_function(name);
    }

    pub fn unset_array_elem(&mut self, name: &str, key: &str) -> Result<(), ExecError> {
        if self.is_single(name) {
            if key == "0" || key == "@" || key == "*" {
                self.unset_var(name);
                return Ok(());
            }
        }

        for layer in &mut self.params {
            if let Some(d) = layer.get_mut(name) {
                let _ = d.remove_elem(key)?;
            }
        }
        Ok(())
    }

    pub fn print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            d.print_with_name(name, false);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    pub fn declare_print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            d.print_with_name(name, true);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    pub fn int_to_str_type(&mut self, name: &str, layer: usize) -> Result<(), ExecError> {
        let layer_len = self.param_options.len();
        for ly in layer..layer_len {
            if let Some(opt) = self.param_options[ly].get_mut(name) {
                opt.retain(|c| c != 'i');
            }
        }

        if let Some(d) = self.params[layer].get_mut(name) {
            let new_d = d.get_str_type();
            self.params[layer].insert(name.to_string(), new_d);
        }

        Ok(())
    }
}
