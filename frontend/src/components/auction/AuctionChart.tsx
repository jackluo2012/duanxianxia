import { Empty, Spin } from 'antd';
import { useEffect, useState } from 'react';
import ReactECharts from 'echarts-for-react';

interface AuctionChartProps {
  code: string;
}

interface AuctionDataPoint {
  time: string;
  price: number;
  buy1_volume: number;
  sell1_volume: number;
}

function AuctionChart({ code }: AuctionChartProps) {
  const [data, setData] = useState<AuctionDataPoint[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchAuctionData = async () => {
      setLoading(true);
      try {
        // TODO: 实际调用后端 API
        // const response = await fetch(`http://localhost:8084/api/auction/details/${code}`);
        // const result = await response.json();
        // setData(result.timeline);

        // 模拟数据 (9:15-9:25 每分钟一个点)
        const mockData: AuctionDataPoint[] = [
          { time: '09:15', price: 12.30, buy1_volume: 10000, sell1_volume: 8000 },
          { time: '09:16', price: 12.35, buy1_volume: 15000, sell1_volume: 9000 },
          { time: '09:17', price: 12.40, buy1_volume: 20000, sell1_volume: 10000 },
          { time: '09:18', price: 12.42, buy1_volume: 25000, sell1_volume: 12000 },
          { time: '09:19', price: 12.45, buy1_volume: 30000, sell1_volume: 15000 },
          { time: '09:20', price: 12.48, buy1_volume: 35000, sell1_volume: 18000 },
          { time: '09:21', price: 12.50, buy1_volume: 40000, sell1_volume: 20000 },
          { time: '09:22', price: 12.52, buy1_volume: 45000, sell1_volume: 22000 },
          { time: '09:23', price: 12.55, buy1_volume: 50000, sell1_volume: 25000 },
          { time: '09:24', price: 12.58, buy1_volume: 55000, sell1_volume: 28000 },
          { time: '09:25', price: 12.60, buy1_volume: 60000, sell1_volume: 30000 },
        ];
        setData(mockData);
      } catch (error) {
        console.error('Failed to fetch auction data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchAuctionData();

    // 定时刷新 (每5秒)
    const interval = setInterval(fetchAuctionData, 5000);
    return () => clearInterval(interval);
  }, [code]);

  const getOption = () => ({
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
        label: {
          backgroundColor: '#6a7985',
        },
      },
      formatter: (params: any) => {
        if (!params || params.length === 0) return '';
        const time = params[0].axisValue;
        let result = `<strong>${time}</strong><br/>`;
        params.forEach((param: any) => {
          result += `${param.marker} ${param.seriesName}: ${param.value}<br/>`;
        });
        return result;
      },
    },
    legend: {
      data: ['价格', '买量', '卖量'],
      top: 0,
    },
    grid: [
      {
        left: '10%',
        right: '10%',
        top: '15%',
        height: '35%',
      },
      {
        left: '10%',
        right: '10%',
        top: '60%',
        height: '25%',
      },
    ],
    xAxis: [
      {
        type: 'category',
        boundaryGap: false,
        data: data.map((d) => d.time),
        axisLabel: {
          fontSize: 11,
        },
      },
      {
        type: 'category',
        boundaryGap: false,
        data: data.map((d) => d.time),
        axisLabel: {
          fontSize: 11,
        },
        gridIndex: 1,
      },
    ],
    yAxis: [
      {
        type: 'value',
        name: '价格 (元)',
        position: 'left',
        axisLabel: {
          formatter: '{value}',
        },
        scale: true,
      },
      {
        type: 'value',
        name: '量',
        position: 'left',
        axisLabel: {
          formatter: (value: number) => {
            if (value >= 10000) {
              return `${(value / 10000).toFixed(1)}万`;
            }
            return value.toString();
          },
        },
        scale: true,
        gridIndex: 1,
      },
    ],
    series: [
      {
        name: '价格',
        type: 'line',
        smooth: true,
        data: data.map((d) => d.price.toFixed(2)),
        itemStyle: {
          color: '#cf1322',
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(207, 19, 34, 0.3)' },
              { offset: 1, color: 'rgba(207, 19, 34, 0.05)' },
            ],
          },
        },
        lineStyle: {
          width: 2,
        },
      },
      {
        name: '买量',
        type: 'bar',
        xAxisIndex: 1,
        yAxisIndex: 1,
        data: data.map((d) => d.buy1_volume),
        itemStyle: {
          color: '#cf1322',
        },
        barWidth: '40%',
      },
      {
        name: '卖量',
        type: 'bar',
        xAxisIndex: 1,
        yAxisIndex: 1,
        data: data.map((d) => d.sell1_volume),
        itemStyle: {
          color: '#3f8600',
        },
        barWidth: '40%',
      },
    ],
  });

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '40px 0' }}>
        <Spin tip="加载竞价数据..." />
      </div>
    );
  }

  if (data.length === 0) {
    return <Empty description="暂无竞价数据" />;
  }

  return (
    <div style={{ width: '100%', height: 350 }}>
      <ReactECharts option={getOption()} style={{ height: '100%', width: '100%' }} />
    </div>
  );
}

export default AuctionChart;
