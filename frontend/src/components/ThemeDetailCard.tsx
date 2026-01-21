import React from 'react';
import './ThemeDetailCard.css';

export type StockRole = 'leader' | 'mid' | 'follower';

export type CycleStage = 'init' | 'fermentation' | 'climax' | 'differentiation' | 'recession';

export interface Stock {
  role: StockRole;
  code: string;
  name: string;
}

export interface ThemeDetail {
  themeName: string;
  limitUpCount: number;
  cycleStage: CycleStage;
  stocks: Stock[];
}

interface ThemeDetailCardProps {
  theme: ThemeDetail;
}

const cycleStageMap: Record<CycleStage, string> = {
  init: '启动期',
  fermentation: '发酵期',
  climax: '高潮期',
  differentiation: '分化期',
  recession: '衰退期',
};

const cycleStageColorMap: Record<CycleStage, string> = {
  init: '#52c41a',      // 绿色
  fermentation: '#1890ff', // 蓝色
  climax: '#f5222d',      // 红色
  differentiation: '#fa8c16', // 橙色
  recession: '#8c8c8c',     // 灰色
};

export const ThemeDetailCard: React.FC<ThemeDetailCardProps> = ({ theme }) => {
  const leaderStocks = theme.stocks.filter(s => s.role === 'leader');
  const midStocks = theme.stocks.filter(s => s.role === 'mid');
  const followerStocks = theme.stocks.filter(s => s.role === 'follower');

  const stageColor = cycleStageColorMap[theme.cycleStage];

  return (
    <div className="theme-detail-card">
      <div className="theme-header">
        <h3 className="theme-name">{theme.themeName}</h3>
        <div className="theme-stats">
          <span className="stat-item">{theme.limitUpCount}只涨停</span>
          <span
            className="stat-item stage-badge"
            style={{ backgroundColor: stageColor }}
          >
            {cycleStageMap[theme.cycleStage]}
          </span>
        </div>
      </div>

      {theme.stocks.length === 0 ? (
        <div className="no-stocks">暂无股票数据</div>
      ) : (
        <div className="stock-sections">
          {leaderStocks.length > 0 && (
            <div className="stock-section">
              <h4 className="section-title leader-title">👑 龙头</h4>
              <ul className="stock-list">
                {leaderStocks.map(stock => (
                  <li key={stock.code} className="stock-item leader">
                    <span className="stock-name">{stock.name}</span>
                    <span className="stock-code">{stock.code}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {midStocks.length > 0 && (
            <div className="stock-section">
              <h4 className="section-title mid-title">⚔️ 中军</h4>
              <ul className="stock-list">
                {midStocks.map(stock => (
                  <li key={stock.code} className="stock-item mid">
                    <span className="stock-name">{stock.name}</span>
                    <span className="stock-code">{stock.code}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {followerStocks.length > 0 && (
            <div className="stock-section">
              <h4 className="section-title follower-title">🌊 跟风</h4>
              <ul className="stock-list">
                {followerStocks.map(stock => (
                  <li key={stock.code} className="stock-item follower">
                    <span className="stock-name">{stock.name}</span>
                    <span className="stock-code">{stock.code}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
