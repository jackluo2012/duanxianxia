/**
 * 增强的K线图表组件
 * 支持缩放、十字线、技术指标叠加
 */

import React, { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import { HistoryPoint } from '../../api/quotes';
import { config } from '../../config';

export interface TechnicalIndicator {
  type: 'ma' | 'ema' | 'boll' | 'none';
  period?: number; // MA/EMA的周期
  color?: string;
}

export interface KLineChartAdvancedProps {
  data: HistoryPoint[];
  period: string;
  loading?: boolean;
  height?: number | string;
  indicators?: TechnicalIndicator[];
  enableZoom?: boolean;
  enableDataZoom?: boolean;
  showVolume?: boolean;
}

// 计算移动平均线
function calculateMA(data: HistoryPoint[], period: number): number[] {
  const result: number[] = [];
  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      result.push(NaN);
      continue;
    }
    let sum = 0;
    for (let j = 0; j < period; j++) {
      sum += data[i - j].close || data[i - j].price || 0;
    }
    result.push(sum / period);
  }
  return result;
}

// 计算布林带
function calculateBOLL(data: HistoryPoint[], period: number = 20, multiplier: number = 2) {
  const ma = calculateMA(data, period);
  const upper: number[] = [];
  const lower: number[] = [];

  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      upper.push(NaN);
      lower.push(NaN);
      continue;
    }

    let sum = 0;
    for (let j = 0; j < period; j++) {
      const price = data[i - j].close || data[i - j].price || 0;
      sum += Math.pow(price - ma[i], 2);
    }
    const std = Math.sqrt(sum / period);
    upper.push(ma[i] + multiplier * std);
    lower.push(ma[i] - multiplier * std);
  }

  return { ma, upper, lower };
}

export default function KLineChartAdvanced({
  data,
  period,
  loading = false,
  height = 400,
  indicators = [],
  enableZoom = true,
  enableDataZoom = true,
  showVolume = true,
}: KLineChartAdvancedProps) {
  const isKLine = period === '5m' || period === '15m' || period === '30m' || period === '60m' || period === '1d';

  // 数据采样
  const sampledData = useMemo(() => {
    const threshold = isKLine ? config.chartSamplingThreshold.kline : config.chartSamplingThreshold.minute;
    if (data.length <= threshold) {
      return data;
    }

    const step = Math.ceil(data.length / threshold);
    const sampled: HistoryPoint[] = [data[0]];

    for (let i = 1; i < data.length - 1; i += step) {
      sampled.push(data[i]);
    }

    sampled.push(data[data.length - 1]);
    return sampled;
  }, [data, isKLine]);

  // 计算技术指标
  const indicatorSeries = useMemo(() => {
    const series: any[] = [];

    indicators.forEach((indicator) => {
      if (indicator.type === 'ma' && indicator.period) {
        const maData = calculateMA(sampledData, indicator.period);
        series.push({
          name: `MA${indicator.period}`,
          type: 'line',
          data: maData,
          smooth: true,
          symbol: 'none',
          lineStyle: {
            opacity: 0.8,
            width: 1,
          },
          itemStyle: {
            color: indicator.color || '#fa0',
          },
        });
      } else if (indicator.type === 'boll') {
        const boll = calculateBOLL(sampledData);
        series.push(
          {
            name: 'BOLL Upper',
            type: 'line',
            data: boll.upper,
            smooth: true,
            symbol: 'none',
            lineStyle: { opacity: 0.5, width: 1 },
            itemStyle: { color: '#f00' },
          },
          {
            name: 'BOLL',
            type: 'line',
            data: boll.ma,
            smooth: true,
            symbol: 'none',
            lineStyle: { opacity: 0.5, width: 1 },
            itemStyle: { color: '#ff0' },
          },
          {
            name: 'BOLL Lower',
            type: 'line',
            data: boll.lower,
            smooth: true,
            symbol: 'none',
            lineStyle: { opacity: 0.5, width: 1 },
            itemStyle: { color: '#0f0' },
          }
        );
      }
    });

    return series;
  }, [sampledData, indicators]);

  // K线数据
  const klineData = useMemo(() => {
    return sampledData.map((d) => [
      d.open || d.price || 0,
      d.close || d.price || 0,
      d.low || d.price || 0,
      d.high || d.price || 0,
    ]);
  }, [sampledData]);

  // 分时数据
  const lineData = useMemo(() => {
    return sampledData.map((d) => d.price || 0);
  }, [sampledData]);

  // 时间轴
  const xAxisData = useMemo(() => {
    return sampledData.map((d) => d.time);
  }, [sampledData]);

  // 成交量数据
  const volumeData = useMemo(() => {
    return sampledData.map((d) => ({
      value: d.vol,
      // 根据涨跌设置颜色
      itemStyle: {
        color: (d.close || d.price) >= (d.open || d.price) ? '#ef5350' : '#26a69a',
      },
    }));
  }, [sampledData]);

  // 图表配置
  const option = useMemo(() => {
    const baseOption = {
      animation: true,
      animationDuration: 300,
      title: {
        text: isKLine
          ? `${period === '5m' ? '5分钟' : period === '1d' ? '日' : period}K线`
          : '实时分时图',
        left: 'center',
        textStyle: {
          fontSize: 14,
        },
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'cross',
          animation: false,
        },
        backgroundColor: 'rgba(50, 50, 50, 0.9)',
        borderColor: '#333',
        borderWidth: 1,
        textStyle: {
          color: '#fff',
          fontSize: 12,
        },
        formatter: (params: any) => {
          if (!params || params.length === 0) return '';

          const dataIndex = params[0].dataIndex;
          const dataPoint = sampledData[dataIndex];

          if (!dataPoint) return '';

          let result = `<div style="padding: 8px;">
            <div style="margin-bottom: 6px; font-weight: bold;">
              ${dataPoint.time}
            </div>`;

          if (isKLine) {
            result += `
              <div>开: ${dataPoint.open?.toFixed(2) || '-'}</div>
              <div>高: ${dataPoint.high?.toFixed(2) || '-'}</div>
              <div>低: ${dataPoint.low?.toFixed(2) || '-'}</div>
              <div>收: ${dataPoint.close?.toFixed(2) || '-'}</div>
              <div>量: ${(dataPoint.vol / 10000).toFixed(2)}万</div>
            `;
          } else {
            result += `
              <div>价格: ${dataPoint.price?.toFixed(2) || '-'}</div>
              <div>量: ${(dataPoint.vol / 10000).toFixed(2)}万</div>
            `;
          }

          // 技术指标
          params.forEach((param: any) => {
            if (param.seriesName && param.value && !isNaN(param.value)) {
              result += `<div>${param.seriesName}: ${param.value.toFixed(2)}</div>`;
            }
          });

          result += '</div>';
          return result;
        },
      },
      axisPointer: {
        link: [
          {
            xAxisIndex: 'all',
          },
        ],
        label: {
          backgroundColor: '#777',
        },
      },
      grid: [
        {
          left: '10%',
          right: '8%',
          top: '15%',
          height: showVolume ? '50%' : '70%',
        },
        {
          left: '10%',
          right: '8%',
          top: showVolume ? '70%' : '15%',
          height: showVolume ? '15%' : '0%',
        },
      ],
      xAxis: [
        {
          type: 'category',
          data: xAxisData,
          boundaryGap: false,
          axisLine: { onZero: false },
          splitLine: { show: false },
          min: 'dataMin',
          max: 'dataMax',
          axisPointer: {
            z: 100,
          },
        },
        {
          type: 'category',
          gridIndex: 1,
          data: xAxisData,
          boundaryGap: false,
          axisLine: { onZero: false },
          axisTick: { show: false },
          splitLine: { show: false },
          axisLabel: { show: false },
          min: 'dataMin',
          max: 'dataMax',
        },
      ],
      yAxis: [
        {
          scale: true,
          splitArea: {
            show: true,
          },
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
        {
          scale: true,
          gridIndex: 1,
          splitNumber: 2,
          axisLabel: { show: false },
          axisLine: { show: false },
          axisTick: { show: false },
          splitLine: { show: false },
        },
      ],
      dataZoom: enableDataZoom
        ? [
            {
              type: 'inside',
              xAxisIndex: [0, 1],
              start: 70,
              end: 100,
            },
            {
              show: true,
              xAxisIndex: [0, 1],
              type: 'slider',
              top: '90%',
              start: 70,
              end: 100,
            },
          ]
        : undefined,
      series: [
        // K线或分时线
        ...(isKLine
          ? [
              {
                name: 'K线',
                type: 'candlestick',
                data: klineData,
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
                data: lineData,
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
            ]),
        // 技术指标
        ...indicatorSeries,
        // 成交量
        ...(showVolume
          ? [
              {
                name: '成交量',
                type: 'bar',
                xAxisIndex: 1,
                yAxisIndex: 1,
                data: volumeData,
                itemStyle: {
                  borderRadius: 2,
                },
              },
            ]
          : []),
      ],
    };

    return baseOption;
  }, [
    sampledData,
    isKLine,
    klineData,
    lineData,
    volumeData,
    xAxisData,
    indicatorSeries,
    period,
    enableZoom,
    enableDataZoom,
    showVolume,
  ]);

  if (loading || sampledData.length === 0) {
    return (
      <div
        style={{
          height: typeof height === 'number' ? height : 400,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#999',
          background: '#fafafa',
          borderRadius: 4,
        }}
      >
        {loading ? (
          <div>
            <div style={{ fontSize: 40, marginBottom: 16 }}>⏳</div>
            <div>加载中...</div>
          </div>
        ) : (
          <div>
            <div style={{ fontSize: 40, marginBottom: 16 }}>📊</div>
            <div>暂无数据</div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div style={{ position: 'relative' }}>
      <ReactECharts
        option={option}
        style={{ height }}
        opts={{ renderer: 'canvas' }}
        notMerge={true}
        lazyUpdate={true}
      />
    </div>
  );
}
