use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::util;
use tradecalendar::*;

use crate::finance_rolling::{FinanceRolling, ProductExitWeek};
use crate::ine_sc_rolling::IneScRolling;
use crate::normal_rolling::NormalRolling;

/// 换月检查需要用到的最基础的行情数据
#[derive(Clone, Debug)]
pub struct SimpleMktData {
    /// 对于郑州品种, 也是4位数字, 必须已经处理好了<br>
    /// 仅依靠合约进行排序是不可靠的, 比如1999.12的合约名为9912, 2000.1月的合约名为0001, 后续合约排序反而在前了<br>
    /// 所以，使用expire_date排序
    pub inst: String,
    pub volume: u64,
    pub openint: u64,
    /// 用此字段排序是可以的
    pub expire_date: NaiveDate,

    /// 复合权重, 利用成交量和持仓量算得
    pub combined_weight: u64,
}

impl SimpleMktData {
    pub fn new(
        inst: String,
        xchg: String,
        volume: u64,
        openint: u64,
        expire_date: NaiveDate,
    ) -> Self {
        Self {
            inst,
            volume,
            openint,
            expire_date,
            combined_weight: 0,
        }
    }

    pub fn calc_combined_weight(&mut self, volume_weight: f64, openint_weight: f64) {
        self.combined_weight =
            (self.volume as f64 * volume_weight + self.openint as f64 * openint_weight) as u64;
    }

    /// next合约是否满足取代current作为主力合约, 需满足两个条件
    /// 1) 复合权重达到旧合约1.1倍
    /// 2) next是更远的月份
    pub fn is_rolling_needed(current: &Self, next: &Self) -> bool {
        // 这里比较Weight不使用>=的原因，是因为有些不活跃的品种，持仓都为零，会被换到最远的远月
        if next.expire_date > current.expire_date
            && next.combined_weight > (current.combined_weight as f64 * 1.1) as u64
        {
            return true;
        }
        return false;
    }
}

pub struct RollingChecker<'a> {
    tdmgr: &'a TradeCalendar,
    /// 用于测试的日期, 实际情况都是today
    test_day: NaiveDate,
    stock_index: HashSet<String>,
    bonds_future: HashSet<String>,
    ine_sc: HashSet<String>,
    /// 换月时成交量权重, default 0.6
    volume_weight: f64,
    /// 换月时持仓量权重, default 0.4
    openint_weight: f64,
}

impl<'a> RollingChecker<'a> {
    pub fn new(tdmgr: &'a TradeCalendar, test_day: &NaiveDate) -> Self {
        let stock_index: HashSet<_> = (vec!["IF", "IC", "IH", "IM"])
            .iter()
            .map(|&x| x.to_string())
            .collect();
        let bonds_future: HashSet<_> = (vec!["T", "TF", "TS", "TL"])
            .iter()
            .map(|&x| x.to_string())
            .collect();
        let mut ine_sc = HashSet::default();
        ine_sc.insert("sc".to_string());
        Self {
            tdmgr,
            test_day: test_day.clone(),
            stock_index,
            bonds_future,
            ine_sc,
            volume_weight: 0.6,
            openint_weight: 0.6,
        }
    }

    /// 若后期有新品种上市,可添加
    pub fn set_ine_sc(&mut self, items: HashSet<String>) {
        self.ine_sc = items;
    }
    /// 若后期有新品种上市,可添加
    pub fn set_stock_index(&mut self, items: HashSet<String>) {
        self.stock_index = items;
    }
    /// 若后期有新品种上市,可添加
    pub fn set_bonds_future(&mut self, items: HashSet<String>) {
        self.bonds_future = items;
    }

    pub fn set_volume_weight(&mut self, weight: f64) {
        self.volume_weight = weight;
    }
    pub fn set_openint_weight(&mut self, weight: f64) {
        self.openint_weight = weight;
    }

    /// 对某一个品种进行换月检查，输入:<br>  
    /// - cur_primary, 当前主力合约,<br>
    /// - md_vec, 当天`该品种`所有合约的行情(成交量,持仓量,到期日)
    ///
    /// 返回值:<br>
    /// - opt(current), 当前主力合约<br>
    /// - next_primary, 下一个主力合约 <br>
    ///
    /// 注意:<br>
    /// 1) 当前主力合约有可能是空,比如新品种上市,主力合约未知;<br>
    /// 2) 有可能是不存在的合约, 比如很久都没有执行换月程序了, 突然重新启用程序, 显然旧合约已经下市了<br>
    pub fn check_product(
        &self,
        cur_primary: &str,
        md_vec: Vec<SimpleMktData>,
        exchange: &str,
    ) -> Result<(Option<SimpleMktData>, SimpleMktData)> {
        anyhow::ensure!(
            md_vec.len() > 0,
            format!("rolling_check: md_vec is empty. {}", cur_primary)
        );

        let mut md_vec = md_vec;
        md_vec
            .iter_mut()
            .for_each(|x| x.calc_combined_weight(self.volume_weight, self.openint_weight));

        let opt_current: Option<&SimpleMktData> = md_vec.iter().find(|&x| x.inst == cur_primary);
        // 按照月份，从小到大排序
        let mut ordby_expire: Vec<_> = md_vec.iter().collect();
        ordby_expire.sort_unstable_by_key(|x| x.expire_date);
        // 以月份序升序为基础, 复制一份
        let mut ordby_weight: Vec<_> = ordby_expire.clone();
        // 权重从大到小排列, 所以比较时t2在前; 若weight相同, 则保持月份顺序, 所以不用unstable_sort
        ordby_weight.sort_by(|t1, t2| t2.combined_weight.cmp(&t1.combined_weight));

        let any_md = &md_vec.iter().next().expect("no fail");
        anyhow::ensure!(
            any_md.inst != "",
            format!("rolling_check: any_md.inst is empty. {}", cur_primary)
        );
        let product = util::trim_num_and_after(&any_md.inst);
        anyhow::ensure!(
            product != "",
            format!("rolling_check: product is empty for {}", any_md.inst)
        );
        let next_primary = if self.ine_sc.contains(product) {
            self.check_ine_sc(opt_current, &ordby_expire, &ordby_weight)
        } else if self.stock_index.contains(product) {
            self.check_stock_index(opt_current, &ordby_expire, &ordby_weight)
        } else if self.bonds_future.contains(product) {
            self.check_bonds(opt_current, &ordby_expire, &ordby_weight)
        } else {
            // 如果没有及时更新股指或者国债品种列表，新品种上市之后，只能作为普通合约处理
            // 但是我们会进行一个提醒, 通知用户更新列表
            if exchange == "CFFEX" {
                return Err(anyhow::anyhow!(
                    "rolling_check: 似乎有CFFEX新品种上市, 请先更新列表, {}",
                    product
                ));
            }

            self.check_others(opt_current, &ordby_expire, &ordby_weight)
        };
        let opt = match opt_current {
            Some(cur) => Some(cur.clone()),
            None => None,
        };
        Ok((opt, next_primary.clone()))
    }

    /// 检查原油期货换月
    fn check_ine_sc(
        &self,
        opt_current: Option<&'a SimpleMktData>,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        // 比如当前日期2018年12月17日，当前主力合约sc1901, 必须在17日收盘后，换月到sc1903（sc1902复合权重较小）
        // orderedByWeight里面的内容: sc1901, sc1903, sc1902 ...

        // 1) 获取可能正确选择的一个合约
        let mut maybe_this = self.get_next_normally(opt_current, ordby_expire, ordby_weight);

        // 2) 计算此合约需要强制换月的日期
        let must_exit_date = IneScRolling::calc_must_exit_date(
            self.tdmgr,
            &maybe_this.inst,
            &maybe_this.expire_date,
        );

        // 3) 若需强制换月, 则进行处理
        if must_exit_date <= self.test_day {
            maybe_this = self.fore_rolling_check(maybe_this, ordby_expire, ordby_weight);
        }

        return maybe_this;
    }

    /// 处理股指期货
    fn check_stock_index(
        &self,
        opt_current: Option<&'a SimpleMktData>,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        // 注意: 股指期货的要求是逐月换, 中间不跨越
        // 即不从IF1901换到IF1903，即使IF1903的权重大于IF1902

        // 确保当前合约非空,
        // 注意, 这里没有根据normal规则往后推,
        // 只有opt为空时, 才选个权重大的
        let maybe_this = match opt_current {
            Some(current) => current,
            None => self.get_next_normally(None, ordby_expire, ordby_weight),
        };
        let must_exit_date = FinanceRolling::calc_must_exit_date(
            self.tdmgr,
            &maybe_this.expire_date,
            ProductExitWeek::StockIndex,
        );
        if must_exit_date <= self.test_day {
            // 强制换月, 直接换到后面一个月, 不管weight权重, 注意ordered_vec是日期排序的ordby_expire
            return self.get_next_by_expire_date(maybe_this, ordby_expire);
        }

        return maybe_this;
    }

    /// 处理国债期货
    fn check_bonds(
        &self,
        opt_current: Option<&'a SimpleMktData>,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        let mut maybe_this = self.get_next_normally(opt_current, ordby_expire, ordby_weight);

        let must_exit_date = FinanceRolling::calc_must_exit_date(
            self.tdmgr,
            &maybe_this.expire_date,
            ProductExitWeek::Bonds,
        );
        if must_exit_date <= self.test_day {
            maybe_this = self.fore_rolling_check(maybe_this, ordby_expire, ordby_weight);
        }
        return maybe_this;
    }

    /// 处理其他普通品种, 非股指, 非国债,非原油
    fn check_others(
        &self,
        opt_current: Option<&'a SimpleMktData>,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        let mut maybe_this = self.get_next_normally(opt_current, ordby_expire, ordby_weight);

        let must_exit_date =
            NormalRolling::calc_must_exit_date(self.tdmgr, &maybe_this.expire_date);
        if must_exit_date <= self.test_day {
            maybe_this = self.fore_rolling_check(maybe_this, ordby_expire, ordby_weight);
        }
        return maybe_this;
    }

    /// 常规处理获取下月, 满足权重1.1倍则向后切换, 不开倒车; opt_current为空则取当前权重最大者; 不对月份日期有其他要求
    /// 注意: 有可能是不改变, 将opt_current原路返回
    fn get_next_normally(
        &self,
        opt_current: Option<&'a SimpleMktData>,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        let result_item = match opt_current {
            Some(mut current) => {
                // 寻找比current更远的月份，并且权重满足1.1的要求
                for item in ordby_expire {
                    if SimpleMktData::is_rolling_needed(current, item) {
                        current = item;
                        break;
                    }
                }
                current
            }
            // 取当前权重最大的合约
            None => &ordby_weight[0],
        };
        return result_item;
    }

    /// 选出expire_date更大的下一个月, 不开倒车
    /// 根据输入的ordered_vec, 既可以是权重排序, 也可以是日期排序的
    fn get_next_by_expire_date(
        &self,
        current: &'a SimpleMktData,
        ordered_vec: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        for item in ordered_vec {
            if item.expire_date > current.expire_date {
                return item;
            }
        }
        return current;
    }

    fn fore_rolling_check(
        &self,
        maybe_this: &'a SimpleMktData,
        ordby_expire: &'a Vec<&SimpleMktData>,
        ordby_weight: &'a Vec<&SimpleMktData>,
    ) -> &'a SimpleMktData {
        // 从理论上说, 这里有个问题, 比如当前主力为1月, 而3月和5月的weight同时满足1.1倍, 且5月更高,
        // 如果仅按权重处理, 则会切换到(最大的)5月, 但显然3月才是正确的选项,
        // 特别是不活跃的品种, 活跃品种一般不会出现这个情况

        // 所以我们从正常方式取得的`下月` 和 权重排序取得的下月 里面, 选取一个日期较近的
        let mut retitem = self.get_next_by_expire_date(maybe_this, ordby_weight);
        let normal = self.get_next_normally(Some(maybe_this), ordby_expire, ordby_weight);
        if normal.inst == maybe_this.inst {
            // normal就是maybe_this本身, 并没有取到`下月`
            // 则直接返回retitem
            return retitem;
        }

        retitem = if retitem.expire_date < normal.expire_date {
            retitem
        } else {
            normal
        };
        return retitem;
    }
}
