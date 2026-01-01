import axios from 'axios';

export interface HistoryPoint {
  time: string;
  price?: number;
  open?: number;
  high?: number;
  low?: number;
  close?: number;
  vol: number;
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

const API_BASE_URL = 'http://localhost:8083';

export async function fetchQuoteHistory(code: string, period: string = '1m'): Promise<HistoryResponse> {
  const response = await axios.get<HistoryResponse>(
    `${API_BASE_URL}/api/quotes/${code}/history?period=${period}`
  );
  return response.data;
}
