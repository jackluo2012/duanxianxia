import axios from 'axios';

const API_BASE_URL = 'http://localhost:8084';

export interface WatchlistItem {
  code: string;
  name: string;
  added_at: string;
}

// 获取自选股列表
export async function getWatchlist(userId: string = 'default'): Promise<WatchlistItem[]> {
  const response = await axios.get(`${API_BASE_URL}/api/auction/watchlist`, {
    params: { user_id: userId },
  });
  return response.data.items;
}

// 添加股票到自选股
export async function addToWatchlist(
  code: string,
  name: string,
  userId: string = 'default'
): Promise<{ message: string; code: string; name: string }> {
  const response = await axios.post(`${API_BASE_URL}/api/auction/watchlist`, {
    code,
    name,
    user_id: userId,
  });
  return response.data;
}

// 从自选股中移除股票
export async function removeFromWatchlist(
  code: string,
  userId: string = 'default'
): Promise<{ message: string; code: string }> {
  const response = await axios.delete(`${API_BASE_URL}/api/auction/watchlist/${code}`, {
    params: { user_id: userId },
  });
  return response.data;
}

// 检查股票是否在自选股中
export async function isWatched(code: string, userId: string = 'default'): Promise<boolean> {
  const response = await axios.get(`${API_BASE_URL}/api/auction/watchlist/${code}/check`, {
    params: { user_id: userId },
  });
  return response.data.watched;
}
