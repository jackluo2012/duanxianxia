import request from './request';

export interface WatchlistItem {
  code: string;
  name: string;
  added_at: string;
}

// 获取自选股列表
export async function getWatchlist(userId: string = 'default'): Promise<WatchlistItem[]> {
  const response = await request.get<{items: WatchlistItem[]}>(
    `/auction/watchlist?user_id=${userId}`
  );
  return response.items || [];
}

// 添加股票到自选股
export async function addToWatchlist(
  code: string,
  name: string,
  userId: string = 'default'
): Promise<{ message: string; code: string; name: string }> {
  return request.post('/auction/watchlist', {
    code,
    name,
    user_id: userId,
  });
}

// 从自选股中移除股票
export async function removeFromWatchlist(
  code: string,
  userId: string = 'default'
): Promise<{ message: string; code: string }> {
  return request.delete(`/auction/watchlist/${code}?user_id=${userId}`);
}

// 检查股票是否在自选股中
export async function isWatched(code: string, userId: string = 'default'): Promise<boolean> {
  const response = await request.get<{watched: boolean}>(
    `/auction/watchlist/${code}/check?user_id=${userId}`
  );
  return response.watched || false;
}
