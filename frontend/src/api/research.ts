import request from './request';
import type {
  ResearchQuery,
  ResearchListResponse,
  ResearchDetailResponse,
  ResearchStatistics,
  ResearchFilterOptions,
} from '../types/research';

// 获取研报列表
export const getResearchList = (params: ResearchQuery): Promise<ResearchListResponse> => {
  return request.get<ResearchListResponse>('/api/research/reports', { params });
};

// 获取研报详情
export const getResearchDetail = (id: string): Promise<ResearchDetailResponse> => {
  return request.get<ResearchDetailResponse>(`/api/research/reports/${id}`);
};

// 获取研报统计信息
export const getResearchStatistics = (): Promise<ResearchStatistics> => {
  return request.get<ResearchStatistics>('/api/research/statistics');
};

// 获取筛选选项
export const getResearchFilterOptions = (): Promise<ResearchFilterOptions> => {
  return request.get<ResearchFilterOptions>('/api/research/filter-options');
};

// 搜索研报（支持关键词、股票代码、股票名称）
export const searchResearch = (keyword: string, page: number = 1, page_size: number = 20): Promise<ResearchListResponse> => {
  return request.get<ResearchListResponse>('/api/research/search', {
    params: { keyword, page, page_size }
  });
};

// 增加研报浏览次数
export const incrementResearchViews = (id: string): Promise<{ views: number }> => {
  return request.post<{ views: number }>(`/api/research/reports/${id}/view`);
};

// 下载研报PDF
export const downloadResearchPDF = (id: string): Promise<Blob> => {
  return request.get<Blob>(`/api/research/reports/${id}/download`, {
    responseType: 'blob'
  });
};
