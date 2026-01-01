import { useState, useEffect } from 'react';
import { Button, Card, Form, InputNumber, Select, Table, Switch, Tag, message } from 'antd';
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons';
import { createAlertRule, deleteAlertRule, getAlertRules, type AlertRule } from '../../api/alerts';

const { Option } = Select;

function AlertConfig() {
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [loading, setLoading] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    fetchRules();
  }, []);

  const fetchRules = async () => {
    setLoading(true);
    try {
      const data = await getAlertRules();
      setRules(data);
    } catch (error) {
      console.error('Failed to fetch alert rules:', error);
      message.error('加载告警规则失败');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      await createAlertRule(
        values.name,
        { [values.ruleType]: { threshold: values.threshold } },
        values.enabled
      );
      message.success('告警规则创建成功');
      form.resetFields();
      fetchRules();
    } catch (error: any) {
      if (error.errorFields) {
        return; // 表单验证错误
      }
      console.error('Failed to create alert rule:', error);
      message.error('创建告警规则失败');
    }
  };

  const handleDelete = async (ruleId: string) => {
    try {
      await deleteAlertRule(ruleId);
      message.success('告警规则已删除');
      fetchRules();
    } catch (error) {
      console.error('Failed to delete alert rule:', error);
      message.error('删除告警规则失败');
    }
  };

  const getRuleTypeDisplay = (ruleType: string) => {
    const typeMap: Record<string, string> = {
      change_percent: '价格涨幅',
      sealed_amount: '封单金额',
      intensity_score: '强度评分',
      anomaly: '异动检测',
    };
    return typeMap[ruleType] || ruleType;
  };

  const getSeverityColor = (threshold: number, ruleType: string) => {
    if (ruleType === 'change_percent') {
      if (threshold >= 10) return 'red';
      if (threshold >= 5) return 'orange';
      return 'blue';
    }
    if (ruleType === 'sealed_amount') {
      if (threshold >= 10000) return 'red';
      if (threshold >= 5000) return 'orange';
      return 'blue';
    }
    return 'blue';
  };

  const columns = [
    {
      title: '规则名称',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: '规则类型',
      dataIndex: 'rule_type',
      key: 'rule_type',
      render: (ruleType: string) => getRuleTypeDisplay(ruleType),
    },
    {
      title: '阈值',
      key: 'threshold',
      render: (_: any, record: AlertRule) => {
        const config = record.rule_config;
        if (config.change_percent) {
          return (
            <Tag color={getSeverityColor(config.change_percent.threshold, 'change_percent')}>
              ≥ {config.change_percent.threshold}%
            </Tag>
          );
        }
        if (config.sealed_amount) {
          return (
            <Tag color={getSeverityColor(config.sealed_amount.threshold, 'sealed_amount')}>
              ≥ {config.sealed_amount.threshold} 万元
            </Tag>
          );
        }
        if (config.intensity_score) {
          return <Tag color="orange">≥ {config.intensity_score.threshold}</Tag>;
        }
        return '-';
      },
    },
    {
      title: '状态',
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled: boolean) => (
        <Tag color={enabled ? 'green' : 'default'}>{enabled ? '启用' : '禁用'}</Tag>
      ),
    },
    {
      title: '操作',
      key: 'action',
      render: (_: any, record: AlertRule) => (
        <Button
          type="link"
          danger
          icon={<DeleteOutlined />}
          onClick={() => handleDelete(record.id)}
        >
          删除
        </Button>
      ),
    },
  ];

  return (
    <div>
      <Card title="创建告警规则" style={{ marginBottom: 16 }}>
        <Form form={form} layout="inline">
          <Form.Item
            name="name"
            label="规则名称"
            rules={[{ required: true, message: '请输入规则名称' }]}
          >
            <Select placeholder="选择或输入规则名称" style={{ width: 200 }} showSearch>
              <Option value="高涨幅告警">高涨幅告警</Option>
              <Option value="大封单告警">大封单告警</Option>
              <Option value="强抢筹告警">强抢筹告警</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="ruleType"
            label="规则类型"
            rules={[{ required: true, message: '请选择规则类型' }]}
          >
            <Select placeholder="选择规则类型" style={{ width: 150 }}>
              <Option value="change_percent">价格涨幅</Option>
              <Option value="sealed_amount">封单金额</Option>
              <Option value="intensity_score">强度评分</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="threshold"
            label="阈值"
            rules={[{ required: true, message: '请输入阈值' }]}
          >
            <InputNumber placeholder="阈值" min={0} style={{ width: 120 }} />
          </Form.Item>

          <Form.Item name="enabled" label="启用" valuePropName="checked" initialValue={true}>
            <Switch />
          </Form.Item>

          <Form.Item>
            <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
              创建规则
            </Button>
          </Form.Item>
        </Form>
      </Card>

      <Card title="告警规则列表">
        <Table
          dataSource={rules}
          columns={columns}
          rowKey="id"
          loading={loading}
          pagination={{ pageSize: 10 }}
        />
      </Card>
    </div>
  );
}

export default AlertConfig;
