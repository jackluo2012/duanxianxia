import { Spin, Empty, Alert } from 'antd';
import { FixedSizeList } from 'react-window';
import { useLeaderBoard } from '../../hooks/useLeader';
import { useLeaderStore } from '../../store/leaderStore';
import LeaderItem from './LeaderItem';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderBoardProps {
  onStockSelect: (item: LeaderBoardItem) => void;
  onAddCompare: (item: LeaderBoardItem) => void;
}

function LeaderBoard({ onStockSelect, onAddCompare }: LeaderBoardProps) {
  const { filters } = useLeaderStore();
  const { data, isLoading, error } = useLeaderBoard(filters);

  if (isLoading) {
    return (
      <div style={{ textAlign: 'center', padding: '50px 0' }}>
        <Spin size="large" tip="加载中..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        message="加载失败"
        description="获取排行榜数据失败,请稍后重试"
        type="error"
        showIcon
        style={{ margin: 16 }}
      />
    );
  }

  if (!data || data.items.length === 0) {
    return (
      <Empty
        description="暂无数据"
        style={{ marginTop: 50 }}
      />
    );
  }

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = data.items[index];
    const isSelected = false;  // TODO: 从store获取选中状态

    return (
      <LeaderItem
        item={item}
        isSelected={isSelected}
        onSelect={onStockSelect}
        onAddCompare={onAddCompare}
        style={style}
      />
    );
  };

  return (
    <div>
      <div style={{ marginBottom: 16, color: '#8c8c8c', fontSize: 14 }}>
        共找到 <span style={{ color: '#1890ff', fontWeight: 'bold' }}>{data.total}</span> 只连板股票
      </div>

      <FixedSizeList
        height={600}
        itemCount={data.items.length}
        itemSize={100}
        width="100%"
      >
        {Row}
      </FixedSizeList>
    </div>
  );
}

export default LeaderBoard;
