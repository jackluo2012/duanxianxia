import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import LeaderItem from '../LeaderItem';
import type { LeaderBoardItem } from '../../../types/leader';

describe('LeaderItem', () => {
  const mockItem: LeaderBoardItem = {
    code: '000001',
    name: '平安银行',
    price: 13.75,
    change_percent: 10.0,
    market_cap: 250.5,
    sector: '银行',
    consecutive_limit_up: 5,
    history_max: 8,
    recent_limit_ups: [],
    sealed_amount: 500000000,
  };

  it('应该正确显示股票信息', () => {
    const onSelect = vi.fn();
    const onAddCompare = vi.fn();

    render(
      <LeaderItem
        item={mockItem}
        isSelected={false}
        onSelect={onSelect}
        onAddCompare={onAddCompare}
      />
    );

    expect(screen.getByText('平安银行')).toBeInTheDocument();
    expect(screen.getByText('000001')).toBeInTheDocument();
    expect(screen.getByText('5连板')).toBeInTheDocument();
  });

  it('点击时应该调用onSelect回调', () => {
    const onSelect = vi.fn();
    const onAddCompare = vi.fn();

    render(
      <LeaderItem
        item={mockItem}
        isSelected={false}
        onSelect={onSelect}
        onAddCompare={onAddCompare}
      />
    );

    const card = screen.getByText('平安银行').closest('.ant-card') as HTMLElement;
    card?.click();

    expect(onSelect).toHaveBeenCalledWith(mockItem);
  });
});
