import { useQuery } from '@tanstack/react-query';
import { getLeaderBoard, getLeaderDetail } from '../api/leader';
import { useLeaderStore } from '../store/leaderStore';
import type { LeaderFilters } from '../types/leader';

// ·Ö’Lœpn
export const useLeaderBoard = (filters: LeaderFilters) => {
  return useQuery({
    queryKey: ['leaderBoard', filters],
    queryFn: () => getLeaderBoard(filters),
    staleTime: 30000,  // 30ÒX
    gcTime: 300000,  // 5ŸX
  });
};

// ·Ö¡hæÅ
export const useLeaderDetail = (code: string) => {
  return useQuery({
    queryKey: ['leaderDetail', code],
    queryFn: () => getLeaderDetail(code),
    enabled: !!code,  // ÅScodeX(öM÷B
    staleTime: 60000,  // 60ÒX
  });
};

// [	aö¡
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
