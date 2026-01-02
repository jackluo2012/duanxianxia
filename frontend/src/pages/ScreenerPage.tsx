// 个股挖掘页面
// 龙头高度、连板统计、涨跌停分析

import React, { useState, useEffect } from 'react';
import {
  Card,
  Tabs,
  Table,
  Button,
  Input,
  Select,
  Space,
  Tag,
  Statistic,
  Row,
  Col,
  message,
} from 'antd';
import {
  ReloadOutlined,
  RiseOutlined,
  FallOutlined,
  TrophyOutlined,
  FireOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import {
  fetchLeaders,
  fetchConsecutiveBoards,
  fetchLimitUp,
  fetchLimitDown,
  LeaderItem,
  ConsecutiveBoardItem,
  LimitItem,
} from '../api/screener';

const { TabPane } = Tabs;
const { Option } = Select;

const ScreenerPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('leaders');

  // 龙头高度数据
  const [leaders, setLeaders] = useState<LeaderItem[]>([]);
  const [leaderSector, setLeaderSector] = useState<string>('');

  // 连板统计数据
  const [consecutiveData, setConsecutiveData] = useState<ConsecutiveBoardItem[]>([]);
  const [minDays, setMinDays] = useState<number>(2);
  const [boardType, setBoardType] = useState<string>('连涨');

  // 涨跌停数据
  const [limitUpData, setLimitUpData] = useState<LimitItem[]>([]);
  const [limitDownData, setLimitDownData] = useState<LimitItem[]>([]);

  // 加载龙头高度数据
  const loadLeaders = async () => {
    setLoading(true);
    try {
      const data = await fetchLeaders(leaderSector || undefined);
      setLeaders(data);
    } catch (error) {
      message.error('获取龙头高度数据失败');
    } finally {
      setLoading(false);
    }
  };

  // 加载连板统计数据
  const loadConsecutive = async () => {
    setLoading(true);
    try {
      const data = await fetchConsecutiveBoards(minDays, boardType);
      setConsecutiveData(data);
    } catch (error) {
      message.error('获取连板统计数据失败');
    } finally {
      setLoading(false);
    }
  };

  // 加载涨跌停数据
  const loadLimits = async () => {
    setLoading(true);
    try {
      const [upData, downData] = await Promise.all([
        fetchLimitUp('today'),
        fetchLimitDown('today'),
      ]);
      setLimitUpData(upData);
      setLimitDownData(downData);
    } catch (error) {
      message.error('获取涨跌停数据失败');
    } finally {
      setLoading(false);
    }
  };

  // 初始加载
  useEffect(() => {
    loadLeaders();
    loadConsecutive();
    loadLimits();
  }, []);

  // Tab 切换时刷新对应数据
  const handleTabChange = (key: string) => {
    setActiveTab(key);
    if (key === 'leaders') loadLeaders();
    else if (key === 'consecutive') loadConsecutive();
    else if (key === 'limit') loadLimits();
  };

  // 龙头高度表格列
  const leaderColumns: ColumnsType<LeaderItem> = [
    {
      title: '排名',
      key: 'rank',
      width: 80,
      render: (_, __, index) => (
        <Tag color={index < 3 ? 'gold' : 'default'}>
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
      width: 120,
      render: (value) => (
        <Statistic
          value={value}
          precision={2}
          suffix="%"
          valueStyle={{ fontSize: 16, color: value > 90 ? '#cf1322' : '#3f8600' }}
        />
      ),
      sorter: (a, b) => a.leader_height - b.leader_height,
    },
    {
      title: '板块排名',
      dataIndex: 'sector_rank',
      key: 'sector_rank',
      width: 100,
      render: (value, record) => `${value}/${record.total_stocks}`,
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
        <span style={{ color: value >= 0 ? '#cf1322' : '#3f8600' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </span>
      ),
    },
  ];

  // 连板统计表格列
  const consecutiveColumns: ColumnsType<ConsecutiveBoardItem> = [
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
      width: 120,
      render: (value) => (
        <Tag icon={<FireOutlined />} color="red" style={{ fontSize: 16 }}>
          {value} 天
        </Tag>
      ),
      sorter: (a, b) => a.consecutive_days - b.consecutive_days,
    },
    {
      title: '连板类型',
      dataIndex: 'board_type',
      key: 'board_type',
      width: 100,
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
      width: 120,
    },
    {
      title: '结束日期',
      dataIndex: 'end_date',
      key: 'end_date',
      width: 120,
    },
    {
      title: '现价',
      dataIndex: 'current_price',
      key: 'current_price',
      width: 100,
      render: (value) => `¥${value.toFixed(2)}`,
    },
    {
      title: '原因',
      dataIndex: 'reason',
      key: 'reason',
      width: 200,
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
    },
    {
      title: '类型',
      dataIndex: 'limit_type',
      key: 'limit_type',
      width: 100,
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
      width: 100,
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
      render: (value) => `${(value / 10000).toFixed(2)}万`,
    },
    {
      title: '成交额',
      dataIndex: 'amount',
      key: 'amount',
      width: 120,
      render: (value) => `${(value / 100000000).toFixed(2)}亿`,
    },
    {
      title: '原因',
      dataIndex: 'reason',
      key: 'reason',
      width: 200,
    },
    {
      title: '首次',
      dataIndex: 'is_first',
      key: 'is_first',
      width: 80,
      render: (value) => (value ? <Tag color="gold">首板</Tag> : null),
    },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Card
        title={
          <Space>
            <TrophyOutlined />
            <span>个股挖掘</span>
          </Space>
        }
        extra={
          <Button
            icon={<ReloadOutlined />}
            onClick={() => {
              if (activeTab === 'leaders') loadLeaders();
              else if (activeTab === 'consecutive') loadConsecutive();
              else if (activeTab === 'limit') loadLimits();
            }}
            loading={loading}
          >
            刷新
          </Button>
        }
      >
        <Tabs activeKey={activeTab} onChange={handleTabChange}>
          {/* 龙头高度 */}
          <TabPane tab="龙头高度" key="leaders">
            <Space style={{ marginBottom: 16 }}>
              <Input
                placeholder="板块代码（可选）"
                value={leaderSector}
                onChange={(e) => setLeaderSector(e.target.value)}
                style={{ width: 200 }}
                onPressEnter={loadLeaders}
              />
              <Button type="primary" onClick={loadLeaders}>
                查询
              </Button>
            </Space>
            <Table
              columns={leaderColumns}
              dataSource={leaders}
              loading={loading}
              rowKey="code"
              pagination={{ pageSize: 20 }}
              scroll={{ x: 1000 }}
            />
          </TabPane>

          {/* 连板统计 */}
          <TabPane tab="连板统计" key="consecutive">
            <Space style={{ marginBottom: 16 }}>
              <Input
                type="number"
                placeholder="最小连板天数"
                value={minDays}
                onChange={(e) => setMinDays(Number(e.target.value))}
                style={{ width: 150 }}
              />
              <Select
                value={boardType}
                onChange={setBoardType}
                style={{ width: 120 }}
              >
                <Option value="连涨">连涨</Option>
                <Option value="连跌">连跌</Option>
              </Select>
              <Button type="primary" onClick={loadConsecutive}>
                查询
              </Button>
            </Space>
            <Table
              columns={consecutiveColumns}
              dataSource={consecutiveData}
              loading={loading}
              rowKey="code"
              pagination={{ pageSize: 20 }}
              scroll={{ x: 1200 }}
            />
          </TabPane>

          {/* 涨跌停 */}
          <TabPane tab="涨跌停" key="limit">
            <Row gutter={16} style={{ marginBottom: 16 }}>
              <Col span={12}>
                <Card>
                  <Statistic
                    title="涨停"
                    value={limitUpData.length}
                    prefix={<RiseOutlined />}
                    valueStyle={{ color: '#cf1322' }}
                  />
                </Card>
              </Col>
              <Col span={12}>
                <Card>
                  <Statistic
                    title="跌停"
                    value={limitDownData.length}
                    prefix={<FallOutlined />}
                    valueStyle={{ color: '#3f8600' }}
                  />
                </Card>
              </Col>
            </Row>
            <Table
              columns={limitColumns}
              dataSource={[...limitUpData, ...limitDownData]}
              loading={loading}
              rowKey="code"
              pagination={{ pageSize: 20 }}
              scroll={{ x: 1200 }}
            />
          </TabPane>
        </Tabs>
      </Card>
    </div>
  );
};

export default ScreenerPage;
