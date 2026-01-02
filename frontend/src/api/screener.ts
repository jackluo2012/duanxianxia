// 个股挖掘 API 客户端
// 对应 query-service 的 Screener API

import axios from 'axios';

const QUERY_API_BASE = 'http://127.0.0.1:8086';

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
 * @param sector - 板块代码（可选）
 * @param limit - 返回数量限制（默认50）
 */
export async function fetchLeaders(
  sector?: string,
  limit: number = 50
): Promise<LeaderItem[]> {
  try {
    const url = sector
      ? `${QUERY_API_BASE}/api/screener/leaders?sector=${sector}&limit=${limit}`
      : `${QUERY_API_BASE}/api/screener/leaders?limit=${limit}`;

    const response = await axios.get<LeaderItem[]>(url);
    return response.data;
  } catch (error) {
    console.error('获取龙头高度排行榜失败:', error);
    throw error;
  }
}

/**
 * 获取连板统计数据
 * @param minDays - 最小连板天数（默认2）
 * @param boardType - 连板类型："连涨" 或 "连跌"（默认"连涨"）
 * @param limit - 返回数量限制（默认50）
 */
export async function fetchConsecutiveBoards(
  minDays: number = 2,
  boardType: string = '连涨',
  limit: number = 50
): Promise<ConsecutiveBoardItem[]> {
  try {
    const response = await axios.get<ConsecutiveBoardItem[]>(
      `${QUERY_API_BASE}/api/screener/consecutive?min_days=${minDays}&board_type=${boardType}&limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取连板统计失败:', error);
    throw error;
  }
}

/**
 * 获取涨停股票列表
 * @param date - 日期（"today" 或具体日期如"2024-01-01"，默认"today"）
 * @param limit - 返回数量限制（默认50）
 */
export async function fetchLimitUp(
  date: string = 'today',
  limit: number = 50
): Promise<LimitItem[]> {
  try {
    const response = await axios.get<LimitItem[]>(
      `${QUERY_API_BASE}/api/screener/limit-up?date=${date}&limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取涨停股票失败:', error);
    throw error;
  }
}

/**
 * 获取跌停股票列表
 * @param date - 日期（"today" 或具体日期如"2024-01-01"，默认"today"）
 * @param limit - 返回数量限制（默认50）
 */
export async function fetchLimitDown(
  date: string = 'today',
  limit: number = 50
): Promise<LimitItem[]> {
  try {
    const response = await axios.get<LimitItem[]>(
      `${QUERY_API_BASE}/api/screener/limit-down?date=${date}&limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取跌停股票失败:', error);
    throw error;
  }
}

/**
 * 实时计算龙头高度（基于当前行情数据）
 * @param sectorCode - 板块代码
 */
export async function fetchLeadersRealtime(
  sectorCode: string
): Promise<LeaderItem[]> {
  try {
    const response = await axios.get<LeaderItem[]>(
      `${QUERY_API_BASE}/api/screener/leaders-realtime?sector_code=${sectorCode}`
    );
    return response.data;
  } catch (error) {
    console.error('获取实时龙头高度失败:', error);
    throw error;
  }
}

/**
 * 实时计算连板天数（基于历史数据）
 * @param code - 股票代码
 */
export async function fetchConsecutiveRealtime(
  code: string
): Promise<number> {
  try {
    const response = await axios.get<number>(
      `${QUERY_API_BASE}/api/screener/consecutive-realtime?code=${code}`
    );
    return response.data;
  } catch (error) {
    console.error('获取实时连板天数失败:', error);
    throw error;
  }
}

/**
 * 实时检测涨跌停（基于当前行情）
 */
export async function detectLimitStocks(): Promise<LimitItem[]> {
  try {
    const response = await axios.get<LimitItem[]>(
      `${QUERY_API_BASE}/api/screener/limit-detect`
    );
    return response.data;
  } catch (error) {
    console.error('实时检测涨跌停失败:', error);
    throw error;
  }
}
