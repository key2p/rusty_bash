// SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
// SPDX-License-Identifier: BSD-3-Clause

use super::elem::ArithElem;
use crate::error::{arith::ArithError, exec::ExecError};

pub fn rearrange(elements: &[ArithElem]) -> Result<Vec<ArithElem>, ExecError> {
    let mut ans = vec![];
    let mut stack: Vec<ArithElem> = vec![];
    let mut prev_is_op = false;

    for e in elements {
        let is_op = e.is_operand();
        if prev_is_op && is_op {
            return Err(ArithError::SyntaxError(e.to_string()).into());
        }
        prev_is_op = is_op;

        match is_op {
            true => ans.push(e.clone()),
            false => rev_polish_op(&e, &mut stack, &mut ans),
        };
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    Ok(ans)
}

fn rev_polish_op(elem: &ArithElem, stack: &mut Vec<ArithElem>, ans: &mut Vec<ArithElem>) {
    loop {
        match stack.last() {
            None => {
                stack.push(elem.clone());
                break;
            },
            Some(_) => {
                let last = stack.last().unwrap();
                if last.order() < elem.order() || (last.order() == 2 && elem.order() == 2) {
                    // assignment
                    stack.push(elem.clone());
                    break;
                }
                ans.push(stack.pop().unwrap());
            },
        }
    }
}
