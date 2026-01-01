import ReactECharts from 'echarts-for-react';
import { useMemo } from 'react';
import { HistoryPoint } from '../api/quotes';

interface KLineChartProps {
  data: HistoryPoint[];
  period: string;
  loading?: boolean;
}

// 数据采样函数
function sampleData(data: HistoryPoint[], maxPoints: number): HistoryPoint[] {
  if (data.length <= maxPoints) {
    return data;
  }

  // 计算采样步长
  const step = Math.ceil(data.length / maxPoints);

  // 均匀采样，保留首尾点
  const sampled: HistoryPoint[] = [data[0]]; // 保留第一个点

  for (let i = 1; i < data.length - 1; i += step) {
    sampled.push(data[i]);
  }

  sampled.push(data[data.length - 1]); // 保留最后一个点

  return sampled;
}

export default function KLineChart({ data, period, loading = false }: KLineChartProps) {
  const isKLine = period === '5m' || period === '1d';

  // 根据周期类型设置采样阈值
  const samplingThreshold = isKLine ? 1000 : 500;

  // 应用数据采样
  const sampledData = useMemo(() => {
    const result = sampleData(data, samplingThreshold);
    console.log(
      `[KLineChart] 数据采样: ${data.length} → ${result.length} 点 (阈值: ${samplingThreshold})`
    );
    return result;
  }, [data, samplingThreshold]);

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
      data: sampledData.map((d) => d.time),
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
            data: sampledData.map((d) => [d.open, d.close, d.low, d.high]),
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
            data: sampledData.map((d) => d.price),
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

  if (loading || sampledData.length === 0) {
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
