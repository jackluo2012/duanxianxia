import { Card, Row, Col, Typography, Tag, Button, Space } from 'antd';
import { ArrowUpOutlined, PlusOutlined } from '@ant-design/icons';
import type { LeaderBoardItem } from '../../types/leader';

const { Text } = Typography;

interface LeaderItemProps {
  item: LeaderBoardItem;
  isSelected: boolean;
  onSelect: (item: LeaderBoardItem) => void;
  onAddCompare: (item: LeaderBoardItem) => void;
  style?: React.CSSProperties;
}

function LeaderItem({ item, isSelected, onSelect, onAddCompare, style }: LeaderItemProps) {
  const getConsecutiveColor = (days: number) => {
    if (days >= 9) return '#f5222d';  // 红色
    if (days >= 6) return '#fa8c16';  // 橙色
    return '#1890ff';  // 蓝色
  };

  return (
    <Card
      hoverable
      style={{
        marginBottom: 8,
        border: isSelected ? '2px solid #1890ff' : '1px solid #f0f0f0',
        ...style,
      }}
      onClick={() => onSelect(item)}
    >
      <Row align="middle" gutter={16}>
        <Col span={6}>
          <Space direction="vertical" size={0}>
            <Text strong>{item.name}</Text>
            <Text type="secondary" style={{ fontSize: 12 }}>{item.code}</Text>
          </Space>
        </Col>

        <Col span={4}>
          <Tag
            color={getConsecutiveColor(item.consecutive_limit_up)}
            style={{ fontSize: 14, padding: '4px 12px' }}
          >
            {item.consecutive_limit_up}连板
            <ArrowUpOutlined style={{ marginLeft: 4 }} />
          </Tag>
        </Col>

        <Col span={5}>
          <Space direction="vertical" size={0}>
            <Text strong style={{ color: '#f5222d' }}>
              ¥{item.price.toFixed(2)}
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              最高: {item.history_max}板
            </Text>
          </Space>
        </Col>

        <Col span={5}>
          <Space direction="vertical" size={0}>
            <Text style={{ color: '#f5222d' }}>
              +{item.change_percent.toFixed(2)}%
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              封单: {(item.sealed_amount / 100000000).toFixed(2)}亿
            </Text>
          </Space>
        </Col>

        <Col span={4}>
          <Button
            type="primary"
            size="small"
            icon={<PlusOutlined />}
            onClick={(e) => {
              e.stopPropagation();
              onAddCompare(item);
            }}
            block
          >
            对比
          </Button>
        </Col>
      </Row>
    </Card>
  );
}

export default LeaderItem;
