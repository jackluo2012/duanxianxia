import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import FilterBar from '../FilterBar';

describe('FilterBar', () => {
  it('应该正确渲染筛选栏', () => {
    render(<FilterBar />);

    expect(screen.getByText('市场')).toBeInTheDocument();
    expect(screen.getByText('连板天数')).toBeInTheDocument();
    expect(screen.getByText('日期范围')).toBeInTheDocument();
  });
});
