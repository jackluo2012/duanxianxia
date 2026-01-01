import ReactECharts from 'echarts-for-react';
import { HistoryPoint } from '../api/quotes';

interface RealtimeChartProps {
  data: HistoryPoint[];
  loading?: boolean;
}

export default function RealtimeChart({ data, loading = false }: RealtimeChartProps) {
  const option = {
    animation: false,
    title: {
      text: '实时分时图',
      left: 'center',
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
      },
    },
    xAxis: {
      type: 'category',
      data: data.map((d) => d.time),
      axisLine: {
        lineStyle: {
          color: '#888',
        },
      },
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLine: {
        lineStyle: {
          color: '#888',
        },
      },
      splitLine: {
        lineStyle: {
          color: '#ddd',
          type: 'dashed',
        },
      },
    },
    series: [
      {
        name: '价格',
        type: 'line',
        data: data.map((d) => d.price),
        smooth: true,
        symbol: 'none',
        lineStyle: {
          color: '#1890ff',
          width: 2,
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(24, 144, 255, 0.3)' },
              { offset: 1, color: 'rgba(24, 144, 255, 0.05)' },
            ],
          },
        },
      },
    ],
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
  };

  if (loading || data.length === 0) {
    return (
      <div
        style={{
          height: 400,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#999',
        }}
      >
        {loading ? '加载中...' : '暂无数据'}
      </div>
    );
  }

  return <ReactECharts option={option} style={{ height: 400 }} />;
}
