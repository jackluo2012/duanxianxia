// ===================================================================
// 测试 TDX 原始数据返回
// ===================================================================

use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};

fn main() {
    println!("🔍 测试 TDX API 原始数据返回\n");

    // 创建 TCP 连接
    let mut tcp = match Tcp::new() {
        Ok(t) => {
            println!("✅ TDX TCP 连接成功");
            t
        }
        Err(e) => {
            eprintln!("❌ TDX TCP 连接失败: {}", e);
            return;
        }
    };

    // 测试股票代码
    let test_codes = vec![
        (1, "600000"), // 浦发银行
        (1, "600036"), // 招商银行
        (0, "000001"), // 平安银行
        (0, "000002"), // 万科A
    ];

    println!("\n📊 获取股票行情数据:");
    println!("{:=<60}", "");

    for (market, code) in test_codes {
        println!("\n股票代码: {} (市场: {})", code, if market == 1 { "上海" } else { "深圳" });

        let mut quotes = SecurityQuotes::new(vec![(market, code)]);
        match quotes.recv_parsed(&mut tcp) {
            Ok(_) => {
                if let Some(q) = quotes.result().first() {
                    println!("  ✅ 数据获取成功:");
                    println!("     - code: {:?}", q.code);
                    println!("     - name: {:?}", q.name);
                    println!("     - price: {}", q.price);
                    println!("     - preclose: {}", q.preclose);
                    println!("     - open: {}", q.open);
                    println!("     - high: {}", q.high);
                    println!("     - low: {}", q.low);
                    println!("     - volume: {}", q.vol);
                    println!("     - amount: {}", q.amount);

                    // 检查关键字段是否有效
                    if q.preclose == 0.0 {
                        println!("  ⚠️  警告: preclose 为 0!");
                    }
                    if q.name.is_empty() {
                        println!("  ⚠️  警告: name 为空!");
                    }
                } else {
                    println!("  ❌ 结果为空");
                }
            }
            Err(e) => {
                println!("  ❌ 获取失败: {}", e);
            }
        }
    }

    println!("\n{:=<60}", "");
    println!("\n💡 结论:");
    println!("如果 preclose 和 name 都为空/0，说明 TDX 服务器没有返回这些数据。");
    println!("可能需要:");
    println!("  1. 使用不同的 TDX 服务器");
    println!("  2. 从历史K线数据中获取昨收价");
    println!("  3. 使用其他数据源补充这些字段");
}
