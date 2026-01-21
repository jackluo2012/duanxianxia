import { render, screen } from '@testing-library/react';
import { ThemeDetailCard } from './ThemeDetailCard';

describe('ThemeDetailCard', () => {
  it('renders theme detail correctly', () => {
    const mockTheme = {
      themeName: '人工智能',
      limitUpCount: 8,
      cycleStage: 'climax' as const,
      stocks: [
        { role: 'leader' as const, code: '300001', name: '龙头A' },
        { role: 'mid' as const, code: '300002', name: '中军B' },
        { role: 'follower' as const, code: '300003', name: '跟风C' },
      ]
    };

    render(<ThemeDetailCard theme={mockTheme} />);

    expect(screen.getByText('人工智能')).toBeInTheDocument();
    expect(screen.getByText('8只涨停')).toBeInTheDocument();
    expect(screen.getByText('高潮期')).toBeInTheDocument();
    expect(screen.getByText('龙头A')).toBeInTheDocument();
    expect(screen.getByText('中军B')).toBeInTheDocument();
    expect(screen.getByText('跟风C')).toBeInTheDocument();
  });

  it('renders empty stocks list', () => {
    const mockTheme = {
      themeName: '测试题材',
      limitUpCount: 0,
      cycleStage: 'init' as const,
      stocks: []
    };

    render(<ThemeDetailCard theme={mockTheme} />);

    expect(screen.getByText('测试题材')).toBeInTheDocument();
    expect(screen.getByText('0只涨停')).toBeInTheDocument();
    expect(screen.getByText('启动期')).toBeInTheDocument();
  });

  it('groups stocks by role correctly', () => {
    const mockTheme = {
      themeName: '新能源汽车',
      limitUpCount: 5,
      cycleStage: 'fermentation' as const,
      stocks: [
        { role: 'leader' as const, code: '001', name: '龙头1' },
        { role: 'leader' as const, code: '002', name: '龙头2' },
        { role: 'mid' as const, code: '003', name: '中军1' },
      ]
    };

    const { container } = render(<ThemeDetailCard theme={mockTheme} />);

    expect(container.textContent).toContain('龙头');
    expect(container.textContent).toContain('中军');
    expect(screen.getByText('龙头1')).toBeInTheDocument();
    expect(screen.getByText('龙头2')).toBeInTheDocument();
    expect(screen.getByText('中军1')).toBeInTheDocument();
  });
});
