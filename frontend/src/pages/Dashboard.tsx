import { Card, Table } from 'antd';
import { useEffect, useState } from 'react';

interface StockQuote {
  code: string;
  name: string;
  price: number;
  change_percent: number;
}

function Dashboard() {
  const [data, setData] = useState<StockQuote[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // TODO: 从 API 获取数据
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
    { title: '涨幅(%)', dataIndex: 'change_percent', key: 'change_percent' },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Card title="实时行情">
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
