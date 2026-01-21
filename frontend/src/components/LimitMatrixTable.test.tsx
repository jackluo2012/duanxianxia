import { render, screen } from '@testing-library/react';
import { LimitMatrixTable } from './LimitMatrixTable';

describe('LimitMatrixTable', () => {
  it('renders matrix table correctly', () => {
    const mockData = {
      tradeDate: '2025-01-16',
      limitData: [
        {
          consecutiveLevel: 8,
          themes: {
            '人工智能': ['龙头A', '龙头B'],
            '芯片': ['强势C'],
          }
        },
        {
          consecutiveLevel: 7,
          themes: {
            '人工智能': [],
            '芯片': ['强势D'],
          }
        }
      ]
    };

    render(<LimitMatrixTable data={mockData} />);

    expect(screen.getByText('8板')).toBeInTheDocument();
    expect(screen.getByText('7板')).toBeInTheDocument();
    expect(screen.getByText('龙头A')).toBeInTheDocument();
    expect(screen.getByText('人工智能')).toBeInTheDocument();
    expect(screen.getByText('芯片')).toBeInTheDocument();
  });

  it('renders empty table when no data', () => {
    const mockData = {
      tradeDate: '2025-01-16',
      limitData: []
    };

    render(<LimitMatrixTable data={mockData} />);

    expect(screen.getByText(/2025-01-16.*涨停板梯队矩阵/)).toBeInTheDocument();
    expect(screen.getByText('暂无数据')).toBeInTheDocument();
  });

  it('calculates row totals correctly', () => {
    const mockData = {
      tradeDate: '2025-01-16',
      limitData: [
        {
          consecutiveLevel: 5,
          themes: {
            '题材A': ['股票1', '股票2'],
            '题材B': ['股票3'],
          }
        }
      ]
    };

    render(<LimitMatrixTable data={mockData} />);

    // 合计应该是3只股票
    expect(screen.getByText('3')).toBeInTheDocument();
  });
});
