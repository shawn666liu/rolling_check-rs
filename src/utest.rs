#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tradecalendar::{self, TradeCalendar};

    use crate::rolling_check::{RollingChecker, SimpleMktData};

    fn get_calendar() -> TradeCalendar {
        tradecalendar::get_buildin_calendar(None).unwrap()
    }

    fn create_test_mkt_data(
        inst: &str,
        volume: u64,
        openint: u64,
        expire_date: &str,
    ) -> SimpleMktData {
        SimpleMktData::new(
            inst.to_string(),
            volume,
            openint,
            NaiveDate::parse_from_str(expire_date, "%Y-%m-%d").unwrap(),
        )
    }

    #[test]
    fn test_check_product_normal_rolling() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 创建测试数据：当前主力合约ru2401，新合约ru2403权重更高
        let md_vec = vec![
            create_test_mkt_data("ru2401", 1000, 500, "2024-01-31"),
            create_test_mkt_data("ru2403", 2000, 1000, "2024-03-31"),
            create_test_mkt_data("ru2405", 500, 300, "2024-05-31"),
        ];

        let result = checker.check_product("ru2401", md_vec, "SHFE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "ru2401");
        assert_eq!(next.inst, "ru2403"); // 权重更高的合约应该成为新主力
    }

    #[test]
    fn test_check_product_normal_rolling2() {
        let calendar = get_calendar();
        let test_date = NaiveDate::from_ymd_opt(2026, 04, 20).unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 虽然j2609的权重更高，但是没有超过j2605的1.1倍，所以主力合约还是j2605
        let md_vec = vec![
            create_test_mkt_data("j2605", 13835, 14340, "2026-05-19"),
            create_test_mkt_data("j2609", 10973, 20290, "2026-09-14"),
            create_test_mkt_data("j2701", 219, 2064, "2027-01-15"),
        ];

        let result = checker.check_product("j2605", md_vec, "SHFE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "j2605");
        assert_eq!(next.inst, "j2605"); // 权重更高的合约应该成为新主力
    }

    #[test]
    fn test_check_product_new_product_no_current() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2023-12-02", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 新品种上市，没有当前主力合约
        let md_vec = vec![
            create_test_mkt_data("NEW2401", 1500, 800, "2024-01-15"),
            create_test_mkt_data("NEW2403", 800, 400, "2024-03-15"),
        ];

        let result = checker.check_product("", md_vec, "DCE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_none()); // 当前主力合约为空
        assert_eq!(next.inst, "NEW2401"); // 应该选择权重最高的合约
    }

    #[test]
    fn test_check_product_new_product_no_current2() {
        let calendar = get_calendar();
        // 注意这个日期，离2401强制退市还有3天，（普遍品种不能进入交割月)
        // 因为normal_early_days缺省设置3天，所以2401不能成为主力合约了，需强制换月
        let test_date = NaiveDate::parse_from_str("2023-12-27", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 新品种上市，没有当前主力合约
        let md_vec = vec![
            create_test_mkt_data("NEW2401", 1500, 800, "2024-01-15"),
            create_test_mkt_data("NEW2403", 800, 400, "2024-03-15"),
        ];

        let result = checker.check_product("", md_vec, "DCE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_none()); // 当前主力合约为空
        assert_eq!(next.inst, "NEW2403"); // 2401强制退市还有3天，所以选择后续合约2403
    }

    #[test]
    fn test_check_product_empty_market_data() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 空的市场数据应该返回错误
        let md_vec: Vec<SimpleMktData> = vec![];

        let result = checker.check_product("ru2401", md_vec, "SHFE");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("md_vec is empty"));
    }

    #[test]
    fn test_check_product_stock_index_rolling() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 股指期货换月测试
        let md_vec = vec![
            create_test_mkt_data("IF2401", 5000, 3000, "2024-01-19"),
            create_test_mkt_data("IF2402", 6000, 3500, "2024-02-16"),
            create_test_mkt_data("IF2403", 4000, 2500, "2024-03-15"),
        ];

        let result = checker.check_product("IF2401", md_vec, "CFFEX");
        assert!(result.is_ok());

        let (current, _next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "IF2401");
        // 股指期货有特殊的换月规则，这里主要测试流程正常
    }

    #[test]
    fn test_check_product_ine_sc_rolling() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 原油期货换月测试
        let md_vec = vec![
            create_test_mkt_data("sc2401", 8000, 5000, "2024-01-31"),
            create_test_mkt_data("sc2402", 9000, 5500, "2024-02-29"),
            create_test_mkt_data("sc2403", 7000, 4500, "2024-03-31"),
        ];

        let result = checker.check_product("sc2401", md_vec, "INE");
        assert!(result.is_ok());

        let (current, _next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "sc2401");
        // 原油期货有特殊的强制换月规则
    }

    #[test]
    fn test_check_product_bonds_future() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 国债期货换月测试
        let md_vec = vec![
            create_test_mkt_data("T2403", 3000, 2000, "2024-03-15"),
            create_test_mkt_data("T2406", 3500, 2200, "2024-06-14"),
            create_test_mkt_data("T2409", 2500, 1800, "2024-09-13"),
        ];

        let result = checker.check_product("T2403", md_vec, "CFFEX");
        assert!(result.is_ok());

        let (current, _next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "T2403");
    }

    #[test]
    fn test_check_product_current_not_in_list() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 当前主力合约不在市场数据列表中（可能已下市）
        let md_vec = vec![
            create_test_mkt_data("ru2403", 2000, 1000, "2024-03-31"),
            create_test_mkt_data("ru2405", 1500, 800, "2024-05-31"),
        ];

        let result = checker.check_product("ru2401", md_vec, "SHFE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_none()); // 当前合约不在列表中，应该返回None
        assert_eq!(next.inst, "ru2403"); // 应该选择权重最高的合约
    }

    #[test]
    fn test_check_product_single_contract() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 只有一个合约的情况
        let md_vec = vec![create_test_mkt_data("ru2401", 1000, 500, "2024-01-31")];

        let result = checker.check_product("ru2401", md_vec, "SHFE");
        assert!(result.is_ok());

        let (current, next) = result.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().inst, "ru2401");
        assert_eq!(next.inst, "ru2401"); // 只有一个合约，应该保持不变
    }

    #[test]
    fn test_check_product_weight_calculation() {
        let calendar = get_calendar();
        let test_date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
        let checker = RollingChecker::new(&calendar, &test_date);

        // 测试权重计算是否正确
        let mut md1 = create_test_mkt_data("ru2401", 1000, 500, "2024-01-31");
        let mut md2 = create_test_mkt_data("ru2403", 2000, 1000, "2024-03-31");

        // 手动计算权重进行验证
        md1.calc_combined_weight(0.6, 0.4);
        md2.calc_combined_weight(0.6, 0.4);

        let md_vec = vec![md1, md2];
        let result = checker.check_product("ru2401", md_vec, "SHFE");
        assert!(result.is_ok());
    }
}
