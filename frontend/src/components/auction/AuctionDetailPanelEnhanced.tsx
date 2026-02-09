/**
 * 增强的竞价详情面板组件
 * 展示竞价详细信息、历史数据、买卖盘等
 */

import { Card, Col, Row, Statistic, Tag, Typography, Divider, Space, Progress } from 'antd';
import { RiseOutlined, FallOutlined, MinusOutlined } from '@ant-design/icons';
import { useEffect, useState } from 'react';
import ReactECharts from 'echarts-for-react';
import { fetchAuctionDetail, AuctionStock, AuctionHistoryPoint } from '../../api/auction';

const { Title, Text } = Typography;

interface AuctionDetailPanelProps {
  stock: AuctionStock | null;
}

function AuctionDetailPanel({ stock }: AuctionDetailPanelProps) {
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<AuctionHistoryPoint[]>([]);

  useEffect(() => {
    if (!stock) {
      setHistory([]);
      return;
    }

    const fetchDetail = async () => {
      setLoading(true);
      try {
        const response = await fetchAuctionDetail(stock.code);
        setHistory(response.history || []);
      } catch (err) {
        console.error('Failed to fetch auction detail:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchDetail();
  }, [stock]);

  if (!stock) {
    return (
      <Card
        style={{ height: 600, display: 'flex', alignItems: 'center', justifyContent: 'center' }}
      >
        <div style={{ textAlign: 'center', color: '#999' }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>📊</div>
          <div>请选择股票查看详情</div>
        </div>
      </Card>
    );
  }

  const formatAmount = (amount: number) => {
    if (amount >= 100000000) {
      return `${(amount / 100000000).toFixed(2)}亿`;
    } else if (amount >= 10000) {
      return `${(amount / 10000).toFixed(2)}万`;
    }
    return amount.toFixed(2);
  };

  const changePercent = stock.change_percent;
  const isUp = changePercent > 0;
  const isDown = changePercent < 0;

  // 计算买卖封单比例
  const totalSealed = stock.sealed_amount_buy + stock.sealed_amount_sell;
  const buyRatio = totalSealed > 0 ? (stock.sealed_amount_buy / totalSealed) * 100 : 0;
  const sellRatio = totalSealed > 0 ? (stock.sealed_amount_sell / totalSealed) * 100 : 0;

  // 历史数据图表配置
  const chartOption = {
    animation: false,
    title: {
      text: '竞价走势',
      left: 'center',
      textStyle: { fontSize: 14 },
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' },
    },
    legend: {
      data: ['价格', '买封', '卖封'],
      top: 25,
    },
    grid: {
      left: '10%',
      right: '5%',
      bottom: '10%',
      top: '20%',
    },
    xAxis: {
      type: 'category',
      data: history.map((h) => h.time),
      axisLine: { lineStyle: { color: '#888' } },
    },
    yAxis: [
      {
        type: 'value',
        name: '价格',
        position: 'left',
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { lineStyle: { color: '#ddd', type: 'dashed' } },
      },
      {
        type: 'value',
        name: '封单(万)',
        position: 'right',
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { show: false },
      },
    ],
    series: [
      {
        name: '价格',
        type: 'line',
        data: history.map((h) => h.price),
        smooth: true,
        symbol: 'none',
        lineStyle: { color: '#1890ff', width: 2 },
      },
      {
        name: '买封',
        type: 'bar',
        yAxisIndex: 1,
        data: history.map((h) => (h.buy_sealed / 10000).toFixed(2)),
        itemStyle: { color: '#cf1322' },
      },
      {
        name: '卖封',
        type: 'bar',
        yAxisIndex: 1,
        data: history.map((h) => (h.sell_sealed / 10000).toFixed(2)),
        itemStyle: { color: '#3f8600' },
      },
    ],
  };

  return (
    <Card
      loading={loading}
      title={
        <Space>
          <Text strong>{stock.name}</Text>
          <Text type="secondary">({stock.code})</Text>
          <Tag
            color={isUp ? 'red' : isDown ? 'green' : 'default'}
            icon={isUp ? <RiseOutlined /> : isDown ? <FallOutlined /> : <MinusOutlined />}
          >
            {changePercent >= 0 ? '+' : ''}{changePercent.toFixed(2)}%
          </Tag>
        </Space>
      }
      extra={
        <Text type="secondary" style={{ fontSize: 12 }}>
          竞价时段
        </Text>
      }
      style={{ height: '100%' }}
      bodyStyle={{ maxHeight: 650, overflowY: 'auto' }}
    >
      {/* 核心数据统计 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={8}>
          <Statistic
            title="当前价格"
            value={stock.price}
            precision={2}
            prefix="¥"
            valueStyle={{
              color: isUp ? '#cf1322' : isDown ? '#3f8600' : '#666',
              fontSize: 20,
            }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="买封金额"
            value={stock.sealed_amount_buy}
            formatter={(value) => formatAmount(Number(value))}
            valueStyle={{ color: '#cf1322', fontSize: 18 }}
          />
        </Col>
        <Col span={8}>
          <Statistic
            title="卖封金额"
            value={stock.sealed_amount_sell}
            formatter={(value) => formatAmount(Number(value))}
            valueStyle={{ color: '#3f8600', fontSize: 18 }}
          />
        </Col>
      </Row>

      <Divider />

      {/* 强度评分 */}
      {stock.intensity_score && (
        <div style={{ marginBottom: 16 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
            <Text strong>抢筹强度</Text>
            <Text type="secondary">{stock.intensity_score.toFixed(1)}/100</Text>
          </div>
          <Progress
            percent={stock.intensity_score}
            strokeColor={{
              '0%': '#108ee9',
              '50%': '#faad14',
              '100%': '#cf1322',
            }}
            format={(percent) => (
              <span style={{ fontSize: 14, fontWeight: 'bold' }}>
                {percent?.toFixed(1)}
              </span>
            )}
          />
        </div>
      )}

      {/* 买卖封单比例 */}
      <div style={{ marginBottom: 16 }}>
        <Text strong style={{ marginBottom: 8, display: 'block' }}>
          封单分布
        </Text>
        <Row gutter={8}>
          <Col span={12}>
            <div style={{ marginBottom: 8 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                <Text type="secondary">买封</Text>
                <Text style={{ color: '#cf1322' }}>{buyRatio.toFixed(1)}%</Text>
              </div>
              <Progress
                percent={buyRatio}
                strokeColor="#cf1322"
                showInfo={false}
                size="small"
              />
            </div>
          </Col>
          <Col span={12}>
            <div style={{ marginBottom: 8 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                <Text type="secondary">卖封</Text>
                <Text style={{ color: '#3f8600' }}>{sellRatio.toFixed(1)}%</Text>
              </div>
              <Progress
                percent={sellRatio}
                strokeColor="#3f8600"
                showInfo={false}
                size="small"
              />
            </div>
          </Col>
        </Row>
      </div>

      <Divider />

      {/* 竞价走势图 */}
      <div style={{ marginBottom: 16 }}>
        <Title level={5} style={{ marginBottom: 12 }}>
          竞价走势
        </Title>
        <ReactECharts
          option={chartOption}
          style={{ height: 250 }}
          opts={{ renderer: 'canvas' }}
          notMerge={true}
        />
      </div>

      <Divider />

      {/* 详细数据 */}
      <div>
        <Title level={5} style={{ marginBottom: 12 }}>
          详细数据
        </Title>
        <Row gutter={[16, 8]}>
          <Col span={12}>
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0' }}>
              <Text type="secondary">今开价:</Text>
              <Text strong>¥{stock.open_price?.toFixed(2) || '-'}</Text>
            </div>
          </Col>
          <Col span={12}>
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0' }}>
              <Text type="secondary">昨收价:</Text>
              <Text>¥{stock.preclose_price?.toFixed(2) || '-'}</Text>
            </div>
          </Col>
          <Col span={12}>
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0' }}>
              <Text type="secondary">成交量:</Text>
              <Text strong>{stock.volume ? formatAmount(stock.volume) : '-'}</Text>
            </div>
          </Col>
          <Col span={12}>
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0' }}>
              <Text type="secondary">成交额:</Text>
              <Text strong>{stock.amount ? formatAmount(stock.amount) : '-'}</Text>
            </div>
          </Col>
        </Row>
      </div>

      {/* 操作提示 */}
      {stock.updateTime && (
        <div style={{
          marginTop: 16,
          padding: '8px 12px',
          background: '#f6ffed',
          border: '1px solid #b7eb8f',
          borderRadius: 4,
          fontSize: 12,
          color: '#52c41a',
        }}>
          💡 数据更新时间: {new Date(stock.updateTime).toLocaleString()}
        </div>
      )}
    </Card>
  );
}

export default AuctionDetailPanel;
