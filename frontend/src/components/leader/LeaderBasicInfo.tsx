import { Card, Descriptions, Tag, Statistic, Row, Col } from 'antd';
import { RiseOutlined, FireOutlined, DollarOutlined } from '@ant-design/icons';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderBasicInfoProps {
  stock: LeaderBoardItem | null;
}

function LeaderBasicInfo({ stock }: LeaderBasicInfoProps) {
  if (!stock) {
    return (
      <Card>
        <div style={{ textAlign: 'center', color: '#8c8c8c', padding: '50px 0' }}>
          请选择一只股票查看详情
        </div>
      </Card>
    );
  }

  const getConsecutiveColor = (days: number) => {
    if (days >= 9) return 'error';
    if (days >= 6) return 'warning';
    return 'processing';
  };

  return (
    <Card
      title={
        <span>
          {stock.name}
          <Tag color={getConsecutiveColor(stock.consecutive_limit_up)} style={{ marginLeft: 8 }}>
            {stock.consecutive_limit_up}连板
          </Tag>
        </span>
      }
      extra={<span style={{ fontSize: 12, color: '#8c8c8c' }}>{stock.code}</span>}
    >
      <Row gutter={16}>
        <Col span={8}>
          <Statistic
            title="当前价格"
            value={stock.price}
            precision={2}
            prefix="¥"
            valueStyle={{ color: '#f5222d' }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="涨幅"
            value={stock.change_percent}
            precision={2}
            suffix="%"
            valueStyle={{ color: '#f5222d' }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="市值"
            value={stock.market_cap}
            precision={2}
            suffix="亿"
          />
        </Col>
      </Row>

      <Descriptions column={2} size="small" style={{ marginTop: 16 }}>
        <Descriptions.Item label="历史最高连板">
          <Tag color="gold">{stock.history_max}板</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="封单金额">
          <Tag icon={<DollarOutlined />} color="cyan">
            {(stock.sealed_amount / 100000000).toFixed(2)}亿
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label="所属板块" span={2}>
          {stock.sector}
        </Descriptions.Item>
      </Descriptions>
    </Card>
  );
}

export default LeaderBasicInfo;
