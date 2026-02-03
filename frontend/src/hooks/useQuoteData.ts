/**
 * 增强的行情数据 Hook
 * 支持历史数据和实时数据获取
 */

import { useCallback, useEffect, useState } from 'react';
import { useWebSocket } from './useWebSocket';
import { fetchQuoteHistory, fetchRealtimeQuote, HistoryPoint, StockQuote } from '../api/quotes';
import { config } from '../config';

interface UseQuoteDataOptions {
  enableRealtime?: boolean;
  autoRefresh?: boolean;
}

export function useQuoteData(
  initialCode: string = '000001',
  initialPeriod: string = '1m',
  options: UseQuoteDataOptions = {}
) {
  const { enableRealtime = true, autoRefresh = true } = options;

  const [selectedCode, setSelectedCode] = useState(initialCode);
  const [period, setPeriod] = useState(initialPeriod);
  const [klineData, setKlineData] = useState<HistoryPoint[]>([]);
  const [realtimeQuote, setRealtimeQuote] = useState<StockQuote | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // WebSocket连接
  const { status, subscribe, unsubscribe } = useWebSocket(
    `${config.realtimeUrl}/ws/realtime`,
    {
      onMessage: (message) => {
        if (message.type === 'quote_update') {
          const quote = message.data as StockQuote;
          if (quote.code === selectedCode) {
            setRealtimeQuote(quote);

            // 如果是1分钟周期，实时更新最后一个数据点
            if (period === '1m') {
              setKlineData((prev) => {
                const newData = [...prev];
                if (newData.length > 0) {
                  newData[newData.length - 1] = {
                    ...newData[newData.length - 1],
                    time: quote.datetime || newData[newData.length - 1].time,
                    price: quote.price,
                    close: quote.price,
                    vol: quote.vol,
                    high: Math.max(newData[newData.length - 1].high || quote.price, quote.price),
                    low: Math.min(newData[newData.length - 1].low || quote.price, quote.price),
                  };
                }
                return newData;
              });
            }
          }
        }
      },
      onError: (error) => {
        console.error('[useQuoteData] WebSocket错误:', error);
        setError('实时数据连接异常');
      },
    }
  );

  /**
   * 加载历史K线数据
   */
  const fetchHistory = useCallback(async (code: string, newPeriod: string) => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchQuoteHistory(code, newPeriod);
      setKlineData(response.data);

      // 从最后一个K线点构造实时行情
      if (response.data.length > 0) {
        const lastPoint = response.data[response.data.length - 1];
        setRealtimeQuote({
          code: response.code,
          name: response.name,
          price: lastPoint.close || lastPoint.price || 0,
          preclose: lastPoint.preclose || 0,
          open: lastPoint.open || 0,
          high: lastPoint.high || 0,
          low: lastPoint.low || 0,
          vol: lastPoint.vol,
          amount: lastPoint.amount || 0,
          change_percent: lastPoint.change_percent || 0,
          datetime: lastPoint.time,
        });
      }
    } catch (err) {
      console.error('[useQuoteData] 加载历史数据失败:', err);
      setError('加载历史数据失败，请稍后重试');
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 加载实时行情
   */
  const loadRealtimeQuote = useCallback(async (code: string) => {
    try {
      const quote = await fetchRealtimeQuote(code);
      setRealtimeQuote(quote);
      return quote;
    } catch (err) {
      console.error('[useQuoteData] 加载实时行情失败:', err);
      return null;
    }
  }, []);

  /**
   * 选择股票
   */
  const selectStock = useCallback(
    (code: string) => {
      setSelectedCode(code);

      // 取消旧订阅
      if (enableRealtime && period === '1m') {
        unsubscribe([selectedCode]);
      }

      // 加载新数据
      fetchHistory(code, period);

      // 订阅新股票
      if (enableRealtime && period === '1m') {
        subscribe([code]);
      }
    },
    [fetchHistory, period, selectedCode, enableRealtime, subscribe, unsubscribe]
  );

  /**
   * 选择周期
   */
  const selectPeriod = useCallback(
    (newPeriod: string) => {
      setPeriod(newPeriod);

      // 取消订阅
      if (enableRealtime && period === '1m') {
        unsubscribe([selectedCode]);
      }

      // 加载新周期数据
      fetchHistory(selectedCode, newPeriod);

      // 如果切换到1分钟，重新订阅
      if (enableRealtime && newPeriod === '1m') {
        subscribe([selectedCode]);
      }
    },
    [fetchHistory, selectedCode, period, enableRealtime, subscribe, unsubscribe]
  );

  /**
   * 刷新数据
   */
  const refresh = useCallback(() => {
    fetchHistory(selectedCode, period);
    if (enableRealtime) {
      loadRealtimeQuote(selectedCode);
    }
  }, [fetchHistory, loadRealtimeQuote, selectedCode, period, enableRealtime]);

  // 初始化加载
  useEffect(() => {
    fetchHistory(selectedCode, period);

    // 加载实时行情
    if (enableRealtime) {
      loadRealtimeQuote(selectedCode);
    }
  }, []); // 只在组件挂载时执行

  // WebSocket连接后订阅
  useEffect(() => {
    if (status === 'connected' && enableRealtime && period === '1m') {
      subscribe([selectedCode]);
    }

    return () => {
      if (enableRealtime) {
        unsubscribe([selectedCode]);
      }
    };
  }, [status, selectedCode, period, enableRealtime, subscribe, unsubscribe]);

  // 自动刷新（仅非1分钟周期）
  useEffect(() => {
    if (!autoRefresh || period === '1m') {
      return;
    }

    const interval = setInterval(() => {
      refresh();
    }, 5000); // 每5秒刷新一次

    return () => clearInterval(interval);
  }, [period, refresh, autoRefresh]);

  return {
    selectedCode,
    period,
    klineData,
    realtimeQuote,
    loading,
    error,
    wsStatus: status,
    selectStock,
    selectPeriod,
    refresh,
  };
}
