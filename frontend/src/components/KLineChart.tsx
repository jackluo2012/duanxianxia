import ReactECharts from 'echarts-for-react';
import { HistoryPoint } from '../api/quotes';

interface KLineChartProps {
  data: HistoryPoint[];
  period: string;
  loading?: boolean;
}

export default function KLineChart({ data, period, loading = false }: KLineChartProps) {
  const isKLine = period === '5m' || period === '1d';

  const option = {
    animation: false,
    title: {
      text: period === '1m' ? '实时分时图' : period === '5m' ? '5分钟K线' : '日K线',
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
    series: isKLine
      ? [
          {
            name: 'K线',
            type: 'candlestick',
            data: data.map((d) => [d.open, d.close, d.low, d.high]),
            itemStyle: {
              color: '#ef5350',
              color0: '#26a69a',
              borderColor: '#ef5350',
              borderColor0: '#26a69a',
            },
          },
        ]
      : [
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
