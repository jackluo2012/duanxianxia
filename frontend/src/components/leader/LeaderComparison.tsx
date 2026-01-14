import { Card, Table, Empty, Tag, Button } from 'antd';
import { DeleteOutlined } from '@ant-design/icons';
import { useLeaderStore } from '../../store/leaderStore';
import type { LeaderStock } from '../../types/leader';

function LeaderComparison() {
  const { comparedStocks, removeComparedStock, clearComparedStocks } = useLeaderStore();

  if (comparedStocks.length === 0) {
    return (
      <Card title="🆚 龙头对比分析" style={{ marginTop: 16 }}>
        <Empty description="点击排行榜中的【对比】按钮添加股票" />
      </Card>
    );
  }

  const columns = [
    {
      title: '股票代码',
      dataIndex: 'code',
      key: 'code',
      width: 100,
    },
    {
      title: '股票名称',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: '连板天数',
      dataIndex: 'consecutive_limit_up',
      key: 'consecutive_limit_up',
      sorter: (a: LeaderStock, b: LeaderStock) =>
        (a as any).consecutive_limit_up - (b as any).consecutive_limit_up,
      render: (days: number) => (
        <Tag color={days >= 7 ? 'error' : days >= 5 ? 'warning' : 'processing'}>
          {days}板
        </Tag>
      ),
    },
    {
      title: '市值(亿)',
      dataIndex: 'market_cap',
      key: 'market_cap',
      sorter: (a: LeaderStock, b: LeaderStock) => a.market_cap - b.market_cap,
      render: (value: number) => value.toFixed(2),
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
    },
    {
      title: '操作',
      key: 'action',
      width: 80,
      render: (_: any, record: LeaderStock) => (
        <Button
          type="text"
          danger
          size="small"
          icon={<DeleteOutlined />}
          onClick={() => removeComparedStock(record.code)}
        />
      ),
    },
  ];

  return (
    <Card
      title="🆚 龙头对比分析"
      style={{ marginTop: 16 }}
      extra={
        <Button size="small" onClick={clearComparedStocks}>
          清空对比
        </Button>
      }
    >
      <Table
        columns={columns}
        dataSource={comparedStocks}
        rowKey="code"
        size="small"
        pagination={false}
      />
    </Card>
  );
}

export default LeaderComparison;
