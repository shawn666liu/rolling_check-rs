#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
#![allow(deprecated)]

// 换月检查模块

mod finance_rolling;
mod ine_sc_rolling;
mod normal_rolling;
mod rolling_check;
mod util;

pub use crate::rolling_check::*;
