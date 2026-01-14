import { useQuery } from '@tanstack/react-query';
import { getLeaderBoard, getLeaderDetail } from '../api/leader';
import { useLeaderStore } from '../store/leaderStore';
import type { LeaderFilters } from '../types/leader';

// 获取排行榜数据
export const useLeaderBoard = (filters: LeaderFilters) => {
  return useQuery({
    queryKey: ['leaderBoard', filters],
    queryFn: () => getLeaderBoard(filters),
    staleTime: 30000,  // 30秒
    gcTime: 300000,  // 5分钟
    select: (data) => ({
      ...data,
      items: data.items.map(item => ({
        ...item,
        id: `${item.code}-${item.consecutive_limit_up}`,  // 添加唯一ID
      })),
    }),
  });
};

// 获取股票详情
export const useLeaderDetail = (code: string) => {
  return useQuery({
    queryKey: ['leaderDetail', code],
    queryFn: () => getLeaderDetail(code),
    enabled: !!code,  // 只有code存在时才启用
    staleTime: 60000,  // 60秒
  });
};

// 筛选条件管理
export const useLeaderFilters = () => {
  const { filters, updateFilters } = useLeaderStore();

  const handleMarketChange = (market: number | undefined) => {
    updateFilters({ market });
  };

  const handleDateRangeChange = (dates: [string, string]) => {
    updateFilters({ date_range: dates });
  };

  const handleMinConsecutiveChange = (min: number) => {
    updateFilters({ min_consecutive: min });
  };

  return {
    filters,
    handleMarketChange,
    handleDateRangeChange,
    handleMinConsecutiveChange,
  };
};
