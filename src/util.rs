use chrono::{Datelike, NaiveDate};

/// 移除首个出现的数字及其后所有字符,如果首字母是数字则从其后首个非数字之后开始移除  
///
/// IF1603 => IF,  
///
/// y1605 => y,  
///
/// 黄大豆1号1501 => 黄大豆,  
///
/// 10年国债2003 => 10年国债,  
///
/// 1605 => 1605,
///
/// IF => IF
///
/// "" => ""
pub fn trim_num_and_after(input: &str) -> &str {
    let mut any_non_number = false;
    let buf = input.as_bytes();
    for (i, c) in buf.iter().enumerate() {
        if !c.is_ascii_digit() {
            any_non_number = true;
            continue;
        }
        if i > 0 && any_non_number {
            return &input[0..i];
        }
    }
    return input;
}

/// 获取尾部的数字<br>
/// IF1905 => 1905<br>
/// IF1905A => ""<br>
/// 黄大豆1号1501 => 1501
pub fn get_tail_numbers(input: &str) -> &str {
    let buf = input.as_bytes();
    for (i, c) in buf.iter().rev().enumerate() {
        if !c.is_ascii_digit() {
            if i == 0 {
                return "";
            }
            return &input[buf.len() - i..];
        }
    }
    return input;
}

/// 根据合约名及最后交易日期所在的年份, 获取该合约名代表的月份(日期为1号),<br>
/// expire_year在这里的作用是提供年份指引, 因为合约名没有年份的前两位数字,<br>
/// 注意, 这里要求instrument已经是4位数字, 如果不是请先调用fix_czce_inst(),<br>
/// 例如, ru2205 => 2022-05-01
pub fn get_inst_month(instrument: &str, expire_year: i32) -> NaiveDate {
    // 一般情况下,合约月份与最后交易日在同一个月, 比如sc2205的交割月为22年5月;
    // 但是因元旦春节等影响, 也有例外, 比如 sc2002的最后交易日为20年1月16日,并不在2月份,

    // 那么会不会有合约名为3001, 而最后交易日在2029-12-xx的情况呢?
    // 这种情况直接取2029的前两位补充到3001, 成为2030-01是没有问题的

    // 但是0001最后交易日在1999-12-xx, 简单补充则为1900-01-01

    // sc0001 => 0001
    let inst_no_prd: i32 = get_tail_numbers(instrument)
        .parse()
        .expect("Failed to parse instrument number");
    let month = (inst_no_prd % 100 + 11) % 12 + 1;

    // 若expire_year = 1999, 则year = 1900
    let mut year = expire_year / 100 * 100 + inst_no_prd / 100;
    if expire_year > year {
        year += 100;
    }
    return NaiveDate::from_ymd_opt(year, month as u32, 1).expect("no fail");
}

/// 日期都是1号
pub fn next_month(date: &NaiveDate) -> NaiveDate {
    let year = date.year();
    let month = date.month();
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("no fail")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("no fail")
    };
    return next_month;
}
