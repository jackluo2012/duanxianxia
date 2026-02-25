import request from './request';
import type {
  VoiceNewsQuery,
  HotNewsQuery,
  VoiceNewsListResponse,
  VoiceNewsTimelineResponse,
  HotNewsListResponse,
  HotNewsDetailResponse,
  NewsStatistics,
} from '../types/news';

// ========== 语音快讯 ==========

// 获取语音快讯列表（分页）
export const getVoiceNewsList = (params: VoiceNewsQuery): Promise<VoiceNewsListResponse> => {
  return request.get<VoiceNewsListResponse>('/api/news/voice', { params });
};

// 获取语音快讯时间线（按日期分组）
export const getVoiceNewsTimeline = (params: {
  date_range?: [string, string];
  page_size?: number;
}): Promise<VoiceNewsTimelineResponse> => {
  return request.get<VoiceNewsTimelineResponse>('/api/news/voice/timeline', { params });
};

// 获取语音快讯详情
export const getVoiceNewsDetail = (id: string): Promise<any> => {
  return request.get<any>(`/api/news/voice/${id}`);
};

// 增加语音快讯浏览次数
export const incrementVoiceNewsViews = (id: string): Promise<{ views: number }> => {
  return request.post<{ views: number }>(`/api/news/voice/${id}/view`);
};

// ========== 热点聚焦 ==========

// 获取热点新闻列表（分页）
export const getHotNewsList = (params: HotNewsQuery): Promise<HotNewsListResponse> => {
  return request.get<HotNewsListResponse>('/api/news/hot', { params });
};

// 获取热点新闻详情
export const getHotNewsDetail = (id: string): Promise<HotNewsDetailResponse> => {
  return request.get<HotNewsDetailResponse>(`/api/news/hot/${id}`);
};

// 增加热点新闻浏览次数
export const incrementHotNewsViews = (id: string): Promise<{ views: number }> => {
  return request.post<{ views: number }>(`/api/news/hot/${id}/view`);
};

// 点赞热点新闻
export const likeHotNews = (id: string): Promise<{ likes: number }> => {
  return request.post<{ likes: number }>(`/api/news/hot/${id}/like`);
};

// 取消点赞热点新闻
export const unlikeHotNews = (id: string): Promise<{ likes: number }> => {
  return request.post<{ likes: number }>(`/api/news/hot/${id}/unlike`);
};

// ========== 通用 ==========

// 获取资讯统计信息
export const getNewsStatistics = (): Promise<NewsStatistics> => {
  return request.get<NewsStatistics>('/api/news/statistics');
};

// 搜索资讯（语音快讯和热点新闻）
export const searchNews = (
  keyword: string,
  type: 'voice' | 'hot' | 'all',
  page: number = 1,
  page_size: number = 20
): Promise<{
  voice: VoiceNewsListResponse;
  hot: HotNewsListResponse;
}> => {
  return request.get('/api/news/search', {
    params: { keyword, type, page, page_size }
  });
};
