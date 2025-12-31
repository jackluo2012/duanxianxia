import { Card, Table, Tag } from 'antd';
import { useEffect, useState } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';

interface StockQuote {
  code: string;
  name: string;
  price: number;
  change_percent: number;
}

function Dashboard() {
  const [data, setData] = useState<StockQuote[]>([]);
  const [loading, setLoading] = useState(true);

  const { status, subscribe } = useWebSocket('ws://localhost:8080/ws/realtime', {
    onMessage: (message) => {
      if (message.type === 'quote_update') {
        const quote = message.data;
        setData((prevData) => {
          const index = prevData.findIndex((item) => item.code === quote.code);
          if (index >= 0) {
            const newData = [...prevData];
            newData[index] = quote;
            return newData;
          } else {
            return [...prevData, quote];
          }
        });
      }
    },
  });

  useEffect(() => {
    // 当 WebSocket 连接成功后订阅股票
    if (status === 'connected') {
      subscribe(['000001', '600000']);
    }
  }, [status, subscribe]);

  useEffect(() => {
    // 初始数据
    setTimeout(() => {
      setData([
        { code: '000001', name: '平安银行', price: 12.50, change_percent: 2.5 },
        { code: '600000', name: '浦发银行', price: 8.30, change_percent: 1.2 },
      ]);
      setLoading(false);
    }, 1000);
  }, []);

  const columns = [
    { title: '代码', dataIndex: 'code', key: 'code' },
    { title: '名称', dataIndex: 'name', key: 'name' },
    { title: '价格', dataIndex: 'price', key: 'price' },
    {
      title: '涨幅(%)',
      dataIndex: 'change_percent',
      key: 'change_percent',
      render: (value: number) => (
        <Tag color={value >= 0 ? 'red' : 'green'}>{value}%</Tag>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Card title="实时行情" extra={<span>WebSocket: {status}</span>}>
        <Table
          columns={columns}
          dataSource={data}
          loading={loading}
          rowKey="code"
        />
      </Card>
    </div>
  );
}

export default Dashboard;
