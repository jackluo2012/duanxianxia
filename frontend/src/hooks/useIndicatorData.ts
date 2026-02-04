/**
 * 技术指标数据 Hook
 * 管理MA、MACD、KDJ、RSI等技术指标数据
 */

import { useState, useCallback } from 'react';
import { message } from 'antd';
import {
  fetchMA,
  fetchMACD,
  fetchKDJ,
  fetchRSI,
  MAData,
  MACDData,
  KDJData,
  RSIData,
} from '../api/indicators';

export function useIndicatorData() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // MA数据
  const [maData, setMAData] = useState<MAData[]>([]);

  // MACD数据
  const [macdData, setMACDData] = useState<MACDData[]>([]);

  // KDJ数据
  const [kdjData, setKDJData] = useState<KDJData[]>([]);

  // RSI数据
  const [rsiData, setRSIData] = useState<RSIData[]>([]);

  /**
   * 加载单个指标数据
   */
  const loadIndicator = useCallback(
    async (code: string, indicator: 'ma' | 'macd' | 'kdj' | 'rsi', limit: number = 100) => {
      if (!code || code.trim() === '') {
        message.warning('请输入股票代码');
        return;
      }

      setLoading(true);
      setError(null);

      try {
        let data;
        switch (indicator) {
          case 'ma':
            data = await fetchMA(code, limit);
            setMAData(data);
            break;
          case 'macd':
            data = await fetchMACD(code, limit);
            setMACDData(data);
            break;
          case 'kdj':
            data = await fetchKDJ(code, limit);
            setKDJData(data);
            break;
          case 'rsi':
            data = await fetchRSI(code, limit);
            setRSIData(data);
            break;
        }
      } catch (err: any) {
        const errorMsg = err?.response?.data?.message || err?.message || '获取指标数据失败';
        setError(errorMsg);
        message.error(errorMsg);
      } finally {
        setLoading(false);
      }
    },
    []
  );

  /**
   * 加载所有指标数据
   */
  const loadAll = useCallback(async (code: string, limit: number = 100) => {
    if (!code || code.trim() === '') {
      message.warning('请输入股票代码');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [ma, macd, kdj, rsi] = await Promise.all([
        fetchMA(code, limit),
        fetchMACD(code, limit),
        fetchKDJ(code, limit),
        fetchRSI(code, limit),
      ]);

      setMAData(ma);
      setMACDData(macd);
      setKDJData(kdj);
      setRSIData(rsi);
    } catch (err: any) {
      const errorMsg = err?.response?.data?.message || err?.message || '获取指标数据失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 清空所有数据
   */
  const clear = useCallback(() => {
    setMAData([]);
    setMACDData([]);
    setKDJData([]);
    setRSIData([]);
    setError(null);
  }, []);

  return {
    // 数据
    maData,
    macdData,
    kdjData,
    rsiData,

    // 状态
    loading,
    error,

    // 方法
    loadIndicator,
    loadAll,
    clear,
  };
}
