/**
 * 概念板块 API 接口
 * 提供板块列表、板块表现、板块成分股等数据
 */

import request from './request';
import { config } from '../config';

// ============================================
// 类型定义
// ============================================

export interface SectorItem {
  code: string;
  name: string;
  stock_count: number;
  avg_change_percent: number;
  total_amount: number;
  limit_up_count: number;
  limit_down_count: number;
  leader_code?: string;
  leader_name?: string;
}

export interface SectorPerformanceItem {
  sector_code: string;
  sector_name: string;
  avg_change_percent: number;
  median_change_percent: number;
  total_volume: number;
  total_amount: number;
  stock_count: number;
  limit_up_count: number;
  limit_down_count: number;
  rise_count: number;
  fall_count: number;
  flat_count: number;
}

export interface SectorStockItem {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  volume: number;
  amount: number;
  is_leader?: boolean;
  limit_up?: boolean;
  limit_down?: boolean;
}

// ============================================
// API 函数
// ============================================

/**
 * 获取板块列表
 * @param limit 返回数量限制
 * @returns 板块列表
 */
export async function fetchSectors(limit: number = 100): Promise<SectorItem[]> {
  return request.get(`${config.storageUrl}/api/sectors/list?limit=${limit}`);
}

/**
 * 获取板块表现排行
 * @param limit 返回数量限制
 * @returns 板块表现列表
 */
export async function fetchSectorPerformance(
  limit: number = 50
): Promise<SectorPerformanceItem[]> {
  return request.get(`${config.storageUrl}/api/sectors/performance?limit=${limit}`);
}

/**
 * 获取板块成分股
 * @param code 板块代码
 * @returns 成分股列表
 */
export async function fetchSectorStocks(code: string): Promise<SectorStockItem[]> {
  return request.get(`${config.storageUrl}/api/sectors/stocks/${code}`);
}

/**
 * 搜索板块
 * @param keyword 搜索关键词
 * @returns 匹配的板块列表
 */
export async function searchSectors(keyword: string): Promise<SectorItem[]> {
  return request.get(`${config.storageUrl}/api/sectors/search/${encodeURIComponent(keyword)}`);
}
