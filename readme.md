# 期货换月检查
- 普通换月规则:   
成交量权重0.6，持仓量权重0.4,  
后续合约权重超过当前主力合约，则进行换月，  
不开倒车，即使前面合约权重又超越了后续的主力合约，也不向前切换,  
- 强制换月规则:  
国债期货，交割月的第二个星期五  
股指期货，交割月的第三个星期五  
原油期货，交割月之前一个月的倒数第11个交易日  
可在强制规则基础上, 提前处理  

```rust
use tradecalendar::get_buildin_calendar;
use rolling_check::*;

let mut md_vec: Vec<SimpleMktData> = ....;
let tdmgr = get_buildin_calendar(None)?;
let checker = RollingChecker::new(&tdmgr, &NaiveDate::from_ymd(2025, 4, 1));
let result = checker.check_product("ru2509", md_vec)?;
println!("{:#?}", result);

```