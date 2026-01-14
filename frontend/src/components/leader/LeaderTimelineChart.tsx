import { Card, Spin, Empty } from 'antd';
import ReactECharts from 'echarts-for-react';
import { useLeaderDetail } from '../../hooks/useLeader';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderTimelineChartProps {
  stock: LeaderBoardItem | null;
}

function LeaderTimelineChart({ stock }: LeaderTimelineChartProps) {
  const { data: detail, isLoading } = useLeaderDetail(stock?.code || '');

  if (!stock) {
    return (
      <Card title="📊 历史涨停时间线">
        <Empty description="请选择股票查看历史时间线" />
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card title="📊 历史涨停时间线">
        <Spin />
      </Card>
    );
  }

  if (!detail || !detail.limit_up_history || detail.limit_up_history.length === 0) {
    return (
      <Card title="📊 历史涨停时间线">
        <Empty description="暂无历史数据" />
      </Card>
    );
  }

  const dates = detail.limit_up_history.map((record) => record.date);
  const changes = detail.limit_up_history.map((record) => record.change_percent);
  const sealedAmounts = detail.limit_up_history.map((record) =>
    (record.sealed_amount / 100000000).toFixed(2)
  );

  const option = {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
      },
    },
    legend: {
      data: ['涨幅(%)', '封单金额(亿)'],
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: {
        rotate: 45,
      },
    },
    yAxis: [
      {
        type: 'value',
        name: '涨幅(%)',
        position: 'left',
      },
      {
        type: 'value',
        name: '封单金额(亿)',
        position: 'right',
      },
    ],
    series: [
      {
        name: '涨幅(%)',
        type: 'line',
        data: changes,
        smooth: true,
        itemStyle: {
          color: '#f5222d',
        },
      },
      {
        name: '封单金额(亿)',
        type: 'bar',
        yAxisIndex: 1,
        data: sealedAmounts,
        itemStyle: {
          color: '#1890ff',
        },
      },
    ],
  };

  return (
    <Card title="📊 历史涨停时间线" style={{ marginTop: 16 }}>
      <ReactECharts option={option} style={{ height: '300px' }} />
    </Card>
  );
}

export default LeaderTimelineChart;
