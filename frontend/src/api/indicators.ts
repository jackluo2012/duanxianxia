// 技术指标 API 客户端
// 对应 query-service 的 Indicators API

import axios from 'axios';

const QUERY_API_BASE = 'http://127.0.0.1:8086';

// ============================================
// 类型定义
// ============================================

export interface IndicatorData {
  code: string;
  date: string;
  ma5: number | null;
  ma10: number | null;
  ma20: number | null;
  ma60: number | null;
  macd_dif: number | null;
  macd_dea: number | null;
  macd_bar: number | null;
  kdj_k: number | null;
  kdj_d: number | null;
  kdj_j: number | null;
  rsi6: number | null;
  rsi12: number | null;
  rsi24: number | null;
}

export interface MAData {
  date: string;
  ma5: number | null;
  ma10: number | null;
  ma20: number | null;
  ma60: number | null;
}

export interface MACDData {
  date: string;
  dif: number | null;
  dea: number | null;
  bar: number | null;
}

export interface KDJData {
  date: string;
  k: number | null;
  d: number | null;
  j: number | null;
}

export interface RSIData {
  date: string;
  rsi6: number | null;
  rsi12: number | null;
  rsi24: number | null;
}

// ============================================
// API 函数
// ============================================

/**
 * 获取股票的所有技术指标
 * @param code - 股票代码
 * @param limit - 返回数据条数（默认100）
 */
export async function fetchIndicators(
  code: string,
  limit: number = 100
): Promise<IndicatorData[]> {
  try {
    const response = await axios.get<IndicatorData[]>(
      `${QUERY_API_BASE}/api/indicators/${code}?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取技术指标失败:', error);
    throw error;
  }
}

/**
 * 获取移动平均线数据
 * @param code - 股票代码
 * @param limit - 返回数据条数（默认100）
 */
export async function fetchMA(
  code: string,
  limit: number = 100
): Promise<MAData[]> {
  try {
    const response = await axios.get<MAData[]>(
      `${QUERY_API_BASE}/api/indicators/${code}/ma?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取移动平均线失败:', error);
    throw error;
  }
}

/**
 * 获取 MACD 指标数据
 * @param code - 股票代码
 * @param limit - 返回数据条数（默认100）
 */
export async function fetchMACD(
  code: string,
  limit: number = 100
): Promise<MACDData[]> {
  try {
    const response = await axios.get<MACDData[]>(
      `${QUERY_API_BASE}/api/indicators/${code}/macd?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取 MACD 指标失败:', error);
    throw error;
  }
}

/**
 * 获取 KDJ 指标数据
 * @param code - 股票代码
 * @param limit - 返回数据条数（默认100）
 */
export async function fetchKDJ(
  code: string,
  limit: number = 100
): Promise<KDJData[]> {
  try {
    const response = await axios.get<KDJData[]>(
      `${QUERY_API_BASE}/api/indicators/${code}/kdj?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取 KDJ 指标失败:', error);
    throw error;
  }
}

/**
 * 获取 RSI 指标数据
 * @param code - 股票代码
 * @param limit - 返回数据条数（默认100）
 */
export async function fetchRSI(
  code: string,
  limit: number = 100
): Promise<RSIData[]> {
  try {
    const response = await axios.get<RSIData[]>(
      `${QUERY_API_BASE}/api/indicators/${code}/rsi?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取 RSI 指标失败:', error);
    throw error;
  }
}
