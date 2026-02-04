/**
 * 技术指标 API 接口
 * 提供MA、MACD、KDJ、RSI等技术指标数据
 */

import request from './request';
import { config } from '../config';

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
 * @param code 股票代码
 * @param limit 返回数据条数
 * @returns 技术指标数据
 */
export async function fetchIndicators(
  code: string,
  limit: number = 100
): Promise<IndicatorData[]> {
  return request.get(`${config.storageUrl}/api/indicators/${code}?limit=${limit}`);
}

/**
 * 获取移动平均线数据
 * @param code 股票代码
 * @param limit 返回数据条数
 * @returns MA数据
 */
export async function fetchMA(
  code: string,
  limit: number = 100
): Promise<MAData[]> {
  return request.get(`${config.storageUrl}/api/indicators/${code}/ma?limit=${limit}`);
}

/**
 * 获取 MACD 指标数据
 * @param code 股票代码
 * @param limit 返回数据条数
 * @returns MACD数据
 */
export async function fetchMACD(
  code: string,
  limit: number = 100
): Promise<MACDData[]> {
  return request.get(`${config.storageUrl}/api/indicators/${code}/macd?limit=${limit}`);
}

/**
 * 获取 KDJ 指标数据
 * @param code 股票代码
 * @param limit 返回数据条数
 * @returns KDJ数据
 */
export async function fetchKDJ(
  code: string,
  limit: number = 100
): Promise<KDJData[]> {
  return request.get(`${config.storageUrl}/api/indicators/${code}/kdj?limit=${limit}`);
}

/**
 * 获取 RSI 指标数据
 * @param code 股票代码
 * @param limit 返回数据条数
 * @returns RSI数据
 */
export async function fetchRSI(
  code: string,
  limit: number = 100
): Promise<RSIData[]> {
  return request.get(`${config.storageUrl}/api/indicators/${code}/rsi?limit=${limit}`);
}
