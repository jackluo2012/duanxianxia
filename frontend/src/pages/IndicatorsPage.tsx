/**
 * 技术指标分析页面
 * 包含MA、MACD、KDJ、RSI等技术指标的图表展示
 */

import { useState, useMemo } from 'react';
import {
  Card,
  Row,
  Col,
  Input,
  Button,
  Tabs,
  Statistic,
  Typography,
  Space,
  Tag,
  Alert,
  message,
} from 'antd';
import {
  SearchOutlined,
  ReloadOutlined,
  LineChartOutlined,
  RiseOutlined,
  FallOutlined,
} from '@ant-design/icons';
import ReactECharts from 'echarts-for-react';
import { useIndicatorData } from '../hooks/useIndicatorData';

const { Title, Text } = Typography;

function IndicatorsPage() {
  const [code, setCode] = useState('000001');
  const [activeTab, setActiveTab] = useState('ma');

  const { maData, macdData, kdjData, rsiData, loading, loadAll } = useIndicatorData();

  /**
   * 加载指标数据
   */
  const handleLoad = async () => {
    if (!code || code.trim() === '') {
      message.warning('请输入股票代码');
      return;
    }
    await loadAll(code.trim(), 100);
  };

  /**
   * MA图表配置
   */
  const maChartOption = useMemo(() => {
    if (maData.length === 0) return null;

    return {
      title: {
        text: '移动平均线 (MA)',
        left: 'center',
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
      },
      legend: {
        data: ['MA5', 'MA10', 'MA20', 'MA60'],
        top: 30,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: maData.map((d) => d.date),
        axisLine: { lineStyle: { color: '#888' } },
      },
      yAxis: {
        type: 'value',
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { lineStyle: { color: '#ddd', type: 'dashed' } },
      },
      series: [
        {
          name: 'MA5',
          type: 'line',
          data: maData.map((d) => d.ma5),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#cf1322', width: 2 },
        },
        {
          name: 'MA10',
          type: 'line',
          data: maData.map((d) => d.ma10),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#faad14', width: 2 },
        },
        {
          name: 'MA20',
          type: 'line',
          data: maData.map((d) => d.ma20),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#1890ff', width: 2 },
        },
        {
          name: 'MA60',
          type: 'line',
          data: maData.map((d) => d.ma60),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#3f8600', width: 2 },
        },
      ],
    };
  }, [maData]);

  /**
   * MACD图表配置
   */
  const macdChartOption = useMemo(() => {
    if (macdData.length === 0) return null;

    return {
      title: {
        text: 'MACD 指标',
        left: 'center',
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
      },
      legend: {
        data: ['DIF', 'DEA', 'BAR'],
        top: 30,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: macdData.map((d) => d.date),
        axisLine: { lineStyle: { color: '#888' } },
      },
      yAxis: {
        type: 'value',
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { lineStyle: { color: '#ddd', type: 'dashed' } },
      },
      series: [
        {
          name: 'DIF',
          type: 'line',
          data: macdData.map((d) => d.dif),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#1890ff', width: 2 },
        },
        {
          name: 'DEA',
          type: 'line',
          data: macdData.map((d) => d.dea),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#faad14', width: 2 },
        },
        {
          name: 'BAR',
          type: 'bar',
          data: macdData.map((d) => ({
            value: d.bar,
            itemStyle: {
              color: (d.bar || 0) > 0 ? '#cf1322' : '#3f8600',
            },
          })),
        },
      ],
    };
  }, [macdData]);

  /**
   * KDJ图表配置
   */
  const kdjChartOption = useMemo(() => {
    if (kdjData.length === 0) return null;

    return {
      title: {
        text: 'KDJ 指标',
        left: 'center',
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
      },
      legend: {
        data: ['K', 'D', 'J'],
        top: 30,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: kdjData.map((d) => d.date),
        axisLine: { lineStyle: { color: '#888' } },
      },
      yAxis: {
        type: 'value',
        min: 0,
        max: 100,
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { lineStyle: { color: '#ddd', type: 'dashed' } },
      },
      series: [
        {
          name: 'K',
          type: 'line',
          data: kdjData.map((d) => d.k),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#cf1322', width: 2 },
        },
        {
          name: 'D',
          type: 'line',
          data: kdjData.map((d) => d.d),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#faad14', width: 2 },
        },
        {
          name: 'J',
          type: 'line',
          data: kdjData.map((d) => d.j),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#1890ff', width: 2 },
        },
      ],
      // 添加参考线
      visualMap: {
        show: false,
        pieces: [
          { gt: 80, color: '#cf1322' },
          { lt: 20, color: '#3f8600' },
        ],
      },
    };
  }, [kdjData]);

  /**
   * RSI图表配置
   */
  const rsiChartOption = useMemo(() => {
    if (rsiData.length === 0) return null;

    return {
      title: {
        text: 'RSI 指标',
        left: 'center',
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
      },
      legend: {
        data: ['RSI6', 'RSI12', 'RSI24'],
        top: 30,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: rsiData.map((d) => d.date),
        axisLine: { lineStyle: { color: '#888' } },
      },
      yAxis: {
        type: 'value',
        min: 0,
        max: 100,
        axisLine: { lineStyle: { color: '#888' } },
        splitLine: { lineStyle: { color: '#ddd', type: 'dashed' } },
      },
      series: [
        {
          name: 'RSI6',
          type: 'line',
          data: rsiData.map((d) => d.rsi6),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#cf1322', width: 2 },
        },
        {
          name: 'RSI12',
          type: 'line',
          data: rsiData.map((d) => d.rsi12),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#faad14', width: 2 },
        },
        {
          name: 'RSI24',
          type: 'line',
          data: rsiData.map((d) => d.rsi24),
          smooth: true,
          symbol: 'none',
          lineStyle: { color: '#1890ff', width: 2 },
        },
      ],
    };
  }, [rsiData]);

  /**
   * 获取最新指标值
   */
  const getLatestValues = () => {
    const kdjLatest = kdjData[kdjData.length - 1];
    const rsiLatest = rsiData[rsiData.length - 1];

    return {
      kdj: kdjLatest ? { k: kdjLatest.k, d: kdjLatest.d, j: kdjLatest.j } : null,
      rsi: rsiLatest ? { rsi6: rsiLatest.rsi6, rsi12: rsiLatest.rsi12, rsi24: rsiLatest.rsi24 } : null,
    };
  };

  const latestValues = getLatestValues();

  /**
   * KDJ信号判断
   */
  const getKDJSignal = () => {
    if (!latestValues.kdj) return null;

    const { k, d, j } = latestValues.kdj;
    if (k && d && k > d && j && j > 100) {
      return { signal: '超买', type: 'warning', icon: <RiseOutlined /> };
    }
    if (k && d && k < d && j && j < 0) {
      return { signal: '超卖', type: 'success', icon: <FallOutlined /> };
    }
    if (k && d && k > d) {
      return { signal: '金叉买入', type: 'warning', icon: <RiseOutlined /> };
    }
    if (k && d && k < d) {
      return { signal: '死叉卖出', type: 'success', icon: <FallOutlined /> };
    }
    return null;
  };

  /**
   * RSI信号判断
   */
  const getRSISignal = () => {
    if (!latestValues.rsi) return null;

    const { rsi6, rsi12 } = latestValues.rsi;
    if (rsi6 && rsi6 > 80) {
      return { signal: '超买', type: 'warning', text: 'RSI6 > 80' };
    }
    if (rsi6 && rsi6 < 20) {
      return { signal: '超卖', type: 'success', text: 'RSI6 < 20' };
    }
    if (rsi12 && rsi12 > 80) {
      return { signal: '强势', type: 'warning', text: 'RSI12 > 80' };
    }
    if (rsi12 && rsi12 < 20) {
      return { signal: '弱势', type: 'success', text: 'RSI12 < 20' };
    }
    return null;
  };

  const kdjSignal = getKDJSignal();
  const rsiSignal = getRSISignal();

  return (
    <div style={{ padding: 24 }}>
      {/* 标题和搜索栏 */}
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={18}>
          <Title level={2} style={{ margin: 0 }}>
            <LineChartOutlined /> 技术指标分析
          </Title>
        </Col>
        <Col span={6} style={{ textAlign: 'right' }}>
          <Button
            icon={<ReloadOutlined />}
            onClick={handleLoad}
            loading={loading}
          >
            刷新
          </Button>
        </Col>
      </Row>

      {/* 搜索框 */}
      <Card style={{ marginBottom: 16 }}>
        <Space size="large">
          <Input
            placeholder="输入股票代码（如：000001）"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            onPressEnter={handleLoad}
            style={{ width: 250 }}
            suffix={
              <Button
                icon={<SearchOutlined />}
                onClick={handleLoad}
                type="primary"
                size="small"
              >
                查询
              </Button>
            }
            allowClear
          />
          <Text type="secondary">支持查询股票的MA、MACD、KDJ、RSI等技术指标</Text>
        </Space>
      </Card>

      {/* 指标图表 */}
      {maData.length > 0 && (
        <Card>
          <Tabs activeKey={activeTab} onChange={setActiveTab}>
            {/* MA */}
            <Tabs.TabPane tab="移动平均线" key="ma">
              <ReactECharts
                option={maChartOption}
                style={{ height: 450 }}
                opts={{ renderer: 'canvas' }}
              />
            </Tabs.TabPane>

            {/* MACD */}
            <Tabs.TabPane tab="MACD" key="macd">
              <ReactECharts
                option={macdChartOption}
                style={{ height: 450 }}
                opts={{ renderer: 'canvas' }}
              />
            </Tabs.TabPane>

            {/* KDJ */}
            <Tabs.TabPane tab="KDJ" key="kdj">
              {/* KDJ信号提示 */}
              {kdjSignal && (
                <Alert
                  message={
                    <Space>
                      {kdjSignal.icon}
                      <Text strong>信号: {kdjSignal.signal}</Text>
                    </Space>
                  }
                  type={kdjSignal.type as any}
                  showIcon
                  style={{ marginBottom: 16 }}
                />
              )}

              {/* 最新KDJ值 */}
              {latestValues.kdj && (
                <Row gutter={16} style={{ marginBottom: 16 }}>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="K值"
                        value={latestValues.kdj.k || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.kdj.k || 0) > 80
                              ? '#cf1322'
                              : (latestValues.kdj.k || 0) < 20
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="D值"
                        value={latestValues.kdj.d || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.kdj.d || 0) > 80
                              ? '#cf1322'
                              : (latestValues.kdj.d || 0) < 20
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="J值"
                        value={latestValues.kdj.j || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.kdj.j || 0) > 100
                              ? '#cf1322'
                              : (latestValues.kdj.j || 0) < 0
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                </Row>
              )}

              <ReactECharts
                option={kdjChartOption}
                style={{ height: 400 }}
                opts={{ renderer: 'canvas' }}
              />
            </Tabs.TabPane>

            {/* RSI */}
            <Tabs.TabPane tab="RSI" key="rsi">
              {/* RSI信号提示 */}
              {rsiSignal && (
                <Alert
                  message={
                    <Space>
                      <Text strong>信号: {rsiSignal.signal}</Text>
                      <Tag>{rsiSignal.text}</Tag>
                    </Space>
                  }
                  type={rsiSignal.type as any}
                  showIcon
                  style={{ marginBottom: 16 }}
                />
              )}

              {/* 最新RSI值 */}
              {latestValues.rsi && (
                <Row gutter={16} style={{ marginBottom: 16 }}>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="RSI6"
                        value={latestValues.rsi.rsi6 || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.rsi.rsi6 || 0) > 70
                              ? '#cf1322'
                              : (latestValues.rsi.rsi6 || 0) < 30
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="RSI12"
                        value={latestValues.rsi.rsi12 || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.rsi.rsi12 || 0) > 70
                              ? '#cf1322'
                              : (latestValues.rsi.rsi12 || 0) < 30
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                  <Col span={8}>
                    <Card size="small">
                      <Statistic
                        title="RSI24"
                        value={latestValues.rsi.rsi24 || 0}
                        precision={2}
                        valueStyle={{
                          color:
                            (latestValues.rsi.rsi24 || 0) > 70
                              ? '#cf1322'
                              : (latestValues.rsi.rsi24 || 0) < 30
                              ? '#3f8600'
                              : '#666',
                        }}
                      />
                    </Card>
                  </Col>
                </Row>
              )}

              <ReactECharts
                option={rsiChartOption}
                style={{ height: 400 }}
                opts={{ renderer: 'canvas' }}
              />
            </Tabs.TabPane>
          </Tabs>
        </Card>
      )}

      {/* 空状态 */}
      {maData.length === 0 && !loading && (
        <Card>
          <div
            style={{
              textAlign: 'center',
              padding: '80px 0',
              color: '#999',
            }}
          >
            <LineChartOutlined style={{ fontSize: 64, marginBottom: 16 }} />
            <div style={{ fontSize: 16, marginBottom: 8 }}>请输入股票代码查询技术指标</div>
            <Text type="secondary">支持MA、MACD、KDJ、RSI等多种技术指标</Text>
          </div>
        </Card>
      )}
    </div>
  );
}

export default IndicatorsPage;
