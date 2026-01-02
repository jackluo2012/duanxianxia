// 技术指标页面（简化版）

import React, { useState, useEffect } from 'react';
import { Card, Input, Button, Tabs, message, Row, Col, Statistic } from 'antd';
import { SearchOutlined, ReloadOutlined } from '@ant-design/icons';
import ReactECharts from 'echarts-for-react';
import {
  fetchMA,
  fetchMACD,
  fetchKDJ,
  fetchRSI,
  MAData,
  MACDData,
  KDJData,
  RSIData,
} from '../api/indicators';

const { TabPane } = Tabs;

const IndicatorsPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [code, setCode] = useState('000001');
  const [maData, setMAData] = useState<MAData[]>([]);
  const [macdData, setMACDData] = useState<MACDData[]>([]);
  const [kdjData, setKDJData] = useState<KDJData[]>([]);
  const [rsiData, setRSIData] = useState<RSIData[]>([]);

  const loadIndicators = async () => {
    if (!code) {
      message.warning('请输入股票代码');
      return;
    }
    setLoading(true);
    try {
      const [ma, macd, kdj, rsi] = await Promise.all([
        fetchMA(code, 100),
        fetchMACD(code, 100),
        fetchKDJ(code, 100),
        fetchRSI(code, 100),
      ]);
      setMAData(ma);
      setMACDData(macd);
      setKDJData(kdj);
      setRSIData(rsi);
    } catch (error) {
      message.error('获取技术指标失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadIndicators();
  }, []);

  const getMAOption = () => ({
    title: { text: '移动平均线' },
    tooltip: { trigger: 'axis' },
    legend: { data: ['MA5', 'MA10', 'MA20', 'MA60'] },
    xAxis: {
      type: 'category',
      data: maData.map((d) => d.date),
    },
    yAxis: { type: 'value' },
    series: [
      {
        name: 'MA5',
        type: 'line',
        data: maData.map((d) => d.ma5),
      },
      {
        name: 'MA10',
        type: 'line',
        data: maData.map((d) => d.ma10),
      },
      {
        name: 'MA20',
        type: 'line',
        data: maData.map((d) => d.ma20),
      },
      {
        name: 'MA60',
        type: 'line',
        data: maData.map((d) => d.ma60),
      },
    ],
  });

  const getMACDOption = () => ({
    title: { text: 'MACD 指标' },
    tooltip: { trigger: 'axis' },
    legend: { data: ['DIF', 'DEA', 'BAR'] },
    xAxis: {
      type: 'category',
      data: macdData.map((d) => d.date),
    },
    yAxis: { type: 'value' },
    series: [
      {
        name: 'DIF',
        type: 'line',
        data: macdData.map((d) => d.dif),
      },
      {
        name: 'DEA',
        type: 'line',
        data: macdData.map((d) => d.dea),
      },
      {
        name: 'BAR',
        type: 'bar',
        data: macdData.map((d) => d.bar),
      },
    ],
  });

  return (
    <div style={{ padding: 24 }}>
      <Card
        title="技术指标"
        extra={
          <Button icon={<ReloadOutlined />} onClick={loadIndicators} loading={loading}>
            刷新
          </Button>
        }
      >
        <Input
          placeholder="输入股票代码（如：000001）"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          onPressEnter={loadIndicators}
          suffix={
            <Button icon={<SearchOutlined />} onClick={loadIndicators} type="link">
              查询
            </Button>
          }
          style={{ marginBottom: 16 }}
        />

        {maData.length > 0 && (
          <Tabs>
            <TabPane tab="移动平均线" key="ma">
              <ReactECharts option={getMAOption()} style={{ height: 400 }} />
            </TabPane>
            <TabPane tab="MACD" key="macd">
              <ReactECharts option={getMACDOption()} style={{ height: 400 }} />
            </TabPane>
            <TabPane tab="KDJ" key="kdj">
              <Row gutter={16}>
                <Col span={8}>
                  <Statistic title="K值" value={kdjData[kdjData.length - 1]?.k || 0} precision={2} />
                </Col>
                <Col span={8}>
                  <Statistic title="D值" value={kdjData[kdjData.length - 1]?.d || 0} precision={2} />
                </Col>
                <Col span={8}>
                  <Statistic title="J值" value={kdjData[kdjData.length - 1]?.j || 0} precision={2} />
                </Col>
              </Row>
            </TabPane>
            <TabPane tab="RSI" key="rsi">
              <Row gutter={16}>
                <Col span={8}>
                  <Statistic title="RSI6" value={rsiData[rsiData.length - 1]?.rsi6 || 0} precision={2} />
                </Col>
                <Col span={8}>
                  <Statistic title="RSI12" value={rsiData[rsiData.length - 1]?.rsi12 || 0} precision={2} />
                </Col>
                <Col span={8}>
                  <Statistic title="RSI24" value={rsiData[rsiData.length - 1]?.rsi24 || 0} precision={2} />
                </Col>
              </Row>
            </TabPane>
          </Tabs>
        )}
      </Card>
    </div>
  );
};

export default IndicatorsPage;
