// 概念板块 API 客户端
// 对应 query-service 的 Sectors API

import axios from 'axios';

const QUERY_API_BASE = 'http://127.0.0.1:8086';

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
}

// ============================================
// API 函数
// ============================================

/**
 * 获取板块列表
 * @param limit - 返回数量限制（默认100）
 */
export async function fetchSectors(
  limit: number = 100
): Promise<SectorItem[]> {
  try {
    const response = await axios.get<SectorItem[]>(
      `${QUERY_API_BASE}/api/sectors/list?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取板块列表失败:', error);
    throw error;
  }
}

/**
 * 获取板块表现排行
 * @param limit - 返回数量限制（默认50）
 */
export async function fetchSectorPerformance(
  limit: number = 50
): Promise<SectorPerformanceItem[]> {
  try {
    const response = await axios.get<SectorPerformanceItem[]>(
      `${QUERY_API_BASE}/api/sectors/performance?limit=${limit}`
    );
    return response.data;
  } catch (error) {
    console.error('获取板块表现失败:', error);
    throw error;
  }
}

/**
 * 获取板块成分股
 * @param code - 板块代码
 */
export async function fetchSectorStocks(
  code: string
): Promise<SectorStockItem[]> {
  try {
    const response = await axios.get<SectorStockItem[]>(
      `${QUERY_API_BASE}/api/sectors/stocks/${code}`
    );
    return response.data;
  } catch (error) {
    console.error('获取板块成分股失败:', error);
    throw error;
  }
}

/**
 * 搜索板块
 * @param keyword - 搜索关键词
 */
export async function searchSectors(
  keyword: string
): Promise<SectorItem[]> {
  try {
    const response = await axios.get<SectorItem[]>(
      `${QUERY_API_BASE}/api/sectors/search/${encodeURIComponent(keyword)}`
    );
    return response.data;
  } catch (error) {
    console.error('搜索板块失败:', error);
    throw error;
  }
}
