import { Alert, Card, Col, Row, Statistic, Table, Tag } from 'antd';
import { useQuoteData } from '../hooks/useQuoteData';
import KLineChart from '../components/KLineChart';
import PeriodSelector from '../components/PeriodSelector';
import StockSelector from '../components/StockSelector';

function Dashboard() {
  const {
    selectedCode,
    period,
    klineData,
    realtimeQuote,
    loading,
    error,
    wsStatus,
    selectStock,
    selectPeriod,
  } = useQuoteData('000001', '1m');

  const columns = [
    { title: '代码', dataIndex: 'code', key: 'code', width: 120 },
    { title: '名称', dataIndex: 'name', key: 'name', width: 150 },
    {
      title: '现价',
      dataIndex: 'price',
      key: 'price',
      width: 100,
      render: (value: number) => value.toFixed(2),
    },
    {
      title: '涨跌幅(%)',
      dataIndex: 'change_percent',
      key: 'change_percent',
      width: 120,
      render: (value: number) => (
        <Tag color={value >= 0 ? 'red' : 'green'}>{value.toFixed(2)}%</Tag>
      ),
    },
    {
      title: '成交量',
      dataIndex: 'vol',
      key: 'vol',
      width: 120,
      render: (value: number) => value.toLocaleString(),
    },
    {
      title: '时间',
      dataIndex: 'datetime',
      key: 'datetime',
      width: 100,
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
        <Col span={18}>
          <Card
            title="实时行情"
            extra={
              <div style={{ display: 'flex', gap: 16, alignItems: 'center' }}>
                <span>
                  WebSocket:
                  <Tag color={wsStatus === 'connected' ? 'success' : 'error'}>
                    {wsStatus}
                  </Tag>
                </span>
                <PeriodSelector value={period} onChange={selectPeriod} disabled={loading} />
                <StockSelector value={selectedCode} onChange={selectStock} loading={loading} />
              </div>
            }
          >
            {error && (
              <Alert
                message="警告"
                description={error}
                type="warning"
                showIcon
                style={{ marginBottom: 16 }}
              />
            )}

            <Row gutter={16} style={{ marginBottom: 16 }}>
              <Col span={6}>
                <Statistic
                  title="股票代码"
                  value={realtimeQuote?.code || '-'}
                  valueStyle={{ fontSize: 18 }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title="股票名称"
                  value={realtimeQuote?.name || '-'}
                  valueStyle={{ fontSize: 18 }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title="当前价格"
                  value={realtimeQuote?.price || 0}
                  precision={2}
                  valueStyle={{
                    fontSize: 24,
                    color: (realtimeQuote?.change_percent || 0) >= 0 ? '#cf1322' : '#3f8600',
                  }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title="涨跌幅"
                  value={realtimeQuote?.change_percent || 0}
                  precision={2}
                  suffix="%"
                  valueStyle={{
                    fontSize: 20,
                    color: (realtimeQuote?.change_percent || 0) >= 0 ? '#cf1322' : '#3f8600',
                  }}
                />
              </Col>
            </Row>

            <KLineChart data={klineData} period={period} loading={loading} />
          </Card>
        </Col>

        <Col span={6}>
          <Card title="最新行情" style={{ height: '100%' }}>
            {realtimeQuote && (
              <Table
                columns={columns}
                dataSource={[realtimeQuote]}
                pagination={false}
                rowKey="code"
                size="small"
                showHeader={false}
              />
            )}
          </Card>
        </Col>
      </Row>
    </div>
  );
}

export default Dashboard;
