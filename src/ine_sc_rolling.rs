use chrono::Datelike;
use chrono::NaiveDate;

use crate::util::get_inst_month;
use tradecalendar::{TradeCalendar, TradingdayCache};

/// 国际能源交易中心INE, 原油期货换月专用
pub struct IneScRolling {}

/// 本来应该是交割月之前一个月的倒数第11个交易日,必须换月,
/// 为了避免期货经纪商经常电话催促, 设定为12,
/// 但目前先测试, 任然保持11
const LOOKBACK_DAYS: usize = 11;

impl IneScRolling {
    /// 根据某个合约的expire_date, 计算它必须换月的日期,<br>
    /// 一般是交割月前一个月最后交易日向前推11个交易日,通过early_days参数指定提前处理增加的天数,<br>
    /// instrument是该合约名称, 一般情况是合约名跟最后交易日是在同一个月, 比如sc2205就是在22年5月某天最后交易日;<br>
    /// 但是因为元旦春节国庆等假期, 会导致不一致, 比如sc2002合约, 理论上最后交易日应该在20年2月某日, 但实际最后交易日在20年1月16日<br>
    /// early_days: 提前处理的天数, 缺省0, 实际在11的基础上增加
    pub fn calc_must_exit_date(
        tdmgr: &TradeCalendar,
        instrument: &str,
        expire_date: &NaiveDate,
        early_days: usize,
    ) -> NaiveDate {
        let inst_month = get_inst_month(instrument, expire_date.year());
        let expire_month = expire_date.with_day(1).expect("no fail");
        let last_day = if expire_month == inst_month {
            // 比如同在5月, 则从5月1号开始, 向前推11个交易日, 就是换月日期
            // expire_month==inst_month==2022-05-01
            &expire_month
        } else {
            // 比如合约sc2002为2月, 但最后交易日1月16, 从1月16日向前推11个交易日换月
            // expire_month==2020-01-01
            //  inst_month ==2020-02-01
            expire_date
        };
        let pr_tdays = tdmgr
            .get_prev_trading_day(last_day, LOOKBACK_DAYS + early_days)
            .expect("failed to get prev LOOKBACK_DAYS trading day");
        return pr_tdays.date;
    }
}
