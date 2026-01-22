use anyhow::Result;
use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::services::{ConsecutiveCalculator, ReviewTableGenerator};

/// 复盘生成用例
pub struct ReviewGenerationUseCase {
    review_generator: Arc<ReviewTableGenerator>,
    consecutive_calculator: Arc<ConsecutiveCalculator>,
}

impl ReviewGenerationUseCase {
    pub fn new(
        review_generator: Arc<ReviewTableGenerator>,
        consecutive_calculator: Arc<ConsecutiveCalculator>,
    ) -> Self {
        Self {
            review_generator,
            consecutive_calculator,
        }
    }

    /// 生成每日复盘
    pub async fn generate_daily_review(&self, date: NaiveDate) -> Result<usize> {
        self.review_generator.generate_daily_review(date).await
    }

    /// 生成人工待标注列表
    pub async fn generate_annotation_queue(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<crate::domain::services::AnnotationItem>> {
        self.review_generator.generate_annotation_queue(date).await
    }
}
