use chrono::{Datelike, Local, Timelike, Weekday};

/// 竞价时序检查器
///
/// 负责判断当前是否在竞价时段（9:15-9:25）
pub struct AuctionTimeChecker;

impl AuctionTimeChecker {
    pub fn new() -> Self {
        Self
    }

    /// 检查当前是否在竞价时段（9:15-9:25）
    pub fn is_auction_time(&self) -> bool {
        let now = Local::now();

        // 只在交易日运行（周一到周五）
        if now.weekday() == Weekday::Sat || now.weekday() == Weekday::Sun {
            return false;
        }

        let hour = now.hour();
        let minute = now.minute();

        // 竞价时段：9:15-9:25
        hour == 9 && minute >= 15 && minute < 25
    }

    /// 获取下一次竞价开始时间（秒）
    pub fn seconds_until_auction(&self) -> Option<u64> {
        let now = Local::now();

        // 周末不计算
        if now.weekday() == Weekday::Sat || now.weekday() == Weekday::Sun {
            return None;
        }

        let hour = now.hour();
        let minute = now.minute();

        // 9:15之前
        if hour < 9 || (hour == 9 && minute < 15) {
            let target = now
                .date_naive()
                .and_hms_opt(9, 15, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();
            let duration = target - now;
            return Some(duration.num_seconds().max(0) as u64);
        }

        // 9:15-9:25之间，立即执行
        if hour == 9 && minute >= 15 && minute < 25 {
            return Some(0);
        }

        // 9:25之后，等待下一天
        None
    }
}

impl Default for AuctionTimeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auction_time_checker_creation() {
        let checker = AuctionTimeChecker::new();
        // 验证检查器创建成功
    }

    #[test]
    fn test_is_auction_time_returns_bool() {
        let checker = AuctionTimeChecker::new();
        // 测试能返回布尔值（具体值取决于测试时间）
        let result = checker.is_auction_time();
        // result是true或false
    }
}
