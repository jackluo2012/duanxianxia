/**
 * 个股挖掘数据 Hook
 * 管理龙头高度、连板统计、涨跌停等数据
 */

import { useState, useCallback, useEffect } from 'react';
import { message } from 'antd';
import {
  fetchLeaders,
  fetchConsecutiveBoards,
  fetchLimitUp,
  fetchLimitDown,
  LeaderItem,
  ConsecutiveBoardItem,
  LimitItem,
} from '../api/screener';

interface UseScreenerDataOptions {
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function useScreenerData(options: UseScreenerDataOptions = {}) {
  const {
    autoRefresh = false,
    refreshInterval = 30000,
  } = options;

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  // 龙头高度数据
  const [leaders, setLeaders] = useState<LeaderItem[]>([]);
  const [leaderSector, setLeaderSector] = useState<string>('');

  // 连板统计数据
  const [consecutiveData, setConsecutiveData] = useState<ConsecutiveBoardItem[]>([]);
  const [minDays, setMinDays] = useState<number>(2);
  const [boardType, setBoardType] = useState<string>('连涨');

  // 涨跌停数据
  const [limitUpData, setLimitUpData] = useState<LimitItem[]>([]);
  const [limitDownData, setLimitDownData] = useState<LimitItem[]>([]);

  /**
   * 加载龙头高度数据
   */
  const loadLeaders = useCallback(async (sector?: string) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchLeaders(sector || undefined);
      setLeaders(data);
      setLastUpdate(new Date());
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取龙头高度数据失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 加载连板统计数据
   */
  const loadConsecutive = useCallback(async (
    days: number = minDays,
    type: string = boardType
  ) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchConsecutiveBoards(days, type);
      setConsecutiveData(data);
      setLastUpdate(new Date());
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取连板统计失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [minDays, boardType]);

  /**
   * 加载涨跌停数据
   */
  const loadLimits = useCallback(async (date: string = 'today') => {
    setLoading(true);
    setError(null);
    try {
      const [upData, downData] = await Promise.all([
        fetchLimitUp(date),
        fetchLimitDown(date),
      ]);
      setLimitUpData(upData);
      setLimitDownData(downData);
      setLastUpdate(new Date());
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取涨跌停数据失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 刷新所有数据
   */
  const refresh = useCallback(() => {
    loadLeaders(leaderSector || undefined);
    loadConsecutive();
    loadLimits();
  }, [loadLeaders, loadConsecutive, loadLimits, leaderSector]);

  // 初始加载
  useEffect(() => {
    loadLeaders();
    loadConsecutive();
    loadLimits();
  }, [loadLeaders, loadConsecutive, loadLimits]);

  // 自动刷新
  useEffect(() => {
    if (!autoRefresh) return;

    const timer = setInterval(() => {
      refresh();
    }, refreshInterval);

    return () => clearInterval(timer);
  }, [autoRefresh, refreshInterval, refresh]);

  return {
    // 龙头高度
    leaders,
    leaderSector,
    setLeaderSector,

    // 连板统计
    consecutiveData,
    minDays,
    setMinDays,
    boardType,
    setBoardType,

    // 涨跌停
    limitUpData,
    limitDownData,

    // 状态
    loading,
    error,
    lastUpdate,

    // 方法
    loadLeaders,
    loadConsecutive,
    loadLimits,
    refresh,
  };
}
