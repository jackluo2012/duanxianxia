import React from 'react';
import './LimitMatrixTable.css';

interface LimitData {
  consecutiveLevel: number;
  themes: Record<string, string[]>;
}

interface LimitMatrixTableProps {
  data: {
    tradeDate: string;
    limitData: LimitData[];
  };
}

export const LimitMatrixTable: React.FC<LimitMatrixTableProps> = ({ data }) => {
  const { tradeDate, limitData } = data;

  // 获取所有题材
  const allThemes = React.useMemo(() => {
    const themes = new Set<string>();
    limitData.forEach(level => {
      Object.keys(level.themes).forEach(theme => themes.add(theme));
    });
    return Array.from(themes);
  }, [limitData]);

  return (
    <div className="limit-matrix-table">
      <h3>📊 {tradeDate} 涨停板梯队矩阵</h3>

      {limitData.length === 0 ? (
        <div className="no-data">暂无数据</div>
      ) : (
        <table>
          <thead>
            <tr>
              <th>板数</th>
              {allThemes.map(theme => (
                <th key={theme}>{theme}</th>
              ))}
              <th>合计</th>
            </tr>
          </thead>
          <tbody>
            {limitData.map((level) => (
              <tr key={level.consecutiveLevel}>
                <td className="consecutive-level">{level.consecutiveLevel}板</td>
                {allThemes.map(theme => (
                  <td key={theme}>
                    {renderCell(level.themes[theme] || [])}
                  </td>
                ))}
                <td className="row-total">{calculateRowTotal(level.themes)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};

function renderCell(stocks: string[]): React.ReactNode {
  if (stocks.length === 0) {
    return <span className="empty-cell">-</span>;
  }

  if (stocks.length <= 3) {
    return (
      <div className="stock-list">
        {stocks.map((stock, idx) => (
          <span key={idx} className="stock-name">{stock}</span>
        ))}
      </div>
    );
  }

  return <span className="stock-count">({stocks.length}只)</span>;
}

function calculateRowTotal(themes: Record<string, string[]>): number {
  return Object.values(themes).reduce((sum, stocks) => sum + stocks.length, 0);
}
