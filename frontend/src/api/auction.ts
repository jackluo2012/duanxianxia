/**
 * 竞价分析相关 API
 */

import request from './request';
import { config } from '../config';

export interface AuctionStock {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  sealed_amount_buy: number;
  sealed_amount_sell: number;
  intensity_score?: number;
  open_price?: number;
  preclose_price?: number;
  volume?: number;
  amount?: number;
  updateTime?: string;
}

export interface AuctionRankingResponse {
  data: AuctionStock[];
  total: number;
  updateTime: string;
}

export interface AuctionDetailResponse extends AuctionStock {
  // 详细字段
  buy_orders: OrderBook[];
  sell_orders: OrderBook[];
  history: AuctionHistoryPoint[];
}

export interface OrderBook {
  price: number;
  volume: number;
  amount: number;
}

export interface AuctionHistoryPoint {
  time: string;
  price: number;
  volume: number;
  buy_sealed: number;
  sell_sealed: number;
}

/**
 * 获取竞价排行榜
 * @param rankingType 排行类型 (buy_sealed/intensity/change/anomaly)
 * @param limit 返回数量限制
 * @returns 排行榜数据
 */
export async function fetchAuctionRankings(
  rankingType: string = 'buy_sealed',
  limit: number = 50
): Promise<AuctionRankingResponse> {
  return request.get(
    `${config.storageUrl}/api/auction/rankings?type=${rankingType}&limit=${limit}`
  );
}

/**
 * 获取竞价详情
 * @param code 股票代码
 * @returns 竞价详情数据
 */
export async function fetchAuctionDetail(code: string): Promise<AuctionDetailResponse> {
  return request.get(`${config.storageUrl}/api/auction/detail/${code}`);
}

/**
 * 获取竞价历史数据
 * @param code 股票代码
 * @param limit 数据点数量
 * @returns 历史数据
 */
export async function fetchAuctionHistory(
  code: string,
  limit: number = 100
): Promise<{ data: AuctionHistoryPoint[] }> {
  return request.get(
    `${config.storageUrl}/api/auction/history/${code}?limit=${limit}`
  );
}
