import { Card, Col, Row, Tabs, Tag, Typography } from 'antd';
import { useState } from 'react';
import AuctionRankingList from '../components/auction/AuctionRankingList';
import AuctionDetailPanel from '../components/auction/AuctionDetailPanel';
import AlertConfig from '../components/auction/AlertConfig';
import AlertHistory from '../components/auction/AlertHistory';

const { Title } = Typography;

interface AuctionStock {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  sealed_amount_buy: number;
  sealed_amount_sell: number;
  intensity_score?: number;
}

function AuctionDashboard() {
  const [selectedStock, setSelectedStock] = useState<AuctionStock | null>(null);
  const [activeTab, setActiveTab] = useState('buy_sealed');
  const [activeSection, setActiveSection] = useState('rankings');

  const handleStockSelect = (stock: AuctionStock) => {
    setSelectedStock(stock);
  };

  const rankingTabItems = [
    {
      key: 'buy_sealed',
      label: '买封排行',
    },
    {
      key: 'intensity',
      label: '强度排行',
    },
    {
      key: 'change',
      label: '涨幅排行',
    },
    {
      key: 'anomaly',
      label: '异动排行',
    },
  ];

  const mainTabItems = [
    {
      key: 'rankings',
      label: '排行榜',
    },
    {
      key: 'alerts_config',
      label: '告警配置',
    },
    {
      key: 'alerts_history',
      label: '告警历史',
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2} style={{ marginBottom: 24 }}>
        竞价分析
      </Title>

      {/* 主导航 Tab */}
      <Tabs
        activeKey={activeSection}
        onChange={setActiveSection}
        items={mainTabItems}
        size="large"
        style={{ marginBottom: 16 }}
      />

      {activeSection === 'rankings' && (
        <Row gutter={[16, 16]}>
          {/* 左侧排行榜 */}
          <Col span={14}>
            <Card
              title="竞价排行榜"
              extra={
                <Tag color="processing">
                  竞价时段: 9:15-9:25
                </Tag>
              }
            >
              <Tabs
                activeKey={activeTab}
                onChange={setActiveTab}
                items={rankingTabItems}
                size="large"
              />

              <AuctionRankingList
                rankingType={activeTab}
                onStockSelect={handleStockSelect}
                selectedCode={selectedStock?.code}
              />
            </Card>
          </Col>

          {/* 右侧详情面板 */}
          <Col span={10}>
            <AuctionDetailPanel stock={selectedStock} />
          </Col>
        </Row>
      )}

      {activeSection === 'alerts_config' && (
        <AlertConfig />
      )}

      {activeSection === 'alerts_history' && (
        <AlertHistory />
      )}
    </div>
  );
}

export default AuctionDashboard;
