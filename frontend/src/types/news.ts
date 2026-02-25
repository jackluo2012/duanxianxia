// 资讯类型定义

// 日期字符串类型 (ISO 8601格式: YYYY-MM-DD)
export type DateString = string;

// 日期时间字符串类型 (ISO 8601格式: YYYY-MM-DD HH:mm:ss)
export type DateTimeString = string;

// 资讯类型
export enum NewsType {
  Voice = 'voice',             // 语音快讯
  Hot = 'hot',                 // 热点聚焦
  Flash = 'flash',             // 7x24快讯
}

// 热度等级
export enum HotLevel {
  High = 'high',               // 高热度
  Medium = 'medium',           // 中热度
  Low = 'low',                 // 低热度
}

// 资讯分类
export enum NewsCategory {
  Policy = 'policy',           // 政策
  Market = 'market',           // 市场
  Company = 'company',         // 公司
  Sector = 'sector',           // 行业
  International = 'international', // 国际
  Technology = 'technology',   // 科技
}

// 语音快讯
export interface VoiceNews {
  id: string;                  // 快讯ID
  title: string;               // 标题
  content: string;             // 内容
  audio_url: string;           // 音频链接
  duration: number;            // 音频时长（秒）
  publish_time: DateTimeString; // 发布时间
  source: string;              // 来源
  category: NewsCategory;      // 分类
  tags: string[];              // 标签
  views: number;               // 浏览次数
  created_at: DateTimeString;  // 创建时间
}

// 热点新闻
export interface HotNews {
  id: string;                  // 新闻ID
  title: string;               // 标题
  summary: string;             // 摘要
  content?: string;            // 完整内容（详情时返回）
  cover_image?: string;        // 封面图片
  hot_level: HotLevel;         // 热度等级
  category: NewsCategory;      // 分类
  tags: string[];              // 标签
  related_stocks?: string[];   // 相关股票代码
  publish_time: DateTimeString; // 发布时间
  source: string;              // 来源
  author?: string;             // 作者
  views: number;               // 浏览次数
  likes: number;               // 点赞数
  comments_count: number;      // 评论数
  created_at: DateTimeString;  // 创建时间
  updated_at: DateTimeString;  // 更新时间
}

// 时间线分组（用于语音快讯按日期分组）
export interface TimelineGroup {
  date: DateString;            // 日期
  items: VoiceNews[];          // 该日期的快讯列表
}

// 语音快讯查询参数
export interface VoiceNewsQuery {
  date?: DateString;           // 按日期查询
  date_range?: [DateString, DateString]; // 日期范围
  category?: NewsCategory;     // 分类筛选
  keyword?: string;            // 关键词搜索
  page: number;                // 页码（从1开始）
  page_size: number;           // 每页数量
}

// 热点新闻查询参数
export interface HotNewsQuery {
  category?: NewsCategory;     // 分类筛选
  hot_level?: HotLevel;        // 热度等级筛选
  keyword?: string;            // 关键词搜索
  tag?: string;                // 标签筛选
  date_range?: [DateString, DateString]; // 日期范围
  page: number;                // 页码（从1开始）
  page_size: number;           // 每页数量
  sort_by?: 'publish_time' | 'views' | 'likes'; // 排序字段
  sort_order?: 'asc' | 'desc'; // 排序方向
}

// 语音快讯列表响应
export interface VoiceNewsListResponse {
  total: number;               // 总数量
  page: number;                // 当前页码
  page_size: number;           // 每页数量
  items: VoiceNews[];          // 快讯列表
}

// 语音快讯时间线响应
export interface VoiceNewsTimelineResponse {
  total: number;               // 总数量
  groups: TimelineGroup[];     // 按日期分组的时间线
}

// 热点新闻列表响应
export interface HotNewsListResponse {
  total: number;               // 总数量
  page: number;                // 当前页码
  page_size: number;           // 每页数量
  items: HotNews[];            // 新闻列表
}

// 热点新闻详情响应
export type HotNewsDetailResponse = HotNews;

// 资讯统计信息
export interface NewsStatistics {
  today_voice_count: number;   // 今日语音快讯数量
  today_hot_count: number;     // 今日热点新闻数量
  total_voice_count: number;   // 语音快讯总数
  total_hot_count: number;     // 热点新闻总数
  hot_categories: NewsCategory[]; // 热门分类
  hot_tags: string[];          // 热门标签
}
