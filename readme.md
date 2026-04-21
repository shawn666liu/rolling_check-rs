# 期货换月检查
- 普通换月规则:   
成交量权重0.6，持仓量权重0.4,(缺省参数可修改)   
后续合约权重超过当前主力合约1.1倍，则进行换月，  
不开倒车，即使前面合约权重又超越了后续的主力合约，也不向前切换,  
- 股指期货规则:  
股指期货逐月换，中间不跨越，即使远月合约权重更大也不换
- 强制换月规则:  
国债期货，交割月的第二个星期五  
股指期货，交割月的第三个星期五  
原油期货，交割月之前一个月的倒数第11个交易日  
可在强制规则基础上, 通过设置early_days, 提前处理,  


```rust
use tradecalendar::get_buildin_calendar;
use rolling_check::*;

let mut md_vec: Vec<SimpleMktData> = ....;
let tdmgr = get_buildin_calendar(None)?;
let mut checker = RollingChecker::new(&tdmgr, &NaiveDate::from_ymd(2025, 4, 1));
// 如果要修改缺省参数，需要在check_product之前调用
checker.set_volume_weight(0.6);
checker.set_openint_weight(0.4);
checker.set_weight_threshold(1.1);
checker.set_normal_early_days(3);
checker.set_finance_early_days(3);

let (opt_current, next) = checker.check_product("ru2509", md_vec)?;
println!("{:#?} -> {:#?}", opt_current, next);
```
参看utest.rs
 