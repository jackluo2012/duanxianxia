import { useState, useEffect } from 'react';
import { Card, Table, Tag, Badge, Space } from 'antd';
import { getAlertHistory, type AlertEvent } from '../../api/alerts';

function AlertHistory() {
  const [alerts, setAlerts] = useState<AlertEvent[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchAlerts();
    // 每 30 秒刷新一次
    const interval = setInterval(fetchAlerts, 30000);
    return () => clearInterval(interval);
  }, []);

  const fetchAlerts = async () => {
    setLoading(true);
    try {
      const data = await getAlertHistory(100);
      setAlerts(data);
    } catch (error) {
      console.error('Failed to fetch alert history:', error);
    } finally {
      setLoading(false);
    }
  };

  const getSeverityColor = (severity: string) => {
    const colorMap: Record<string, string> = {
      info: 'blue',
      warning: 'orange',
      critical: 'red',
    };
    return colorMap[severity] || 'default';
  };

  const getSeverityText = (severity: string) => {
    const textMap: Record<string, string> = {
      info: '信息',
      warning: '警告',
      critical: '严重',
    };
    return textMap[severity] || severity;
  };

  const columns = [
    {
      title: '时间',
      dataIndex: 'triggered_at',
      key: 'triggered_at',
      width: 180,
      render: (time: string) => new Date(time).toLocaleString('zh-CN'),
    },
    {
      title: '严重程度',
      dataIndex: 'severity',
      key: 'severity',
      width: 100,
      render: (severity: string) => (
        <Tag color={getSeverityColor(severity)}>{getSeverityText(severity)}</Tag>
      ),
    },
    {
      title: '股票',
      key: 'stock',
      width: 150,
      render: (_: any, record: AlertEvent) => (
        <Space>
          <span>{record.stock_name}</span>
          <Badge count={record.stock_code} style={{ backgroundColor: '#52c41a' }} />
        </Space>
      ),
    },
    {
      title: '告警消息',
      dataIndex: 'message',
      key: 'message',
      ellipsis: true,
    },
    {
      title: '触发规则',
      dataIndex: 'rule_name',
      key: 'rule_name',
      width: 150,
    },
  ];

  return (
    <Card
      title="告警历史"
      extra={<Badge count={alerts.length} showZero title="总告警数" />}
    >
      <Table
        dataSource={alerts}
        columns={columns}
        rowKey="id"
        loading={loading}
        pagination={{ pageSize: 20 }}
        scroll={{ x: 800 }}
        size="small"
      />
    </Card>
  );
}

export default AlertHistory;
