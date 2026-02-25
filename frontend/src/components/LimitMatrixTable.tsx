import React, { useMemo } from 'react';
import './LimitMatrixTable.css';
import { TableExportButton } from './table/TableExportButton';

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

interface ExportItem {
  stock: string;
  level: string;
  theme: string;
}

export const LimitMatrixTable: React.FC<LimitMatrixTableProps> = ({ data }) => {
  const { tradeDate, limitData } = data;

  // 获取所有题材
  const allThemes = useMemo(() => {
    const themes = new Set<string>();
    limitData.forEach(level => {
      Object.keys(level.themes).forEach(theme => themes.add(theme));
    });
    return Array.from(themes);
  }, [limitData]);

  const exportData = useMemo(() => {
    const items: ExportItem[] = [];
    limitData.forEach(level => {
      Object.entries(level.themes).forEach(([theme, stocks]) => {
        stocks.forEach(stock => {
          items.push({
            stock,
            level: `${level.consecutiveLevel}板`,
            theme,
          });
        });
      });
    });
    return items;
  }, [limitData]);

  const exportColumns = [
    { title: '股票名称', dataIndex: 'stock', key: 'stock' },
    { title: '连板数', dataIndex: 'level', key: 'level' },
    { title: '题材', dataIndex: 'theme', key: 'theme' },
  ];

  return (
    <div className="limit-matrix-table">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 20 }}>
        <h3 style={{ margin: 0 }}>📊 {tradeDate} 涨停板梯队矩阵</h3>
        <TableExportButton
          data={exportData}
          columns={exportColumns}
          tableType="matrix"
          filename={`涨停梯队_${tradeDate}`}
          disabled={limitData.length === 0}
        />
      </div>

      {limitData.length === 0 ? (
        <div className="no-data">暂无数据</div>
      ) : (
        <>
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
        </>
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
