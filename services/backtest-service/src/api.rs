use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::{BacktestRequest, BacktestResult, BacktestError, StrategyType};
use crate::engine::BacktestEngine;
use crate::metrics::{BacktestTimer, update_queue_metrics, record_capital_metrics, record_trade_metrics};

/// 回测任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacktestStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// 回测任务
#[derive(Debug, Clone, Serialize)]
pub struct BacktestTask {
    pub backtest_id: String,
    pub status: BacktestStatus,
    pub result: Option<BacktestResult>,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 任务管理器
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<std::collections::HashMap<String, BacktestTask>>>,
    engine: Arc<RwLock<BacktestEngine>>,
}

impl TaskManager {
    pub fn new(clickhouse_url: &str) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            engine: Arc::new(RwLock::new(BacktestEngine::new(clickhouse_url))),
        }
    }

    /// 提交新的回测任务
    pub async fn submit_backtest(&self, request: BacktestRequest) -> Result<String, BacktestError> {
        let backtest_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let strategy_name = format!("{:?}", request.strategy_type);

        // 创建任务
        let task = BacktestTask {
            backtest_id: backtest_id.clone(),
            status: BacktestStatus::Pending,
            result: None,
            error: None,
            created_at: now,
        };

        // 保存任务
        self.tasks.write().await.insert(backtest_id.clone(), task);

        // 异步执行回测
        let tasks = self.tasks.clone();
        let engine = self.engine.clone();
        let id_clone = backtest_id.clone();

        tokio::spawn(async move {
            // 启动计时器
            let timer = BacktestTimer::new(strategy_name.clone());

            // 更新状态为运行中
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(task) = tasks_guard.get_mut(&id_clone) {
                    task.status = BacktestStatus::Running;
                }
                // 更新队列指标
                let all_tasks = tasks_guard.values().collect::<Vec<_>>();
                let pending = all_tasks.iter().filter(|t| matches!(t.status, BacktestStatus::Pending)).count() as i64;
                let running = all_tasks.iter().filter(|t| matches!(t.status, BacktestStatus::Running)).count() as i64;
                let completed = all_tasks.iter().filter(|t| matches!(t.status, BacktestStatus::Completed)).count() as i64;
                drop(tasks_guard);
                update_queue_metrics(pending, running, completed);
            }

            // 执行回测
            let mut engine_guard = engine.write().await;
            let result = engine_guard.run(request).await;

            // 更新任务状态
            let mut tasks_guard = tasks.write().await;
            if let Some(task) = tasks_guard.get_mut(&id_clone) {
                match result {
                    Ok(backtest_result) => {
                        task.status = BacktestStatus::Completed;

                        // 记录回测指标
                        let perf = &backtest_result.performance;
                        record_capital_metrics(
                            backtest_result.request.initial_capital,
                            perf.final_capital,
                            perf.total_return
                        );
                        record_trade_metrics(
                            perf.trade_count as i64,
                            perf.win_rate,
                            perf.profit_loss_ratio
                        );

                        timer.finish();
                        task.result = Some(backtest_result);
                    },
                    Err(e) => {
                        task.status = BacktestStatus::Failed;
                        task.error = Some(e.to_string());
                        timer.finish_with_error("backtest_error");
                    }
                }
            }
        });

        Ok(backtest_id)
    }

    /// 获取任务状态
    pub async fn get_task(&self, backtest_id: &str) -> Option<BacktestTask> {
        self.tasks.read().await.get(backtest_id).cloned()
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<BacktestTask> {
        self.tasks.read().await.values().cloned().collect()
    }
}

/// 启动回测请求
#[derive(Deserialize)]
pub struct StartBacktestRequest {
    pub strategy_type: StrategyType,
    pub strategy_params: crate::models::StrategyParams,
    pub backtest_period: crate::models::BacktestPeriod,
    pub initial_capital: f64,
    pub commission_rate: Option<f64>,
}

impl From<StartBacktestRequest> for BacktestRequest {
    fn from(req: StartBacktestRequest) -> Self {
        Self {
            strategy_type: req.strategy_type,
            strategy_params: req.strategy_params,
            backtest_period: req.backtest_period,
            initial_capital: req.initial_capital,
            commission_rate: req.commission_rate.unwrap_or(0.0003),
        }
    }
}

/// 启动回测响应
#[derive(Serialize)]
pub struct StartBacktestResponse {
    pub backtest_id: String,
    pub status: BacktestStatus,
    pub estimated_time: u64, // 秒
}

/// POST /api/backtest/run
pub async fn start_backtest(
    req: web::Json<StartBacktestRequest>,
    task_manager: web::Data<TaskManager>,
) -> impl Responder {
    let request: BacktestRequest = req.into_inner().into();

    // 验证请求
    if let Err(e) = request.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        }));
    }

    // 提交任务
    match task_manager.submit_backtest(request).await {
        Ok(backtest_id) => {
            HttpResponse::Accepted().json(StartBacktestResponse {
                backtest_id,
                status: BacktestStatus::Running,
                estimated_time: 30, // 预估30秒
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// GET /api/backtest/{backtest_id}
pub async fn get_backtest_result(
    path: web::Path<String>,
    task_manager: web::Data<TaskManager>,
) -> impl Responder {
    let backtest_id = path.into_inner();

    match task_manager.get_task(&backtest_id).await {
        Some(task) => {
            HttpResponse::Ok().json(task)
        },
        None => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "回测任务不存在"
            }))
        }
    }
}

/// 策略信息
#[derive(Serialize)]
pub struct StrategyInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub params: Vec<ParamInfo>,
}

#[derive(Serialize)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: serde_json::Value,
    pub description: String,
}

/// GET /api/backtest/strategies
pub async fn get_strategies() -> impl Responder {
    let strategies = vec![
        StrategyInfo {
            id: "auction_leader".to_string(),
            name: "竞价龙头策略".to_string(),
            description: "竞价强度评分>80且买封金额>1000万".to_string(),
            params: vec![
                ParamInfo {
                    name: "min_strength_score".to_string(),
                    param_type: "integer".to_string(),
                    default: serde_json::json!(80),
                    description: "最低强度评分 (0-100)".to_string(),
                },
                ParamInfo {
                    name: "min_buy_seal_amount".to_string(),
                    param_type: "float".to_string(),
                    default: serde_json::json!(1000),
                    description: "最低买封金额 (万)".to_string(),
                },
                ParamInfo {
                    name: "holding_days".to_string(),
                    param_type: "integer".to_string(),
                    default: serde_json::json!(1),
                    description: "持仓天数 (1-10)".to_string(),
                },
            ],
        },
        StrategyInfo {
            id: "auction_seal".to_string(),
            name: "竞价封单策略".to_string(),
            description: "买封金额排名前N且涨幅<5%".to_string(),
            params: vec![
                ParamInfo {
                    name: "top_n".to_string(),
                    param_type: "integer".to_string(),
                    default: serde_json::json!(10),
                    description: "排名前N (1-50)".to_string(),
                },
                ParamInfo {
                    name: "holding_days".to_string(),
                    param_type: "integer".to_string(),
                    default: serde_json::json!(3),
                    description: "持仓天数 (1-10)".to_string(),
                },
            ],
        },
        StrategyInfo {
            id: "intraday_breakout".to_string(),
            name: "盘中突破策略".to_string(),
            description: "突破前高+成交量放大2倍 (待实现)".to_string(),
            params: vec![
                ParamInfo {
                    name: "volume_multiplier".to_string(),
                    param_type: "float".to_string(),
                    default: serde_json::json!(2),
                    description: "成交量放大倍数 (1.5-5)".to_string(),
                },
            ],
        },
    ];

    HttpResponse::Ok().json(serde_json::json!({
        "strategies": strategies
    }))
}

/// GET /api/backtest/history
pub async fn get_backtest_history(
    query: web::Query<std::collections::HashMap<String, String>>,
    task_manager: web::Data<TaskManager>,
) -> impl Responder {
    let page: usize = query.get("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let page_size: usize = query.get("page_size")
        .and_then(|p| p.parse().ok())
        .unwrap_or(10);

    let all_tasks = task_manager.get_all_tasks().await;

    // 按创建时间倒序排序
    let mut sorted_tasks: Vec<_> = all_tasks.into_iter().collect();
    sorted_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = sorted_tasks.len();
    let start = (page - 1) * page_size;
    let end = std::cmp::min(start + page_size, total);
    let items: Vec<_> = sorted_tasks.into_iter()
        .skip(start)
        .take(end - start)
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "total": total,
        "page": page,
        "page_size": page_size,
        "items": items
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BacktestPeriod, StrategyParams};
    use chrono::NaiveDate;

    #[test]
    fn test_start_request_conversion() {
        let req = StartBacktestRequest {
            strategy_type: StrategyType::AuctionLeader,
            strategy_params: StrategyParams::default(),
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 10, 31).unwrap(),
            },
            initial_capital: 100000.0,
            commission_rate: Some(0.0003),
        };

        let backtest_req: BacktestRequest = req.into();
        assert_eq!(backtest_req.commission_rate, 0.0003);
    }

    #[test]
    fn test_start_request_default_commission() {
        let req = StartBacktestRequest {
            strategy_type: StrategyType::AuctionLeader,
            strategy_params: StrategyParams::default(),
            backtest_period: BacktestPeriod {
                start_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 10, 31).unwrap(),
            },
            initial_capital: 100000.0,
            commission_rate: None,
        };

        let backtest_req: BacktestRequest = req.into();
        assert_eq!(backtest_req.commission_rate, 0.0003); // 默认值
    }
}
