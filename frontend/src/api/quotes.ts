import request from './request';
import { config } from '../config';

export interface HistoryPoint {
  time: string;
  price?: number;
  open?: number;
  high?: number;
  low?: number;
  close?: number;
  vol: number;
  amount?: number;
}

export interface HistoryResponse {
  code: string;
  name: string;
  period: string;
  data: HistoryPoint[];
}

export interface StockQuote {
  code: string;
  name: string;
  price: number;
  preclose: number;
  open: number;
  high: number;
  low: number;
  vol: number;
  amount: number;
  change_percent: number;
  datetime?: string;
}

/**
 * 获取股票历史K线数据
 * @param code 股票代码
 * @param period 周期 (1m/5m/15m/30m/60m/1d)
 * @returns 历史K线数据
 */
export async function fetchQuoteHistory(
  code: string,
  period: string = '1m'
): Promise<HistoryResponse> {
  return request.get(
    `${config.storageUrl}/api/quotes/${code}/history?period=${period}`
  );
}

/**
 * 获取实时行情
 * @param codes 股票代码数组
 * @returns 实时行情数据
 */
export async function fetchRealtimeQuotes(codes: string[]): Promise<StockQuote[]> {
  return request.post(`${config.apiBaseUrl}/api/quotes/batch`, { codes });
}

/**
 * 获取单只股票实时行情
 * @param code 股票代码
 * @returns 实时行情数据
 */
export async function fetchRealtimeQuote(code: string): Promise<StockQuote> {
  return request.get(`${config.apiBaseUrl}/api/quotes/${code}`);
}
