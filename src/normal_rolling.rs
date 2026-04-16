use chrono::Datelike;
use chrono::NaiveDate;
use tradecalendar::{TradeCalendar, TradingdayCache};

/// 处理其他普通品种, 非股指, 非国债,非原油
pub struct NormalRolling {}

impl NormalRolling {
    /// expire_date是法人户的最后交易日期，自然人需要提早到上一个月末,
    /// (该月倒数第3个交易日), 此函数计算这个交易日的具体日期
    pub fn calc_must_exit_date(tdmgr: &TradeCalendar, expire_date: &NaiveDate) -> NaiveDate {
        // 比如ru2205, 在5月交割, 我们必须在4月份最后的交易日前换月,
        // 如果4月31日是最后交易日, 那么应该在29号左右强制换月, 这时候无需满足权重1.1倍的要求
        let expire_month = expire_date.with_day(1).expect("no fail");
        let pre_3_tdays = tdmgr
            .get_prev_trading_day(&expire_month, 3)
            .expect("failed to get prev 3 trading day");
        return pre_3_tdays.date;
    }
}
