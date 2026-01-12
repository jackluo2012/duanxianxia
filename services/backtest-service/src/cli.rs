use crate::models::{BacktestRequest, StrategyType, StrategyParams, BacktestPeriod};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};

/// Backtest Service CLI
#[derive(Parser)]
#[command(name = "backtest-cli")]
#[command(about = "命令行回测工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行回测
    Run {
        /// 策略类型
        #[arg(short, long)]
        strategy: String,

        /// 开始日期
        #[arg(short, long)]
        start_date: String,

        /// 结束日期
        #[arg(short, long)]
        end_date: String,

        /// 初始资金
        #[arg(short, long, default_value_t = 100000.0)]
        capital: f64,

        /// 强度评分
        #[arg(long)]
        strength_score: Option<i32>,

        /// 买封金额
        #[arg(long)]
        seal_amount: Option<f64>,

        /// 持仓天数
        #[arg(short, long, default_value_t = 1)]
        holding_days: i32,

        /// 手续费率
        #[arg(short, long, default_value_t = 0.0003)]
        commission: f64,

        /// 输出格式 (json/table)
        #[arg(short, long, default_value = "json")]
        output: String,
    },
    /// 策略列表
    List,
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            strategy,
            start_date,
            end_date,
            capital,
            strength_score,
            seal_amount,
            holding_days,
            commission,
            output,
        } => {
            run_backtest(
                strategy,
                start_date,
                end_date,
                capital,
                strength_score,
                seal_amount,
                holding_days,
                commission,
                output,
            ).await?
        },
        Commands::List => {
            list_strategies();
        }
    }

    Ok(())
}

async fn run_backtest(
    strategy: String,
    start_date: String,
    end_date: String,
    capital: f64,
    strength_score: Option<i32>,
    seal_amount: Option<f64>,
    holding_days: i32,
    commission: f64,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::engine::BacktestEngine;

    println!("🚀 启动回测...");
    println!("策略: {}", strategy);
    println!("时间: {} 至 {}", start_date, end_date);
    println!("资金: {:.2}", capital);
    println!();

    // 解析日期
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")?;

    // 构建请求
    let strategy_type = match strategy.as_str() {
        "auction_leader" => StrategyType::AuctionLeader,
        "auction_seal" => StrategyType::AuctionSeal,
        "intraday_breakout" => StrategyType::IntradayBreakout,
        _ => return Err("未知策略类型".into()),
    };

    let mut params = StrategyParams::default();
    params.holding_days = Some(holding_days);
    params.min_strength_score = strength_score;
    params.min_buy_seal_amount = seal_amount;

    let request = BacktestRequest {
        strategy_type,
        strategy_params: params,
        backtest_period: BacktestPeriod {
            start_date: start,
            end_date: end,
        },
        initial_capital: capital,
        commission_rate: commission,
    };

    // 运行回测
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());

    let mut engine = BacktestEngine::new(&clickhouse_url);

    println!("⏳ 正在运行回测...");
    let result = engine.run(request).await;

    match result {
        Ok(backtest_result) => {
            println!();
            println!("✅ 回测完成!");
            println!();

            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&backtest_result)?);
            } else {
                print_table(&backtest_result);
            }
        },
        Err(e) => {
            println!();
            println!("❌ 回测失败: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn list_strategies() {
    println!("可用策略:");
    println!();
    println!("1. auction_leader - 竞价龙头策略");
    println!("   条件: 竞价强度评分>80 且 买封金额>1000万");
    println!("   参数: --strength-score --seal-amount --holding-days");
    println!();
    println!("2. auction_seal - 竞价封单策略");
    println!("   条件: 买封金额排名前N 且 涨幅<5%");
    println!("   参数: --holding-days");
    println!();
    println!("3. intraday_breakout - 盘中突破策略 (待实现)");
    println!();
}

fn print_table(result: &crate::models::BacktestResult) {
    let perf = &result.performance;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  回测结果");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("📊 收益指标:");
    println!("  总收益率:     {:.2}%", perf.total_return * 100.0);
    println!("  年化收益率:   {:.2}%", perf.annualized_return * 100.0);
    println!("  胜率:         {:.2}%", perf.win_rate * 100.0);
    println!("  盈亏比:       {:.2}", perf.profit_loss_ratio);
    println!();

    println!("💰 资金指标:");
    println!("  初始资金:     {:.2}", result.request.initial_capital);
    println!("  最终资金:     {:.2}", perf.final_capital);
    println!("  总盈利:       {:.2}", perf.total_profit);
    println!("  总亏损:       {:.2}", perf.total_loss);
    println!();

    println!("⚡ 交易效率:");
    println!("  交易次数:     {}", perf.trade_count);
    println!("  平均持仓天数: {:.1}", perf.avg_holding_days);
    println!("  换手率:       {:.2}%", perf.turnover_rate * 100.0);
    println!();

    println!("⚠️  风险指标:");
    println!("  最大回撤:     {:.2}%", perf.max_drawdown * 100.0);
    println!("  波动率:       {:.2}", perf.volatility);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_parsing() {
        // 策略类型解析测试
        let strategies = vec!["auction_leader", "auction_seal", "intraday_breakout"];
        for strategy in strategies {
            match strategy {
                "auction_leader" => StrategyType::AuctionLeader,
                "auction_seal" => StrategyType::AuctionSeal,
                "intraday_breakout" => StrategyType::IntradayBreakout,
                _ => panic!("Unknown strategy"),
            };
        }
    }
}
