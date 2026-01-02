// 概念板块页面（简化版）

import React, { useState, useEffect } from 'react';
import { Card, Table, Button, message } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import {
  fetchSectors,
  fetchSectorPerformance,
  SectorItem,
  SectorPerformanceItem,
} from '../api/sectors';

const SectorsPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [sectors, setSectors] = useState<SectorItem[]>([]);
  const [performance, setPerformance] = useState<SectorPerformanceItem[]>([]);

  const loadSectors = async () => {
    setLoading(true);
    try {
      const [data1, data2] = await Promise.all([
        fetchSectors(100),
        fetchSectorPerformance(50),
      ]);
      setSectors(data1);
      setPerformance(data2);
    } catch (error) {
      message.error('获取板块数据失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSectors();
  }, []);

  const sectorColumns: ColumnsType<SectorItem> = [
    { title: '代码', dataIndex: 'code', key: 'code', width: 120 },
    { title: '名称', dataIndex: 'name', key: 'name', width: 200 },
    {
      title: '股票数',
      dataIndex: 'stock_count',
      key: 'stock_count',
      width: 100,
    },
    {
      title: '平均涨幅',
      dataIndex: 'avg_change_percent',
      key: 'avg_change_percent',
      width: 120,
      render: (value) => (
        <span style={{ color: value >= 0 ? '#cf1322' : '#3f8600' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </span>
      ),
      sorter: (a, b) => a.avg_change_percent - b.avg_change_percent,
    },
    {
      title: '总成交额',
      dataIndex: 'total_amount',
      key: 'total_amount',
      width: 120,
      render: (value) => `${(value / 100000000).toFixed(2)}亿`,
    },
    {
      title: '涨停数',
      dataIndex: 'limit_up_count',
      key: 'limit_up_count',
      width: 100,
    },
    {
      title: '跌停数',
      dataIndex: 'limit_down_count',
      key: 'limit_down_count',
      width: 100,
    },
  ];

  const perfColumns: ColumnsType<SectorPerformanceItem> = [
    { title: '代码', dataIndex: 'sector_code', key: 'sector_code', width: 120 },
    { title: '名称', dataIndex: 'sector_name', key: 'sector_name', width: 200 },
    {
      title: '平均涨幅',
      dataIndex: 'avg_change_percent',
      key: 'avg_change_percent',
      width: 120,
      render: (value) => (
        <span style={{ color: value >= 0 ? '#cf1322' : '#3f8600' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </span>
      ),
      sorter: (a, b) => a.avg_change_percent - b.avg_change_percent,
    },
    {
      title: '中位数涨幅',
      dataIndex: 'median_change_percent',
      key: 'median_change_percent',
      width: 120,
      render: (value) => (
        <span style={{ color: value >= 0 ? '#cf1322' : '#3f8600' }}>
          {value >= 0 ? '+' : ''}{value.toFixed(2)}%
        </span>
      ),
    },
    { title: '上涨', dataIndex: 'rise_count', key: 'rise_count', width: 80 },
    { title: '下跌', dataIndex: 'fall_count', key: 'fall_count', width: 80 },
    { title: '平盘', dataIndex: 'flat_count', key: 'flat_count', width: 80 },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Card
        title="概念板块"
        extra={
          <Button icon={<ReloadOutlined />} onClick={loadSectors} loading={loading}>
            刷新
          </Button>
        }
      >
        <Table
          title={() => '板块列表'}
          columns={sectorColumns}
          dataSource={sectors}
          loading={loading}
          rowKey="code"
          pagination={{ pageSize: 20 }}
          style={{ marginBottom: 24 }}
        />
        <Table
          title={() => '板块表现排行'}
          columns={perfColumns}
          dataSource={performance}
          loading={loading}
          rowKey="sector_code"
          pagination={{ pageSize: 20 }}
        />
      </Card>
    </div>
  );
};

export default SectorsPage;
