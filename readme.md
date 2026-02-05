# 期货换月检查
```rust
use tradecalendar::get_buildin_calendar;
use rolling_check::*;

let mut md_vec: Vec<SimpleMktData> = ....;
let tdmgr = get_buildin_calendar(None)?;
let checker = RollingChecker::new(&tdmgr, &NaiveDate::from_ymd(2025, 4, 1));
let result = checker.check_product("ru2509", md_vec)?;
println!("{:#?}", result);

```