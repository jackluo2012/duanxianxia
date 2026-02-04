/**
 * 概念板块分析页面
 * 包含板块热度排行、板块列表、成分股详情等功能
 */

import { useState, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Table,
  Input,
  Button,
  Select,
  Tag,
  Statistic,
  Typography,
  Space,
  Divider,
  message,
} from 'antd';
import {
  ReloadOutlined,
  SearchOutlined,
  RiseOutlined,
  FallOutlined,
  FireOutlined,
  TrophyOutlined,
  DollarOutlined,
  AppstoreOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import ReactECharts from 'echarts-for-react';
import { useSectorData } from '../hooks/useSectorData';
import type { SectorItem, SectorPerformanceItem, SectorStockItem } from '../api/sectors';

const { Title, Text } = Typography;
const { Option } = Select;

function SectorsPage() {
  const {
    sectors,
    performance,
    selectedSectorStocks,
    loading,
    lastUpdate,
    loadSectorStocks,
    search,
    refresh,
  } = useSectorData({
    autoRefresh: true,
    refreshInterval: 30000,
  });

  const [selectedSector, setSelectedSector] = useState<SectorPerformanceItem | null>(null);
  const [sortBy, setSortBy] = useState<'change' | 'amount' | 'count'>('change');
  const [searchKeyword, setSearchKeyword] = useState('');

  // 格式化金额
  const formatAmount = (amount: number) => {
    if (amount >= 100000000) {
      return `${(amount / 100000000).toFixed(2)}亿`;
    } else if (amount >= 10000) {
      return `${(amount / 10000).toFixed(2)}万`;
    }
    return amount.toFixed(2);
  };

  // 统计数据
  const stats = useMemo(() => {
    const riseCount = performance.filter((p) => p.avg_change_percent > 0).length;
    const fallCount = performance.filter((p) => p.avg_change_percent < 0).length;
    const flatCount = performance.filter((p) => p.avg_change_percent === 0).length;
    const totalAmount = performance.reduce((sum, p) => sum + p.total_amount, 0);

    return { riseCount, fallCount, flatCount, totalAmount };
  }, [performance]);

  // 板块表现排行数据（根据排序条件）
  const sortedPerformance = useMemo(() => {
    const data = [...performance];
    switch (sortBy) {
      case 'change':
        return data.sort((a, b) => b.avg_change_percent - a.avg_change_percent);
      case 'amount':
        return data.sort((a, b) => b.total_amount - a.total_amount);
      case 'count':
        return data.sort((a, b) => b.stock_count - a.stock_count);
      default:
        return data;
    }
  }, [performance, sortBy]);

  // 板块热度图表配置
  const heatChartOption = useMemo(() => {
    const top10 = sortedPerformance.slice(0, 10);
    return {
      title: {
        text: '板块热度TOP10',
        left: 'center',
        textStyle: { fontSize: 16 },
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'value',
        axisLabel: {
          formatter: '{value}%',
        },
      },
      yAxis: {
        type: 'category',
        data: top10.map((p) => p.sector_name),
        axisLabel: {
          interval: 0,
        },
      },
      series: [
        {
          name: '平均涨幅',
          type: 'bar',
          data: top10.map((p) => ({
            value: p.avg_change_percent.toFixed(2),
            itemStyle: {
              color: p.avg_change_percent > 0 ? '#cf1322' : '#3f8600',
            },
          })),
          label: {
            show: true,
            position: 'right',
            formatter: '{c}%',
          },
        },
      ],
    };
  }, [sortedPerformance]);

  // 板块列表表格列
  const sectorColumns: ColumnsType<SectorItem> = [
    {
      title: '排名',
      key: 'rank',
      width: 70,
      render: (_, __, index) => (
        <Tag color={index < 3 ? 'red' : 'default'} icon={index < 3 ? <TrophyOutlined /> : undefined}>
          {index + 1}
        </Tag>
      ),
    },
    {
      title: '代码',
      dataIndex: 'code',
      key: 'code',
      width: 100,
    },
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: 150,
      render: (text, record) => (
        <Space>
          <Text strong>{text}</Text>
          {record.leader_code && <Tag color="gold">龙头</Tag>}
        </Space>
      ),
    },
    {
      title: '股票数',
      dataIndex: 'stock_count',
      key: 'stock_count',
      width: 90,
      sorter: (a, b) => a.stock_count - b.stock_count,
    },
    {
      title: '平均涨幅',
      dataIndex: 'avg_change_percent',
      key: 'avg_change_percent',
      width: 120,
      render: (value) => (
        <Text style={{ color: value > 0 ? '#cf1322' : value < 0 ? '#3f8600' : '#666' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </Text>
      ),
      sorter: (a, b) => a.avg_change_percent - b.avg_change_percent,
      defaultSortOrder: 'descend',
    },
    {
      title: '总成交额',
      dataIndex: 'total_amount',
      key: 'total_amount',
      width: 120,
      render: (value) => formatAmount(value),
      sorter: (a, b) => a.total_amount - b.total_amount,
    },
    {
      title: '涨停/跌停',
      key: 'limits',
      width: 110,
      render: (_, record) => (
        <Space>
          <Tag color="red">{record.limit_up_count}</Tag>
          <Tag color="green">{record.limit_down_count}</Tag>
        </Space>
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: 100,
      render: (_, record) => (
        <Button
          type="link"
          size="small"
          onClick={() => handleSectorClick(record)}
        >
          查看详情
        </Button>
      ),
    },
  ];

  // 成分股表格列
  const stockColumns: ColumnsType<SectorStockItem> = [
    {
      title: '代码',
      dataIndex: 'code',
      key: 'code',
      width: 100,
    },
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: 120,
      render: (text, record) => (
        <Space>
          <Text strong>{text}</Text>
          {record.is_leader && <Tag color="gold">龙头</Tag>}
          {record.limit_up && <Tag color="red">涨停</Tag>}
          {record.limit_down && <Tag color="green">跌停</Tag>}
        </Space>
      ),
    },
    {
      title: '现价',
      dataIndex: 'price',
      key: 'price',
      width: 100,
      render: (value) => `¥${value.toFixed(2)}`,
    },
    {
      title: '涨幅',
      dataIndex: 'change_percent',
      key: 'change_percent',
      width: 100,
      render: (value) => (
        <Text style={{ color: value > 0 ? '#cf1322' : value < 0 ? '#3f8600' : '#666' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </Text>
      ),
      sorter: (a, b) => a.change_percent - b.change_percent,
    },
    {
      title: '成交量',
      dataIndex: 'volume',
      key: 'volume',
      width: 120,
      render: (value) => formatAmount(value),
    },
    {
      title: '成交额',
      dataIndex: 'amount',
      key: 'amount',
      width: 120,
      render: (value) => formatAmount(value),
    },
  ];

  // 板块详情点击
  const handleSectorClick = async (sector: SectorItem | SectorPerformanceItem) => {
    const code = 'code' in sector ? sector.code : sector.sector_code;
    const name = 'name' in sector ? sector.name : sector.sector_name;

    setSelectedSector(null);
    try {
      await loadSectorStocks(code);
      setSelectedSector({
        sector_code: code,
        sector_name: name,
        avg_change_percent: sector.avg_change_percent,
        median_change_percent: 0,
        total_volume: 0,
        total_amount: sector.total_amount || 0,
        stock_count: sector.stock_count || 0,
        limit_up_count: sector.limit_up_count || 0,
        limit_down_count: sector.limit_down_count || 0,
        rise_count: 0,
        fall_count: 0,
        flat_count: 0,
      });
    } catch (error) {
      message.error('加载成分股失败');
    }
  };

  // 搜索处理
  const handleSearch = () => {
    search(searchKeyword);
  };

  return (
    <div style={{ padding: 24 }}>
      {/* 页面标题和操作栏 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={18}>
          <Title level={2} style={{ margin: 0 }}>
            <FireOutlined /> 概念板块分析
          </Title>
        </Col>
        <Col span={6} style={{ textAlign: 'right' }}>
          <Button
            icon={<ReloadOutlined />}
            onClick={refresh}
            loading={loading}
          >
            刷新
          </Button>
        </Col>
      </Row>

      {/* 统计卡片 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="板块总数"
              value={performance.length}
              prefix={<AppstoreOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="上涨板块"
              value={stats.riseCount}
              prefix={<RiseOutlined />}
              valueStyle={{ color: '#cf1322' }}
              suffix={`/ ${performance.length}`}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="下跌板块"
              value={stats.fallCount}
              prefix={<FallOutlined />}
              valueStyle={{ color: '#3f8600' }}
              suffix={`/ ${performance.length}`}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="总成交额"
              value={stats.totalAmount / 100000000}
              prefix={<DollarOutlined />}
              precision={2}
              suffix="亿"
              valueStyle={{ color: '#faad14' }}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={16}>
        {/* 左侧：板块热度图表 */}
        <Col span={8}>
          <Card
            title="板块热度排行"
            extra={
              <Select
                value={sortBy}
                onChange={setSortBy}
                style={{ width: 120 }}
                size="small"
              >
                <Option value="change">按涨幅</Option>
                <Option value="amount">按成交额</Option>
                <Option value="count">按股票数</Option>
              </Select>
            }
          >
            <ReactECharts
              option={heatChartOption}
              style={{ height: 400 }}
              opts={{ renderer: 'canvas' }}
            />
          </Card>

          {/* 板块TOP5列表 */}
          <Card
            title="热门板块 TOP5"
            style={{ marginTop: 16 }}
            size="small"
          >
            {sortedPerformance.slice(0, 5).map((sector, index) => (
              <div
                key={sector.sector_code}
                style={{
                  padding: '8px 0',
                  cursor: 'pointer',
                  borderBottom: index < 4 ? '1px solid #f0f0f0' : 'none',
                }}
                onClick={() => handleSectorClick(sector)}
              >
                <Row justify="space-between" align="middle">
                  <Col>
                    <Space>
                      <Tag color={index < 3 ? 'red' : 'default'}>{index + 1}</Tag>
                      <Text>{sector.sector_name}</Text>
                    </Space>
                  </Col>
                  <Col>
                    <Text
                      style={{
                        color: sector.avg_change_percent > 0 ? '#cf1322' : '#3f8600',
                        fontWeight: 'bold',
                      }}
                    >
                      {sector.avg_change_percent >= 0 ? '+' : ''}{sector.avg_change_percent.toFixed(2)}%
                    </Text>
                  </Col>
                </Row>
              </div>
            ))}
          </Card>
        </Col>

        {/* 中间：板块列表 */}
        <Col span={10}>
          <Card
            title="板块列表"
            extra={
              <Space>
                <Input
                  placeholder="搜索板块"
                  value={searchKeyword}
                  onChange={(e) => setSearchKeyword(e.target.value)}
                  onPressEnter={handleSearch}
                  suffix={
                    <Button
                      icon={<SearchOutlined />}
                      onClick={handleSearch}
                      type="text"
                      size="small"
                    />
                  }
                  style={{ width: 150 }}
                  allowClear
                />
              </Space>
            }
          >
            <Table
              columns={sectorColumns}
              dataSource={sectors}
              loading={loading}
              rowKey="code"
              size="small"
              pagination={{ pageSize: 15, size: 'small' }}
              scroll={{ y: 500 }}
              rowClassName={(record) => {
                if (record.avg_change_percent > 5) return 'row-high-gain';
                if (record.avg_change_percent < -5) return 'row-high-loss';
                return '';
              }}
            />
          </Card>
        </Col>

        {/* 右侧：板块详情 */}
        <Col span={6}>
          <Card
            title={selectedSector ? `${selectedSector.sector_name} 成分股` : '成分股详情'}
            extra={
              lastUpdate && (
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {lastUpdate.toLocaleTimeString()}
                </Text>
              )
            }
          >
            {selectedSector ? (
              <>
                {/* 板块统计 */}
                <Row gutter={[8, 8]} style={{ marginBottom: 16 }}>
                  <Col span={12}>
                    <Statistic
                      title="平均涨幅"
                      value={selectedSector.avg_change_percent}
                      precision={2}
                      suffix="%"
                      valueStyle={{
                        color: selectedSector.avg_change_percent > 0 ? '#cf1322' : '#3f8600',
                        fontSize: 16,
                      }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="股票数"
                      value={selectedSector.stock_count}
                      valueStyle={{ fontSize: 16 }}
                    />
                  </Col>
                </Row>

                <Divider style={{ margin: '12px 0' }} />

                {/* 成分股列表 */}
                <Table
                  columns={stockColumns}
                  dataSource={selectedSectorStocks}
                  loading={loading}
                  rowKey="code"
                  size="small"
                  pagination={false}
                  scroll={{ y: 400 }}
                />
              </>
            ) : (
              <div
                style={{
                  textAlign: 'center',
                  padding: '60px 0',
                  color: '#999',
                }}
              >
                <FireOutlined style={{ fontSize: 48, marginBottom: 16 }} />
                <div>请选择一个板块查看成分股</div>
              </div>
            )}
          </Card>
        </Col>
      </Row>

      {/* CSS样式 */}
      <style>{`
        .row-high-gain {
          background-color: #fff1f0;
        }
        .row-high-loss {
          background-color: #f6ffed;
        }
        .row-high-gain:hover,
        .row-high-loss:hover {
          background-color: #e6f7ff !important;
        }
      `}</style>
    </div>
  );
}

export default SectorsPage;
