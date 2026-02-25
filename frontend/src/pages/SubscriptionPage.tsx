/**
 * 订阅管理页面
 * 显示当前订阅状态、计划选项对比和功能对比
 */

import { useState } from 'react';
import {
  Card,
  Button,
  Tag,
  Space,
  Typography,
  Row,
  Col,
  Descriptions,
  Table,
  message,
  Modal,
  Divider,
} from 'antd';
import {
  CrownOutlined,
  CheckOutlined,
  CloseOutlined,
  StarOutlined,
  RocketOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';
import type { SubscriptionPlan } from '../types/auth';

const { Title, Text, Paragraph } = Typography;

interface Plan {
  id: SubscriptionPlan;
  name: string;
  description: string;
  price: string;
  period: string;
  color: string;
  icon: React.ReactNode;
  features: string[];
  popular?: boolean;
}

interface FeatureComparison {
  feature: string;
  free: boolean;
  basic: boolean;
  premium: boolean;
  enterprise: boolean;
}

const plans: Plan[] = [
  {
    id: 'free',
    name: '免费版',
    description: '适合个人用户体验',
    price: '¥0',
    period: '永久',
    color: '#bfbfbf',
    icon: <StarOutlined />,
    features: [
      '基础市场数据查看',
      '简单筛选条件',
      '基本图表展示',
      '3个自选股',
      '社区支持',
    ],
  },
  {
    id: 'basic',
    name: '基础版',
    description: '适合个人投资者',
    price: '¥99',
    period: '月',
    color: '#1890ff',
    icon: <RocketOutlined />,
    features: [
      '包含免费版所有功能',
      '高级筛选条件',
      '数据导出功能',
      '50个自选股',
      '价格提醒',
      '邮件支持',
    ],
  },
  {
    id: 'premium',
    name: '高级版',
    description: '适合专业交易者',
    price: '¥299',
    period: '月',
    color: '#faad14',
    icon: <CrownOutlined />,
    features: [
      '包含基础版所有功能',
      '实时WebSocket数据',
      '高级技术指标',
      '竞价分析深度数据',
      '200个自选股',
      '自定义指标',
      '优先客服支持',
    ],
    popular: true,
  },
  {
    id: 'enterprise',
    name: '企业版',
    description: '适合机构和团队',
    price: '¥999',
    period: '月',
    color: '#722ed1',
    icon: <ThunderboltOutlined />,
    features: [
      '包含高级版所有功能',
      'API访问权限',
      '团队协作功能',
      '无限制自选股',
      '专属客户经理',
      '定制化需求支持',
      'SLA保障',
    ],
  },
];

const featureComparison: FeatureComparison[] = [
  { feature: '基础市场数据', free: true, basic: true, premium: true, enterprise: true },
  { feature: '高级筛选条件', free: false, basic: true, premium: true, enterprise: true },
  { feature: '数据导出', free: false, basic: true, premium: true, enterprise: true },
  { feature: '实时WebSocket数据', free: false, basic: false, premium: true, enterprise: true },
  { feature: '竞价分析深度数据', free: false, basic: false, premium: true, enterprise: true },
  { feature: '高级技术指标', free: false, basic: false, premium: true, enterprise: true },
  { feature: '自定义指标', free: false, basic: false, premium: true, enterprise: true },
  { feature: 'API访问', free: false, basic: false, premium: false, enterprise: true },
  { feature: '团队协作', free: false, basic: false, premium: false, enterprise: true },
  { feature: '自选股数量限制', free: true, basic: true, premium: true, enterprise: false },
  { feature: '邮件支持', free: false, basic: true, premium: true, enterprise: true },
  { feature: '优先客服', free: false, basic: false, premium: true, enterprise: true },
  { feature: '专属客户经理', free: false, basic: false, premium: false, enterprise: true },
];

export default function SubscriptionPage() {
  const [selectedPlan, setSelectedPlan] = useState<SubscriptionPlan | null>(null);
  const [upgradeModalVisible, setUpgradeModalVisible] = useState(false);

  const handleUpgrade = (planId: SubscriptionPlan) => {
    setSelectedPlan(planId);
    setUpgradeModalVisible(true);
  };

  const handleConfirmUpgrade = async () => {
    if (!selectedPlan) return;

    try {
      // 这里应该调用实际的API进行升级
      // await upgradeSubscription(selectedPlan);
      message.success(`升级到${plans.find(p => p.id === selectedPlan)?.name}成功！`);
      setUpgradeModalVisible(false);
      setSelectedPlan(null);
    } catch (error) {
      message.error('升级失败，请稍后重试');
    }
  };

  const featureColumns = [
    {
      title: '功能',
      dataIndex: 'feature',
      key: 'feature',
      width: 200,
      fixed: 'left' as const,
    },
    {
      title: '免费版',
      dataIndex: 'free',
      key: 'free',
      align: 'center' as const,
      width: 100,
      render: (enabled: boolean) =>
        enabled ? <CheckOutlined style={{ color: '#52c41a' }} /> : <CloseOutlined style={{ color: '#ff4d4f' }} />,
    },
    {
      title: '基础版',
      dataIndex: 'basic',
      key: 'basic',
      align: 'center' as const,
      width: 100,
      render: (enabled: boolean) =>
        enabled ? <CheckOutlined style={{ color: '#52c41a' }} /> : <CloseOutlined style={{ color: '#ff4d4f' }} />,
    },
    {
      title: '高级版',
      dataIndex: 'premium',
      key: 'premium',
      align: 'center' as const,
      width: 100,
      render: (enabled: boolean) =>
        enabled ? <CheckOutlined style={{ color: '#52c41a' }} /> : <CloseOutlined style={{ color: '#ff4d4f' }} />,
    },
    {
      title: '企业版',
      dataIndex: 'enterprise',
      key: 'enterprise',
      align: 'center' as const,
      width: 100,
      render: (enabled: boolean) =>
        enabled ? <CheckOutlined style={{ color: '#52c41a' }} /> : <CloseOutlined style={{ color: '#ff4d4f' }} />,
    },
  ];

  return (
    <div style={{ padding: '24px', maxWidth: '1400px', margin: '0 auto' }}>
      <Title level={2}>
        <CrownOutlined /> 订阅管理
      </Title>
      <Paragraph type="secondary">
        选择最适合您的订阅计划，解锁更多专业功能
      </Paragraph>

      {/* 当前订阅状态 */}
      <Card title="当前订阅" style={{ marginBottom: 24 }}>
        <Descriptions column={3} bordered>
          <Descriptions.Item label="当前计划">
            <Tag color="blue">基础版</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="状态">
            <Tag color="success">活跃</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="到期时间">2025-02-25</Descriptions.Item>
        </Descriptions>
        <div style={{ marginTop: 16, textAlign: 'right' }}>
          <Text type="secondary">
            如需取消订阅或修改计划，请联系客服 support@duanxianxia.com
          </Text>
        </div>
      </Card>

      {/* 订阅计划 */}
      <Title level={3}>选择您的计划</Title>
      <Row gutter={[16, 16]} style={{ marginBottom: 32 }}>
        {plans.map((plan) => (
          <Col xs={24} sm={12} lg={6} key={plan.id}>
            <Card
              hoverable
              style={{
                height: '100%',
                borderColor: plan.popular ? plan.color : undefined,
                borderWidth: plan.popular ? 2 : 1,
                position: 'relative',
              }}
              bodyStyle={{ padding: '24px' }}
            >
              {plan.popular && (
                <div
                  style={{
                    position: 'absolute',
                    top: -12,
                    right: 24,
                    background: plan.color,
                    color: 'white',
                    padding: '4px 12px',
                    borderRadius: '12px',
                    fontSize: '12px',
                    fontWeight: 'bold',
                  }}
                >
                  最受欢迎
                </div>
              )}

              <div style={{ textAlign: 'center', marginBottom: 16 }}>
                <div style={{ fontSize: '32px', color: plan.color, marginBottom: 8 }}>
                  {plan.icon}
                </div>
                <Title level={4} style={{ marginBottom: 4 }}>
                  {plan.name}
                </Title>
                <Text type="secondary">{plan.description}</Text>
              </div>

              <Divider />

              <div style={{ textAlign: 'center', marginBottom: 16 }}>
                <Text style={{ fontSize: '32px', fontWeight: 'bold', color: plan.color }}>
                  {plan.price}
                </Text>
                <Text type="secondary">/{plan.period}</Text>
              </div>

              <Button
                type={plan.popular ? 'primary' : 'default'}
                size="large"
                block
                style={{
                  marginBottom: 16,
                  backgroundColor: plan.popular ? plan.color : undefined,
                  borderColor: plan.color,
                }}
                onClick={() => handleUpgrade(plan.id)}
              >
                {plan.id === 'basic' ? '续费' : '立即升级'}
              </Button>

              <div>
                <Text strong>功能包含：</Text>
                <ul style={{ paddingLeft: 20, marginTop: 8 }}>
                  {plan.features.map((feature, index) => (
                    <li key={index}>
                      <Text type="secondary">{feature}</Text>
                    </li>
                  ))}
                </ul>
              </div>
            </Card>
          </Col>
        ))}
      </Row>

      {/* 功能对比表格 */}
      <Title level={3}>功能对比</Title>
      <Card>
        <Table
          dataSource={featureComparison}
          columns={featureColumns}
          pagination={false}
          rowKey="feature"
          size="middle"
          bordered
        />
      </Card>

      {/* 常见问题 */}
      <Title level={3} style={{ marginTop: 32 }}>
        常见问题
      </Title>
      <Card>
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          <div>
            <Title level={5}>如何取消订阅？</Title>
            <Text type="secondary">
              您可以在账户设置中随时取消订阅。取消后，您将继续享受当前计划直到计费周期结束。
            </Text>
          </div>
          <div>
            <Title level={5}>可以随时升级或降级吗？</Title>
            <Text type="secondary">
              可以。升级将立即生效，费用将按比例计算。降级将在当前计费周期结束后生效。
            </Text>
          </div>
          <div>
            <Title level={5}>支付方式有哪些？</Title>
            <Text type="secondary">
              我们支持支付宝、微信支付、银行卡等多种支付方式。
            </Text>
          </div>
          <div>
            <Title level={5}>企业版如何购买？</Title>
            <Text type="secondary">
              企业版需要联系我们销售团队，我们将根据您的需求提供定制化方案和报价。
              联系邮箱：enterprise@duanxianxia.com
            </Text>
          </div>
        </Space>
      </Card>

      {/* 升级确认对话框 */}
      <Modal
        title="确认升级"
        open={upgradeModalVisible}
        onOk={handleConfirmUpgrade}
        onCancel={() => {
          setUpgradeModalVisible(false);
          setSelectedPlan(null);
        }}
        okText="确认升级"
        cancelText="取消"
      >
        {selectedPlan && (
          <div>
            <Paragraph>
              您即将升级到 <Text strong>{plans.find(p => p.id === selectedPlan)?.name}</Text>
            </Paragraph>
            <Descriptions column={1} bordered size="small">
              <Descriptions.Item label="计划名称">
                {plans.find(p => p.id === selectedPlan)?.name}
              </Descriptions.Item>
              <Descriptions.Item label="价格">
                {plans.find(p => p.id === selectedPlan)?.price}/{plans.find(p => p.id === selectedPlan)?.period}
              </Descriptions.Item>
              <Descriptions.Item label="生效时间">
                立即生效
              </Descriptions.Item>
            </Descriptions>
            <Paragraph style={{ marginTop: 16, color: '#ff4d4f' }}>
              *升级后将立即扣费，请确认信息无误
            </Paragraph>
          </div>
        )}
      </Modal>
    </div>
  );
}