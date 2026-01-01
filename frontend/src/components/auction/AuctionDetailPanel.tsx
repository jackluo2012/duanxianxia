import { Card, Col, Descriptions, Empty, Progress, Row, Statistic, Tag } from 'antd';
import AuctionChart from './AuctionChart';

interface AuctionStock {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  sealed_amount_buy: number;
  sealed_amount_sell: number;
  intensity_score?: number;
}

interface AuctionDetailPanelProps {
  stock: AuctionStock | null;
}

function AuctionDetailPanel({ stock }: AuctionDetailPanelProps) {
  if (!stock) {
    return (
      <Card
        title="股票详情"
        style={{ height: '100%', minHeight: 600 }}
        styles={{ body: { display: 'flex', alignItems: 'center', justifyContent: 'center' } }}
      >
        <Empty description="请从左侧选择一只股票查看详情" />
      </Card>
    );
  }

  const formatAmount = (amount: number) => {
    if (amount >= 100000000) {
      return `${(amount / 100000000).toFixed(2)}亿`;
    } else if (amount >= 10000) {
      return `${(amount / 10000).toFixed(2)}万`;
    }
    return amount.toFixed(2);
  };

  const totalSealed = stock.sealed_amount_buy + stock.sealed_amount_sell;
  const buyRatio = totalSealed > 0 ? (stock.sealed_amount_buy / totalSealed) * 100 : 0;
  const sellRatio = totalSealed > 0 ? (stock.sealed_amount_sell / totalSealed) * 100 : 0;

  return (
    <Card
      title={`${stock.name} (${stock.code})`}
      style={{ height: '100%', minHeight: 600 }}
    >
      <Row gutter={[16, 16]}>
        {/* 核心指标 */}
        <Col span={24}>
          <Row gutter={16}>
            <Col span={8}>
              <Statistic
                title="当前价格"
                value={stock.price}
                precision={2}
                prefix="¥"
                valueStyle={{
                  color: stock.change_percent >= 0 ? '#cf1322' : '#3f8600',
                  fontSize: 24,
                }}
              />
            </Col>
            <Col span={8}>
              <Statistic
                title="涨跌幅"
                value={stock.change_percent}
                precision={2}
                suffix="%"
                valueStyle={{
                  color: stock.change_percent >= 0 ? '#cf1322' : '#3f8600',
                  fontSize: 20,
                }}
              />
            </Col>
            {stock.intensity_score !== undefined && (
              <Col span={8}>
                <Statistic
                  title="强度评分"
                  value={stock.intensity_score}
                  precision={1}
                  suffix="/ 100"
                  valueStyle={{
                    color: stock.intensity_score >= 80 ? '#cf1322' : stock.intensity_score >= 60 ? '#faad14' : '#52c41a',
                    fontSize: 20,
                  }}
                />
              </Col>
            )}
          </Row>
        </Col>

        {/* 封单分析 */}
        <Col span={24}>
          <Card title="封单分析" size="small">
            <Row gutter={16}>
              <Col span={12}>
                <Descriptions column={1} size="small">
                  <Descriptions.Item label="买封金额">
                    <Tag color="red" style={{ fontSize: 16, padding: '4px 12px' }}>
                      {formatAmount(stock.sealed_amount_buy)}
                    </Tag>
                  </Descriptions.Item>
                  <Descriptions.Item label="卖封金额">
                    <Tag color="green" style={{ fontSize: 16, padding: '4px 12px' }}>
                      {formatAmount(stock.sealed_amount_sell)}
                    </Tag>
                  </Descriptions.Item>
                </Descriptions>
              </Col>
              <Col span={12}>
                <div style={{ marginTop: 8 }}>
                  <div style={{ marginBottom: 8, fontSize: 14, fontWeight: 'bold' }}>
                    买卖比例
                  </div>
                  <Progress
                    percent={parseFloat(buyRatio.toFixed(2))}
                    strokeColor="#cf1322"
                    showInfo={true}
                    format={(percent) => `买 ${percent?.toFixed(1)}%`}
                  />
                  <Progress
                    percent={parseFloat(sellRatio.toFixed(2))}
                    strokeColor="#3f8600"
                    showInfo={true}
                    format={(percent) => `卖 ${percent?.toFixed(1)}%`}
                    style={{ marginTop: 8 }}
                  />
                </div>
              </Col>
            </Row>
          </Card>
        </Col>

        {/* 竞价曲线图 */}
        <Col span={24}>
          <Card title="竞价曲线" size="small" styles={{ body: { padding: 12 } }}>
            <AuctionChart code={stock.code} />
          </Card>
        </Col>
      </Row>
    </Card>
  );
}

export default AuctionDetailPanel;
