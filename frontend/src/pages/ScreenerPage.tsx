/**
 * 个股挖掘页面
 * 包含龙头高度、连板统计、涨跌停分析等功能
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
  Tabs,
  Progress,
} from 'antd';
import {
  ReloadOutlined,
  SearchOutlined,
  RiseOutlined,
  FallOutlined,
  TrophyOutlined,
  FireOutlined,
  ThunderboltOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useScreenerData } from '../hooks/useScreenerData';
import type { LeaderItem, ConsecutiveBoardItem, LimitItem } from '../api/screener';

const { Title, Text } = Typography;
const { Option } = Select;

function ScreenerPage() {
  const {
    leaders,
    leaderSector,
    setLeaderSector,
    consecutiveData,
    minDays,
    setMinDays,
    boardType,
    setBoardType,
    limitUpData,
    limitDownData,
    loading,
    lastUpdate,
    loadLeaders,
    loadConsecutive,
    refresh,
  } = useScreenerData({
    autoRefresh: true,
    refreshInterval: 30000,
  });

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

  // 涨跌停统计
  const limitStats = useMemo(() => {
    return {
      upCount: limitUpData.length,
      downCount: limitDownData.length,
      total: limitUpData.length + limitDownData.length,
      upRatio: limitUpData.length / (limitUpData.length + limitDownData.length) * 100 || 0,
      downRatio: limitDownData.length / (limitUpData.length + limitDownData.length) * 100 || 0,
    };
  }, [limitUpData, limitDownData]);

  // 连板统计
  const consecutiveStats = useMemo(() => {
    const maxDays = consecutiveData.length > 0
      ? Math.max(...consecutiveData.map(d => d.consecutive_days))
      : 0;
    const avgDays = consecutiveData.length > 0
      ? consecutiveData.reduce((sum, d) => sum + d.consecutive_days, 0) / consecutiveData.length
      : 0;

    return { maxDays, avgDays };
  }, [consecutiveData]);

  // 龙头高度表格列
  const leaderColumns: ColumnsType<LeaderItem> = [
    {
      title: '排名',
      key: 'rank',
      width: 70,
      render: (_, __, index) => (
        <Tag
          color={index < 3 ? 'red' : 'default'}
          icon={index < 3 ? <TrophyOutlined /> : undefined}
        >
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
      width: 120,
      render: (text) => <Text strong>{text}</Text>,
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
      width: 150,
    },
    {
      title: '龙头高度',
      dataIndex: 'leader_height',
      key: 'leader_height',
      width: 140,
      render: (value, record) => (
        <Space direction="vertical" size={0}>
          <Progress
            percent={value}
            size="small"
            strokeColor={value > 90 ? '#cf1322' : value > 70 ? '#faad14' : '#1890ff'}
            format={(percent) => `${percent?.toFixed(1)}%`}
          />
          <Text type="secondary" style={{ fontSize: 12 }}>
            排名: {record.sector_rank}/{record.total_stocks}
          </Text>
        </Space>
      ),
      sorter: (a, b) => a.leader_height - b.leader_height,
      defaultSortOrder: 'descend' as any,
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
      title: '成交额',
      dataIndex: 'amount',
      key: 'amount',
      width: 120,
      render: (value) => formatAmount(value),
      sorter: (a, b) => a.amount - b.amount,
    },
  ];

  // 连板统计表格列
  const consecutiveColumns: ColumnsType<ConsecutiveBoardItem> = [
    {
      title: '排名',
      key: 'rank',
      width: 70,
      render: (_, __, index) => (
        <Tag
          color={index < 3 ? 'red' : 'default'}
          icon={index < 3 ? <FireOutlined /> : undefined}
        >
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
      width: 120,
      render: (text) => <Text strong>{text}</Text>,
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
      width: 150,
    },
    {
      title: '连板天数',
      dataIndex: 'consecutive_days',
      key: 'consecutive_days',
      width: 130,
      render: (value) => (
        <Tag
          icon={<ThunderboltOutlined />}
          color={value >= 5 ? 'red' : value >= 3 ? 'orange' : 'default'}
          style={{ fontSize: 15, fontWeight: 'bold' }}
        >
          {value} 天
        </Tag>
      ),
      sorter: (a, b) => b.consecutive_days - a.consecutive_days,
      defaultSortOrder: 'descend' as any,
    },
    {
      title: '类型',
      dataIndex: 'board_type',
      key: 'board_type',
      width: 90,
      render: (value) => (
        <Tag color={value === '连涨' ? 'red' : 'green'}>
          {value}
        </Tag>
      ),
    },
    {
      title: '起始日期',
      dataIndex: 'start_date',
      key: 'start_date',
      width: 110,
    },
    {
      title: '现价',
      dataIndex: 'current_price',
      key: 'current_price',
      width: 100,
      render: (value) => `¥${value.toFixed(2)}`,
    },
    {
      title: '连板原因',
      dataIndex: 'reason',
      key: 'reason',
      width: 200,
      ellipsis: true,
    },
  ];

  // 涨跌停表格列
  const limitColumns: ColumnsType<LimitItem> = [
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
          {record.is_first && <Tag color="gold">首板</Tag>}
        </Space>
      ),
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
      width: 150,
    },
    {
      title: '类型',
      dataIndex: 'limit_type',
      key: 'limit_type',
      width: 90,
      render: (value) => (
        <Tag
          icon={value === '涨停' ? <RiseOutlined /> : <FallOutlined />}
          color={value === '涨停' ? 'red' : 'green'}
        >
          {value}
        </Tag>
      ),
    },
    {
      title: '时间',
      dataIndex: 'limit_time',
      key: 'limit_time',
      width: 90,
      render: (value) => (
        <Space>
          <ClockCircleOutlined />
          <Text>{value}</Text>
        </Space>
      ),
    },
    {
      title: '涨停价',
      dataIndex: 'limit_price',
      key: 'limit_price',
      width: 100,
      render: (value) => `¥${value.toFixed(2)}`,
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
    {
      title: '涨停原因',
      dataIndex: 'reason',
      key: 'reason',
      width: 200,
      ellipsis: true,
    },
  ];

  // 过滤数据
  const filteredLeaders = useMemo(() => {
    if (!searchKeyword) return leaders;
    return leaders.filter(
      (item) =>
        item.code.includes(searchKeyword) ||
        item.name.includes(searchKeyword) ||
        item.sector.includes(searchKeyword)
    );
  }, [leaders, searchKeyword]);

  return (
    <div style={{ padding: 24 }}>
      {/* 标题和操作栏 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={18}>
          <Title level={2} style={{ margin: 0 }}>
            <TrophyOutlined /> 个股挖掘
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

      {/* 更新时间 */}
      {lastUpdate && (
        <Text type="secondary" style={{ display: 'block', marginBottom: 16 }}>
          数据更新时间: {lastUpdate.toLocaleString()}
        </Text>
      )}

      <Card>
        <Tabs
          defaultActiveKey="leaders"
          items={[
            {
              key: 'leaders',
              label: (
                <Space>
                  <TrophyOutlined />
                  龙头高度
                </Space>
              ),
              children: (
                <>
                  {/* 筛选栏 */}
                  <Row gutter={16} style={{ marginBottom: 16 }}>
                    <Col span={8}>
                      <Input
                        placeholder="输入板块代码筛选（可选）"
                        value={leaderSector}
                        onChange={(e) => setLeaderSector(e.target.value)}
                        onPressEnter={() => loadLeaders(leaderSector || undefined)}
                        suffix={
                          <Button
                            icon={<SearchOutlined />}
                            onClick={() => loadLeaders(leaderSector || undefined)}
                            type="primary"
                            size="small"
                          >
                            查询
                          </Button>
                        }
                        allowClear
                      />
                    </Col>
                    <Col span={8}>
                      <Input
                        placeholder="搜索股票代码或名称"
                        value={searchKeyword}
                        onChange={(e) => setSearchKeyword(e.target.value)}
                        allowClear
                      />
                    </Col>
                  </Row>

                  {/* 统计信息 */}
                  <Row gutter={16} style={{ marginBottom: 16 }}>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="龙头数量"
                          value={filteredLeaders.length}
                          prefix={<TrophyOutlined />}
                          valueStyle={{ color: '#1890ff' }}
                        />
                      </Card>
                    </Col>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="高度>90%"
                          value={filteredLeaders.filter(l => l.leader_height > 90).length}
                          valueStyle={{ color: '#cf1322' }}
                        />
                      </Card>
                    </Col>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="高度>80%"
                          value={filteredLeaders.filter(l => l.leader_height > 80).length}
                          valueStyle={{ color: '#faad14' }}
                        />
                      </Card>
                    </Col>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="高度>70%"
                          value={filteredLeaders.filter(l => l.leader_height > 70).length}
                          valueStyle={{ color: '#1890ff' }}
                        />
                      </Card>
                    </Col>
                  </Row>

                  {/* 龙头高度表格 */}
                  <Table
                    columns={leaderColumns}
                    dataSource={filteredLeaders}
                    loading={loading}
                    rowKey="code"
                    size="small"
                    pagination={{ pageSize: 20, size: 'small' }}
                    scroll={{ x: 1000 }}
                    rowClassName={(record) => {
                      if (record.leader_height > 90) return 'row-leader-high';
                      if (record.leader_height > 80) return 'row-leader-medium';
                      return '';
                    }}
                  />
                </>
              ),
            },
            {
              key: 'consecutive',
              label: (
                <Space>
                  <FireOutlined />
                  连板统计
                </Space>
              ),
              children: (
                <>
                  {/* 筛选栏 */}
                  <Row gutter={16} style={{ marginBottom: 16 }}>
                    <Col span={6}>
                      <Input
                        type="number"
                        placeholder="最小连板天数"
                        value={minDays}
                        onChange={(e) => setMinDays(Number(e.target.value))}
                        addonAfter="天"
                        min={2}
                      />
                    </Col>
                    <Col span={6}>
                      <Select
                        value={boardType}
                        onChange={setBoardType}
                        style={{ width: '100%' }}
                      >
                        <Option value="连涨">连涨</Option>
                        <Option value="连跌">连跌</Option>
                      </Select>
                    </Col>
                    <Col span={6}>
                      <Button
                        type="primary"
                        icon={<SearchOutlined />}
                        onClick={() => loadConsecutive(minDays, boardType)}
                        block
                      >
                        查询
                      </Button>
                    </Col>
                  </Row>

                  {/* 统计信息 */}
                  <Row gutter={16} style={{ marginBottom: 16 }}>
                    <Col span={8}>
                      <Card size="small">
                        <Statistic
                          title="连板股票数"
                          value={consecutiveData.length}
                          prefix={<FireOutlined />}
                          valueStyle={{ color: '#1890ff' }}
                        />
                      </Card>
                    </Col>
                    <Col span={8}>
                      <Card size="small">
                        <Statistic
                          title="最高连板"
                          value={consecutiveStats.maxDays}
                          suffix="天"
                          valueStyle={{ color: '#cf1322' }}
                        />
                      </Card>
                    </Col>
                    <Col span={8}>
                      <Card size="small">
                        <Statistic
                          title="平均连板"
                          value={consecutiveStats.avgDays}
                          suffix="天"
                          precision={1}
                          valueStyle={{ color: '#faad14' }}
                        />
                      </Card>
                    </Col>
                  </Row>

                  {/* 连板统计表格 */}
                  <Table
                    columns={consecutiveColumns}
                    dataSource={consecutiveData}
                    loading={loading}
                    rowKey="code"
                    size="small"
                    pagination={{ pageSize: 20, size: 'small' }}
                    scroll={{ x: 1200 }}
                    rowClassName={(record) => {
                      if (record.consecutive_days >= 5) return 'row-consecutive-high';
                      if (record.consecutive_days >= 3) return 'row-consecutive-medium';
                      return '';
                    }}
                  />
                </>
              ),
            },
            {
              key: 'limit',
              label: (
                <Space>
                  <RiseOutlined />
                  涨跌停
                </Space>
              ),
              children: (
                <>
                  {/* 统计信息 */}
                  <Row gutter={16} style={{ marginBottom: 16 }}>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="涨停"
                          value={limitStats.upCount}
                          prefix={<RiseOutlined />}
                          valueStyle={{ color: '#cf1322' }}
                          suffix={`/ ${limitStats.total}`}
                        />
                      </Card>
                    </Col>
                    <Col span={6}>
                      <Card size="small">
                        <Statistic
                          title="跌停"
                          value={limitStats.downCount}
                          prefix={<FallOutlined />}
                          valueStyle={{ color: '#3f8600' }}
                          suffix={`/ ${limitStats.total}`}
                        />
                      </Card>
                    </Col>
                    <Col span={12}>
                      <Card size="small">
                        <Space direction="vertical" style={{ width: '100%' }}>
                          <Text type="secondary">涨跌分布</Text>
                          <Progress
                            percent={limitStats.upRatio}
                            strokeColor="#cf1322"
                            format={(percent) => `涨停 ${percent?.toFixed(1)}%`}
                          />
                          <Progress
                            percent={limitStats.downRatio}
                            strokeColor="#3f8600"
                            format={(percent) => `跌停 ${percent?.toFixed(1)}%`}
                          />
                        </Space>
                      </Card>
                    </Col>
                  </Row>

                  {/* 涨跌停表格 */}
                  <Table
                    columns={limitColumns}
                    dataSource={[...limitUpData, ...limitDownData]}
                    loading={loading}
                    rowKey="code"
                    size="small"
                    pagination={{ pageSize: 20, size: 'small' }}
                    scroll={{ x: 1200 }}
                    rowClassName={(record) => {
                      if (record.limit_type === '涨停') return 'row-limit-up';
                      return 'row-limit-down';
                    }}
                  />
                </>
              ),
            },
          ]}
        />
      </Card>

      {/* CSS样式 */}
      <style>{`
        .row-leader-high {
          background-color: #fff1f0 !important;
        }
        .row-leader-medium {
          background-color: #fff7e6 !important;
        }
        .row-consecutive-high {
          background-color: #fff1f0 !important;
        }
        .row-consecutive-medium {
          background-color: #fff7e6 !important;
        }
        .row-limit-up {
          background-color: #fff1f0 !important;
        }
        .row-limit-down {
          background-color: #f6ffed !important;
        }
        .row-leader-high:hover,
        .row-leader-medium:hover,
        .row-consecutive-high:hover,
        .row-consecutive-medium:hover,
        .row-limit-up:hover,
        .row-limit-down:hover {
          background-color: #e6f7ff !important;
        }
      `}</style>
    </div>
  );
}

export default ScreenerPage;
