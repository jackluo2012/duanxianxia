import { Alert, Card, Col, Row, Statistic, Table, Tag, Space, Button, Typography } from 'antd';
import { ReloadOutlined, SyncOutlined } from '@ant-design/icons';
import { useQuoteData } from '../hooks/useQuoteData';
import KLineChartAdvanced from '../components/charts/KLineChartAdvanced';
import PeriodSelector from '../components/PeriodSelector';
import StockSelector from '../components/StockSelector';

const { Text } = Typography;

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
    refresh,
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
      dataIndex: 'volume',
      key: 'volume',
      width: 120,
      render: (value: number) => value ? value.toLocaleString() : '-',
    },
    {
      title: '时间',
      dataIndex: 'datetime',
      key: 'datetime',
      width: 100,
    },
  ];

  // 计算涨跌额
  const changeAmount = realtimeQuote
    ? (realtimeQuote.price - realtimeQuote.preclose).toFixed(2)
    : '0.00';

  // 格式化成交量
  const formatVolume = (vol: number | undefined) => {
    if (vol === undefined || vol === null || isNaN(vol)) {
      return '-';
    }
    if (vol >= 100000000) {
      return `${(vol / 100000000).toFixed(2)}亿`;
    } else if (vol >= 10000) {
      return `${(vol / 10000).toFixed(2)}万`;
    }
    return vol.toString();
  };

  // 技术指标配置
  const indicators = [
    { type: 'ma' as const, period: 5, color: '#f5222d' },
    { type: 'ma' as const, period: 10, color: '#fa8c16' },
    { type: 'ma' as const, period: 20, color: '#52c41a' },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
        <Col span={18}>
          <Card
            title="实时行情"
            extra={
              <Space size="middle">
                <span>
                  WebSocket:
                  <Tag color={wsStatus === 'connected' ? 'success' : 'error'}>
                    {wsStatus === 'connected' ? '已连接' : '未连接'}
                  </Tag>
                </span>
                <PeriodSelector value={period} onChange={selectPeriod} disabled={loading} />
                <StockSelector value={selectedCode} onChange={selectStock} loading={loading} />
                <Button
                  icon={<ReloadOutlined />}
                  onClick={refresh}
                  loading={loading}
                  size="small"
                >
                  刷新
                </Button>
              </Space>
            }
          >
            {error && (
              <Alert
                message="数据加载异常"
                description={error}
                type="warning"
                showIcon
                closable
                style={{ marginBottom: 16 }}
              />
            )}

            {/* 实时行情统计 */}
            <Row gutter={16} style={{ marginBottom: 16, padding: '12px', background: '#fafafa', borderRadius: 4 }}>
              <Col span={4}>
                <Statistic
                  title={<Text type="secondary">股票代码</Text>}
                  value={realtimeQuote?.code || '-'}
                  valueStyle={{ fontSize: 16, fontWeight: 'bold' }}
                />
              </Col>
              <Col span={4}>
                <Statistic
                  title={<Text type="secondary">股票名称</Text>}
                  value={realtimeQuote?.name || '-'}
                  valueStyle={{ fontSize: 16, fontWeight: 'bold' }}
                />
              </Col>
              <Col span={5}>
                <Statistic
                  title={<Text type="secondary">当前价格</Text>}
                  value={realtimeQuote?.price || 0}
                  precision={2}
                  prefix={realtimeQuote && realtimeQuote.change_percent >= 0 ? '↑' : '↓'}
                  valueStyle={{
                    fontSize: 28,
                    fontWeight: 'bold',
                    color: (realtimeQuote?.change_percent || 0) >= 0 ? '#cf1322' : '#3f8600',
                  }}
                />
              </Col>
              <Col span={4}>
                <Statistic
                  title={<Text type="secondary">涨跌幅</Text>}
                  value={realtimeQuote?.change_percent || 0}
                  precision={2}
                  suffix="%"
                  valueStyle={{
                    fontSize: 20,
                    fontWeight: 'bold',
                    color: (realtimeQuote?.change_percent || 0) >= 0 ? '#cf1322' : '#3f8600',
                  }}
                />
              </Col>
              <Col span={4}>
                <Statistic
                  title={<Text type="secondary">涨跌额</Text>}
                  value={changeAmount}
                  valueStyle={{
                    fontSize: 18,
                    color: parseFloat(changeAmount) >= 0 ? '#cf1322' : '#3f8600',
                  }}
                />
              </Col>
              <Col span={3}>
                <Statistic
                  title={<Text type="secondary">成交量</Text>}
                  value={realtimeQuote ? formatVolume(realtimeQuote.volume) : '-'}
                  valueStyle={{ fontSize: 16 }}
                />
              </Col>
            </Row>

            {/* 增强的K线图表 */}
            <KLineChartAdvanced
              data={klineData}
              period={period}
              loading={loading}
              height={450}
              indicators={indicators}
              enableZoom={true}
              enableDataZoom={true}
              showVolume={true}
            />
          </Card>
        </Col>

        <Col span={6}>
          <Card
            title="最新行情"
            extra={
              wsStatus === 'connected' && (
                <SyncOutlined spin style={{ color: '#52c41a', fontSize: 16 }} />
              )
            }
            style={{ height: '100%' }}
          >
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

            {/* 额外信息 */}
            {realtimeQuote && (
              <div style={{ marginTop: 16, padding: 12, background: '#f5f5f5', borderRadius: 4 }}>
                <Space direction="vertical" size="small" style={{ width: '100%' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Text type="secondary">今开:</Text>
                    <Text strong>{realtimeQuote.open.toFixed(2)}</Text>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Text type="secondary">最高:</Text>
                    <Text strong style={{ color: '#cf1322' }}>
                      {realtimeQuote.high.toFixed(2)}
                    </Text>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Text type="secondary">最低:</Text>
                    <Text strong style={{ color: '#3f8600' }}>
                      {realtimeQuote.low.toFixed(2)}
                    </Text>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Text type="secondary">成交额:</Text>
                    <Text strong>
                      {realtimeQuote.amount
                        ? `${(realtimeQuote.amount / 100000000).toFixed(2)}亿`
                        : '-'}
                    </Text>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Text type="secondary">昨收:</Text>
                    <Text>{realtimeQuote.preclose.toFixed(2)}</Text>
                  </div>
                </Space>
              </div>
            )}
          </Card>
        </Col>
      </Row>
    </div>
  );
}

export default Dashboard;
