import { Row, Col, Typography } from 'antd';
import { useLeaderStore } from '../store/leaderStore';
import FilterBar from '../components/leader/FilterBar';
import LeaderBoard from '../components/leader/LeaderBoard';
import LeaderDetail from '../components/leader/LeaderDetail';
import type { LeaderBoardItem } from '../types/leader';

const { Title } = Typography;

function LeaderPage() {
  const { selectedStock, setSelectedStock, addComparedStock } = useLeaderStore();

  const handleStockSelect = (item: LeaderBoardItem) => {
    setSelectedStock(item);
  };

  const handleAddCompare = (item: LeaderBoardItem) => {
    addComparedStock(item);
  };

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2} style={{ marginBottom: 24 }}>
        龙头高度
      </Title>

      <Row gutter={16}>
        <Col span={14}>
          <FilterBar />
          <LeaderBoard
            onStockSelect={handleStockSelect}
            onAddCompare={handleAddCompare}
          />
        </Col>

        <Col span={10}>
          <LeaderDetail stock={selectedStock} />
        </Col>
      </Row>
    </div>
  );
}

export default LeaderPage;
