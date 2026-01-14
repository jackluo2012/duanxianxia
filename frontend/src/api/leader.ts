import request from './request';
import type {
  LeaderBoardResponse,
  LeaderDetail,
  LeaderStock,
  LeaderFilters
} from '../types/leader';

// 获取连板排行榜
export const getLeaderBoard = (params: LeaderFilters): Promise<LeaderBoardResponse> => {
  return request.get<LeaderBoardResponse>('/review/leader-board', { params });
};

// 获取股票详情
export const getLeaderDetail = (code: string): Promise<LeaderDetail> => {
  return request.get<LeaderDetail>(`/review/leader-detail`, {
    params: { code }
  });
};

// 搜索股票(用于对比功能)
export const searchStocks = (keyword: string): Promise<LeaderStock[]> => {
  return request.get<LeaderStock[]>('/search/stocks', {
    params: { q: keyword }
  });
};
