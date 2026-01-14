import { create } from 'zustand';
import type { LeaderStock, LeaderFilters } from '../types/leader';

interface LeaderStore {
  // State
  selectedStock: LeaderStock | null;
  comparedStocks: LeaderStock[];
  filters: LeaderFilters;

  // Actions
  setSelectedStock: (stock: LeaderStock | null) => void;
  addComparedStock: (stock: LeaderStock) => void;
  removeComparedStock: (code: string) => void;
  updateFilters: (filters: Partial<LeaderFilters>) => void;
  clearComparedStocks: () => void;
}

const getStartDate = (): string => {
  const date = new Date();
  date.setDate(date.getDate() - 30);
  return date.toISOString().split('T')[0];
};

const today = (): string => {
  return new Date().toISOString().split('T')[0];
};

export const useLeaderStore = create<LeaderStore>((set) => ({
  selectedStock: null,
  comparedStocks: [],
  filters: {
    market: undefined,
    min_consecutive: 3,
    date_range: [getStartDate(), today()],
  },

  setSelectedStock: (stock) => set({ selectedStock: stock }),

  addComparedStock: (stock) => set((state) => {
    // Avoid duplicates
    if (state.comparedStocks.some(s => s.code === stock.code)) {
      return state;
    }
    return { comparedStocks: [...state.comparedStocks, stock] };
  }),

  removeComparedStock: (code) => set((state) => ({
    comparedStocks: state.comparedStocks.filter(s => s.code !== code)
  })),

  updateFilters: (filters) => set((state) => ({
    filters: { ...state.filters, ...filters }
  })),

  clearComparedStocks: () => set({ comparedStocks: [] }),
}));
