// SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDX-License-Identifier: BSD-3-Clause

use nix::unistd;

use crate::{
    Feeder, ShellCore,
    elements::{
        Pipe,
        command::{Command, paren::ParenCommand},
        subword::Subword,
        word::WordMode,
    },
    error::{exec::ExecError, parse::ParseError},
};

#[derive(Debug, Clone, Default)]
pub struct ProcessSubstitution {
    pub text:      String,
    command:       ParenCommand,
    pub direction: char,
}

impl Subword for ProcessSubstitution {
    fn get_text(&self) -> &str {
        &self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.direction != '<' {
            return Err(ExecError::Other(">() is not supported yet".to_string()));
        }

        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let _ = self.command.exec(core, &mut pipe)?;
        self.text = "/dev/fd/".to_owned() + &pipe.recv.to_string();
        Ok(())
    }
}

impl ProcessSubstitution {
    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        mode: &Option<WordMode>,
    ) -> Result<Option<Self>, ParseError> {
        if let Some(WordMode::Arithmetic) = mode {
            return Ok(None);
        }

        if !feeder.starts_with("<(") && !feeder.starts_with(">(") {
            return Ok(None);
        }
        let mut ans = ProcessSubstitution::default();
        ans.text = feeder.consume(1);
        ans.direction = ans.text.chars().nth(0).unwrap();

        if let Some(pc) = ParenCommand::parse(feeder, core, true)? {
            ans.text += &pc.get_text();
            ans.command = pc;
            return Ok(Some(ans));
        }

        Ok(None)
    }
}
