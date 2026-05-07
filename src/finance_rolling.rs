use chrono::{Datelike, Duration, NaiveDate, Weekday};
use tradecalendar::{TradeCalendar, TradingdayCache};

use crate::{SimpleMktData, util};

/// 金融期货最后交易所在的星期
#[derive(Copy, Clone)]
pub enum ProductExitWeek {
    /// 国债期货, 交割月的第二个星期五
    Bonds = 2,
    /// 股指期货, 交割月的第三个星期五
    StockIndex = 3,
}

// 用于判断金融期货强制换月，
// 股指期货，强制在交割月第三周星期四换月,
// 国债期货，强制在交割月第二周星期四换月,
pub struct FinanceRolling {}

impl FinanceRolling {
    /// 计算股指/国债强制换月日期
    pub fn calc_must_exit_date(
        tdmgr: &TradeCalendar,
        expire_date: &NaiveDate,
        weeks: ProductExitWeek,
        early_days: usize,
    ) -> NaiveDate {
        // 目前股指和国债的合约名与交割月是统一的

        let friday = &get_friday(expire_date, weeks);
        let last_day = friday.min(expire_date);
        // 最后日期向前推2个交易日
        let pre_tdays = tdmgr
            .get_prev_trading_day(last_day, early_days)
            .expect(&format!("failed to get prev {} trading day", early_days));
        return pre_tdays.date;
    }

    /// 因为股指期货是每个月都存在合约的，计算出下一个月就可以知道下一个合约名
    pub fn calc_next_inst(smd: &SimpleMktData) -> String {
        let curr_month = util::get_inst_month(&smd.inst, smd.expire_date.year());
        let next_month = util::next_month(&curr_month);
        let prd = util::trim_num_and_after(&smd.inst);

        let next_inst = format!(
            "{}{:>02}{:>02}",
            prd,
            next_month.year() % 100,
            next_month.month()
        );
        return next_inst;
    }
}

/// 获取输入日期所在月份的第某个（二或者三）星期五
fn get_friday(input: &NaiveDate, which_week: ProductExitWeek) -> NaiveDate {
    let first_day = input.with_day(1).expect("no fail");
    let next_month = util::next_month(input);

    let mut result = first_day;
    let mut friday_count = 0;
    while result < next_month {
        if result.weekday() == Weekday::Fri {
            friday_count += 1;
            if friday_count == which_week as u8 {
                break;
            }
        }
        result += Duration::days(1);
    }
    return result;
}
