/**
 * 板块数据 Hook
 * 管理板块列表、表现排行、成分股等数据
 */

import { useState, useCallback, useEffect } from 'react';
import { message } from 'antd';
import {
  fetchSectors,
  fetchSectorPerformance,
  fetchSectorStocks,
  searchSectors,
  SectorItem,
  SectorPerformanceItem,
  SectorStockItem,
} from '../api/sectors';

interface UseSectorDataOptions {
  autoRefresh?: boolean;
  refreshInterval?: number;
  initialLimit?: number;
}

export function useSectorData(options: UseSectorDataOptions = {}) {
  const {
    autoRefresh = false,
    refreshInterval = 30000,
    initialLimit = 100,
  } = options;

  const [sectors, setSectors] = useState<SectorItem[]>([]);
  const [performance, setPerformance] = useState<SectorPerformanceItem[]>([]);
  const [selectedSectorStocks, setSelectedSectorStocks] = useState<SectorStockItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  /**
   * 加载板块列表
   */
  const loadSectors = useCallback(async (limit: number = initialLimit) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchSectors(limit);
      setSectors(data);
      setLastUpdate(new Date());
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取板块列表失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [initialLimit]);

  /**
   * 加载板块表现排行
   */
  const loadPerformance = useCallback(async (limit: number = 50) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchSectorPerformance(limit);
      setPerformance(data);
      setLastUpdate(new Date());
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取板块表现失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 加载板块成分股
   */
  const loadSectorStocks = useCallback(async (sectorCode: string) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchSectorStocks(sectorCode);
      setSelectedSectorStocks(data);
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取成分股失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 搜索板块
   */
  const search = useCallback(async (keyword: string) => {
    if (!keyword.trim()) {
      return loadSectors();
    }
    setLoading(true);
    setError(null);
    try {
      const data = await searchSectors(keyword);
      setSectors(data);
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '搜索板块失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [loadSectors]);

  /**
   * 刷新所有数据
   */
  const refresh = useCallback(() => {
    loadSectors();
    loadPerformance();
  }, [loadSectors, loadPerformance]);

  // 初始加载
  useEffect(() => {
    loadSectors();
    loadPerformance();
  }, [loadSectors, loadPerformance]);

  // 自动刷新
  useEffect(() => {
    if (!autoRefresh) return;

    const timer = setInterval(() => {
      refresh();
    }, refreshInterval);

    return () => clearInterval(timer);
  }, [autoRefresh, refreshInterval, refresh]);

  return {
    // 数据
    sectors,
    performance,
    selectedSectorStocks,
    lastUpdate,

    // 状态
    loading,
    error,

    // 方法
    loadSectors,
    loadPerformance,
    loadSectorStocks,
    search,
    refresh,
  };
}
