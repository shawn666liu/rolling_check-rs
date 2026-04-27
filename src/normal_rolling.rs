use chrono::Datelike;
use chrono::NaiveDate;
use tradecalendar::{TradeCalendar, TradingdayCache};

/// 处理其他普通品种, 非股指, 非国债,非原油
pub struct NormalRolling {}

impl NormalRolling {
    /// expire_date是法人户的最后交易日期，自然人需要提早到上一个月末,
    /// (该月倒数第early_days个交易日), 此函数计算这个交易日的具体日期
    pub fn calc_must_exit_date(
        tdmgr: &TradeCalendar,
        expire_date: &NaiveDate,
        early_days: usize,
    ) -> NaiveDate {
        // 比如ru2205, 在5月交割, 我们必须在4月份最后的交易日前换月,
        // 如果4月31日是最后交易日, 那么应该在31-early_days左右强制换月, 这时候无需满足权重1.1倍的要求
        let expire_month = expire_date.with_day(1).expect("no fail");
        // 这里加上1, 因为expire_month是交割月第一天，强制换月是上个月最后一个交易日, 所以需要加1
        let early_days = early_days + 1;
        let pre_n_tdays = tdmgr
            .get_prev_trading_day(&expire_month, early_days)
            .expect(&format!("failed to get prev {} trading day", early_days));
        return pre_n_tdays.date;
    }
}
