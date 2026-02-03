/**
 * 增强的竞价排行榜 Hook
 * 支持自动刷新、WebSocket实时更新
 */

import { useState, useEffect, useCallback } from 'react';
import { fetchAuctionRankings, AuctionStock } from '../api/auction';

export interface UseAuctionRankingOptions {
  rankingType?: string;
  limit?: number;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function useAuctionRanking(options: UseAuctionRankingOptions = {}) {
  const {
    rankingType = 'buy_sealed',
    limit = 50,
    autoRefresh = true,
    refreshInterval = 5000,
  } = options;

  const [data, setData] = useState<AuctionStock[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  /**
   * 加载排行榜数据
   */
  const fetchRankings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchAuctionRankings(rankingType, limit);
      setData(response.data);
      setLastUpdate(new Date(response.updateTime));
    } catch (err: any) {
      console.error('[useAuctionRanking] 加载失败:', err);
      setError(err.message || '加载竞价排行榜失败');
    } finally {
      setLoading(false);
    }
  }, [rankingType, limit]);

  /**
   * 手动刷新
   */
  const refresh = useCallback(() => {
    fetchRankings();
  }, [fetchRankings]);

  // 初始加载
  useEffect(() => {
    fetchRankings();
  }, [fetchRankings]);

  // 自动刷新
  useEffect(() => {
    if (!autoRefresh) {
      return;
    }

    const interval = setInterval(() => {
      fetchRankings();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, fetchRankings]);

  return {
    data,
    loading,
    error,
    lastUpdate,
    refresh,
  };
}
