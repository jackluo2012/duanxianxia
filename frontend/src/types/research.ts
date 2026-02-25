// 研报类型定义

// 日期字符串类型 (ISO 8601格式: YYYY-MM-DD)
export type DateString = string;

// 研究机构类型
export enum InstituteType {
  Securities = 'securities',  // 券商
  Fund = 'fund',              // 基金
  Other = 'other',            // 其他
}

// 报告类型
export enum ReportType {
  Research = 'research',      // 研究报告
  Comment = 'comment',        // 点评报告
  Deep = 'deep',              // 深度报告
  Industry = 'industry',      // 行业报告
  Strategy = 'strategy',      // 策略报告
}

// 评级类型
export enum RatingType {
  Buy = 'buy',                // 买入
  Overweight = 'overweight',  // 增持
  Neutral = 'neutral',        // 中性
  Underweight = 'underweight',// 减持
  Sell = 'sell',              // 卖出
}

// 研报基本信息
export interface ResearchReport {
  id: string;                  // 研报ID
  title: string;               // 报告标题
  institute: string;           // 研究机构名称
  institute_type: InstituteType; // 机构类型
  analyst: string;             // 分析师姓名
  report_type: ReportType;     // 报告类型
  rating: RatingType;          // 评级
  target_price?: number;       // 目标价格
  stock_code: string;          // 股票代码
  stock_name: string;          // 股票名称
  sector: string;              // 所属板块
  publish_date: DateString;    // 发布日期
  summary: string;             // 摘要
  content?: string;            // 完整内容（详情时返回）
  pdf_url?: string;            // PDF报告链接
  views: number;               // 浏览次数
  created_at: DateString;      // 创建时间
  updated_at: DateString;      // 更新时间
}

// 研报查询参数
export interface ResearchQuery {
  keyword?: string;            // 关键词搜索（标题/摘要/股票名称）
  stock_code?: string;         // 股票代码筛选
  sector?: string;             // 板块筛选
  institute?: string;          // 机构筛选
  institute_type?: InstituteType; // 机构类型筛选
  analyst?: string;            // 分析师筛选
  report_type?: ReportType;    // 报告类型筛选
  rating?: RatingType;         // 评级筛选
  date_range?: [DateString, DateString]; // 发布日期范围
  page: number;                // 页码（从1开始）
  page_size: number;           // 每页数量
  sort_by?: 'publish_date' | 'views' | 'created_at'; // 排序字段
  sort_order?: 'asc' | 'desc'; // 排序方向
}

// 研报列表响应
export interface ResearchListResponse {
  total: number;               // 总数量
  page: number;                // 当前页码
  page_size: number;           // 每页数量
  items: ResearchReport[];     // 研报列表
}

// 研报详情响应
export type ResearchDetailResponse = ResearchReport;

// 统计信息
export interface ResearchStatistics {
  total_reports: number;       // 总研报数量
  today_reports: number;       // 今日新增
  institute_count: number;     // 机构数量
  hot_sectors: string[];       // 热门板块
}

// 筛选选项
export interface ResearchFilterOptions {
  institutes: string[];        // 机构列表
  analysts: string[];          // 分析师列表
  sectors: string[];           // 板块列表
  report_types: ReportType[];  // 报告类型列表
  rating_types: RatingType[];  // 评级类型列表
}
