import { useCallback, useEffect, useState } from 'react';
import { useWebSocket } from './useWebSocket';
import { fetchQuoteHistory, HistoryPoint, StockQuote } from '../api/quotes';

export function useQuoteData(initialCode: string = '000001', initialPeriod: string = '1m') {
  const [selectedCode, setSelectedCode] = useState(initialCode);
  const [period, setPeriod] = useState(initialPeriod);
  const [klineData, setKlineData] = useState<HistoryPoint[]>([]);
  const [realtimeQuote, setRealtimeQuote] = useState<StockQuote | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { status, subscribe } = useWebSocket('ws://localhost:8080/ws/realtime', {
    onMessage: (message) => {
      if (message.type === 'quote_update' && period === '1m') {
        const quote = message.data as StockQuote;
        if (quote.code === selectedCode) {
          setRealtimeQuote(quote);
          setKlineData((prev) => {
            const newData = [...prev];
            if (newData.length > 0) {
              newData[newData.length - 1] = {
                time: quote.datetime || newData[newData.length - 1].time,
                price: quote.price,
                vol: quote.vol,
              };
            }
            return newData;
          });
        }
      }
    },
  });

  const fetchHistory = useCallback(async (code: string, newPeriod: string) => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchQuoteHistory(code, newPeriod);
      setKlineData(response.data);
      if (response.data.length > 0) {
        const lastPoint = response.data[response.data.length - 1];
        setRealtimeQuote({
          code: response.code,
          name: response.name,
          price: lastPoint.close || lastPoint.price || 0,
          vol: lastPoint.vol,
          preclose: 0,
          open: lastPoint.open || 0,
          high: lastPoint.high || 0,
          low: lastPoint.low || 0,
          close: lastPoint.close || 0,
          amount: 0,
          change_percent: 0,
          datetime: lastPoint.time,
        });
      }
    } catch (err) {
      console.error('加载历史数据失败:', err);
      setError('加载历史数据失败');
    } finally {
      setLoading(false);
    }
  }, []);

  const selectStock = useCallback(
    (code: string) => {
      setSelectedCode(code);
      fetchHistory(code, period);
      if (period === '1m') {
        subscribe([code]);
      }
    },
    [fetchHistory, period, subscribe]
  );

  const selectPeriod = useCallback(
    (newPeriod: string) => {
      setPeriod(newPeriod);
      fetchHistory(selectedCode, newPeriod);
      if (newPeriod === '1m') {
        subscribe([selectedCode]);
      }
    },
    [fetchHistory, selectedCode, subscribe]
  );

  useEffect(() => {
    fetchHistory(selectedCode, period);
  }, []);

  useEffect(() => {
    if (status === 'connected' && period === '1m') {
      subscribe([selectedCode]);
    }
  }, [status, selectedCode, period, subscribe]);

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
  };
}
