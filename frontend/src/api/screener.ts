/**
 * 个股挖掘 API 接口
 * 提供龙头高度、连板统计、涨跌停等数据
 */

import request from './request';
import { config } from '../config';

// ============================================
// 类型定义
// ============================================

export interface LeaderItem {
  code: string;
  name: string;
  sector: string;
  leader_height: number;
  sector_rank: number;
  total_stocks: number;
  price: number;
  change_percent: number;
  amount: number;
}

export interface ConsecutiveBoardItem {
  code: string;
  name: string;
  sector: string;
  consecutive_days: number;
  start_date: string;
  end_date: string;
  board_type: string;
  current_price: number;
  reason: string;
}

export interface LimitItem {
  code: string;
  name: string;
  sector: string;
  limit_type: string;
  limit_time: string;
  limit_price: number;
  volume: number;
  amount: number;
  reason: string;
  is_first: boolean;
}

// ============================================
// API 函数
// ============================================

/**
 * 获取龙头高度排行榜
 * @param sector 板块代码（可选）
 * @param limit 返回数量限制
 * @returns 龙头高度列表
 */
export async function fetchLeaders(
  sector?: string,
  limit: number = 50
): Promise<LeaderItem[]> {
  const params = new URLSearchParams();
  if (sector) params.append('sector', sector);
  params.append('limit', limit.toString());

  return request.get(`${config.storageUrl}/api/screener/leaders?${params.toString()}`);
}

/**
 * 获取连板统计数据
 * @param minDays 最小连板天数
 * @param boardType 连板类型："连涨" 或 "连跌"
 * @param limit 返回数量限制
 * @returns 连板统计列表
 */
export async function fetchConsecutiveBoards(
  minDays: number = 2,
  boardType: string = '连涨',
  limit: number = 50
): Promise<ConsecutiveBoardItem[]> {
  return request.get(
    `${config.storageUrl}/api/screener/consecutive?min_days=${minDays}&board_type=${boardType}&limit=${limit}`
  );
}

/**
 * 获取涨停股票列表
 * @param date 日期（"today" 或具体日期）
 * @param limit 返回数量限制
 * @returns 涨停股票列表
 */
export async function fetchLimitUp(
  date: string = 'today',
  limit: number = 50
): Promise<LimitItem[]> {
  return request.get(`${config.storageUrl}/api/screener/limit-up?date=${date}&limit=${limit}`);
}

/**
 * 获取跌停股票列表
 * @param date 日期（"today" 或具体日期）
 * @param limit 返回数量限制
 * @returns 跌停股票列表
 */
export async function fetchLimitDown(
  date: string = 'today',
  limit: number = 50
): Promise<LimitItem[]> {
  return request.get(`${config.storageUrl}/api/screener/limit-down?date=${date}&limit=${limit}`);
}

/**
 * 实时计算龙头高度
 * @param sectorCode 板块代码
 * @returns 龙头高度列表
 */
export async function fetchLeadersRealtime(
  sectorCode: string
): Promise<LeaderItem[]> {
  return request.get(`${config.storageUrl}/api/screener/leaders-realtime?sector_code=${sectorCode}`);
}

/**
 * 实时计算连板天数
 * @param code 股票代码
 * @returns 连板天数
 */
export async function fetchConsecutiveRealtime(code: string): Promise<number> {
  return request.get(`${config.storageUrl}/api/screener/consecutive-realtime?code=${code}`);
}

/**
 * 实时检测涨跌停
 * @returns 涨跌停股票列表
 */
export async function detectLimitStocks(): Promise<LimitItem[]> {
  return request.get(`${config.storageUrl}/api/screener/limit-detect`);
}
