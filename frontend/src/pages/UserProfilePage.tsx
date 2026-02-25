/**
 * 用户中心页面
 * 显示用户信息、角色、权限和订阅状态
 */

import { useEffect, useState } from 'react';
import {
  Card,
  Descriptions,
  Table,
  Tag,
  Space,
  Avatar,
  Typography,
  Divider,
  message,
  Spin,
  Button,
} from 'antd';
import {
  UserOutlined,
  SafetyOutlined,
  CrownOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { useAuthStore } from '../stores/authStore';
import type { Permission } from '../types/auth';

const { Title, Text } = Typography;

interface SubscriptionInfo {
  plan: string;
  status: string;
  startDate: string;
  endDate?: string;
  features: string[];
}

export default function UserProfilePage() {
  const { user, roles, permissions, refreshPermissions } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [subscription, setSubscription] = useState<SubscriptionInfo | null>(null);

  useEffect(() => {
    fetchUserInfo();
  }, []);

  const fetchUserInfo = async () => {
    setLoading(true);
    try {
      // 这里应该调用实际的API获取用户详细信息
      // 暂时使用模拟数据
      setSubscription({
        plan: 'basic',
        status: 'active',
        startDate: '2024-01-01',
        endDate: '2025-01-01',
        features: ['基础筛选', '基本图表', '数据导出'],
      });
    } catch (error) {
      message.error('获取用户信息失败');
    } finally {
      setLoading(false);
    }
  };

  const handleRefreshPermissions = async () => {
    try {
      await refreshPermissions();
      message.success('权限信息已刷新');
    } catch (error) {
      message.error('刷新权限失败');
    }
  };

  const getSubscriptionColor = (plan: string) => {
    const colors: Record<string, string> = {
      free: 'default',
      basic: 'blue',
      premium: 'gold',
      enterprise: 'purple',
    };
    return colors[plan] || 'default';
  };

  const getSubscriptionLabel = (plan: string) => {
    const labels: Record<string, string> = {
      free: '免费版',
      basic: '基础版',
      premium: '高级版',
      enterprise: '企业版',
    };
    return labels[plan] || plan;
  };

  // 权限表格列定义
  const permissionColumns = [
    {
      title: '权限名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
    },
    {
      title: '权限代码',
      dataIndex: 'code',
      key: 'code',
      width: 250,
      render: (code: string) => <Text code>{code}</Text>,
    },
    {
      title: '模块',
      dataIndex: 'module',
      key: 'module',
      width: 120,
      render: (module: string) => (module ? <Tag color="blue">{module}</Tag> : '-'),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
    },
  ];

  // 角色表格列定义
  const roleColumns = [
    {
      title: '角色名称',
      dataIndex: 'name',
      key: 'name',
      width: 150,
    },
    {
      title: '角色代码',
      dataIndex: 'code',
      key: 'code',
      width: 150,
      render: (code: string) => <Text code>{code}</Text>,
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
    },
    {
      title: '权限数量',
      dataIndex: 'permissions',
      key: 'permissions',
      width: 100,
      render: (perms: Permission[]) => perms?.length || 0,
    },
    {
      title: '系统角色',
      dataIndex: 'isSystem',
      key: 'isSystem',
      width: 100,
      render: (isSystem: boolean) =>
        isSystem ? <Tag color="red">系统角色</Tag> : <Tag>自定义</Tag>,
    },
  ];

  if (loading) {
    return (
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh',
        }}
      >
        <Spin size="large" tip="加载中..." />
      </div>
    );
  }

  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <Title level={2}>
        <UserOutlined /> 用户中心
      </Title>

      {/* 用户基本信息 */}
      <Card
        title={
          <Space>
            <Avatar size={32} icon={<UserOutlined />} />
            <span>基本信息</span>
          </Space>
        }
        style={{ marginBottom: 16 }}
      >
        <Descriptions column={2} bordered>
          <Descriptions.Item label="用户名">{user?.username}</Descriptions.Item>
          <Descriptions.Item label="昵称">{user?.nickname || '-'}</Descriptions.Item>
          <Descriptions.Item label="邮箱">{user?.email || '-'}</Descriptions.Item>
          <Descriptions.Item label="用户ID">{user?.id}</Descriptions.Item>
          <Descriptions.Item label="注册时间">
            {user?.createdAt ? new Date(user.createdAt).toLocaleDateString('zh-CN') : '-'}
          </Descriptions.Item>
          <Descriptions.Item label="账户角色">
            <Space>
              {roles?.map((role) => (
                <Tag key={role.id} color="blue">
                  {role.name}
                </Tag>
              ))}
              {roles?.length === 0 && <Tag>暂无角色</Tag>}
            </Space>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* 订阅信息 */}
      <Card
        title={
          <Space>
            <CrownOutlined />
            <span>订阅信息</span>
          </Space>
        }
        style={{ marginBottom: 16 }}
        extra={
          <a href="/subscription" onClick={(e) => e.preventDefault()}>
            查看详情
          </a>
        }
      >
        {subscription ? (
          <Descriptions column={2} bordered>
            <Descriptions.Item label="当前计划">
              <Tag color={getSubscriptionColor(subscription.plan)}>
                {getSubscriptionLabel(subscription.plan)}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="状态">
              <Tag
                color={
                  subscription.status === 'active'
                    ? 'success'
                    : subscription.status === 'expired'
                    ? 'error'
                    : 'default'
                }
              >
                {subscription.status === 'active'
                  ? '活跃'
                  : subscription.status === 'expired'
                  ? '已过期'
                  : subscription.status}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="开始日期">
              {new Date(subscription.startDate).toLocaleDateString('zh-CN')}
            </Descriptions.Item>
            <Descriptions.Item label="结束日期">
              {subscription.endDate
                ? new Date(subscription.endDate).toLocaleDateString('zh-CN')
                : '永久有效'}
            </Descriptions.Item>
            <Descriptions.Item label="包含功能" span={2}>
              <Space wrap>
                {subscription.features.map((feature, index) => (
                  <Tag key={index} color="green">
                    {feature}
                  </Tag>
                ))}
              </Space>
            </Descriptions.Item>
          </Descriptions>
        ) : (
          <Text type="secondary">暂无订阅信息</Text>
        )}
      </Card>

      {/* 角色信息 */}
      <Card
        title={
          <Space>
            <SafetyOutlined />
            <span>角色信息</span>
            <Tag color="blue">{roles?.length || 0} 个角色</Tag>
          </Space>
        }
        style={{ marginBottom: 16 }}
      >
        {roles && roles.length > 0 ? (
          <Table
            dataSource={roles}
            columns={roleColumns}
            rowKey="id"
            pagination={false}
            size="small"
          />
        ) : (
          <Text type="secondary">暂无角色</Text>
        )}
      </Card>

      {/* 权限信息 */}
      <Card
        title={
          <Space>
            <SafetyOutlined />
            <span>权限列表</span>
            <Tag color="green">{permissions?.length || 0} 个权限</Tag>
            <Button
              type="link"
              size="small"
              icon={<ReloadOutlined />}
              onClick={handleRefreshPermissions}
            >
              刷新权限
            </Button>
          </Space>
        }
      >
        {permissions && permissions.length > 0 ? (
          <Table
            dataSource={permissions}
            columns={permissionColumns}
            rowKey="id"
            pagination={{
              pageSize: 10,
              showSizeChanger: true,
              showTotal: (total) => `共 ${total} 个权限`,
            }}
            size="small"
          />
        ) : (
          <Text type="secondary">暂无权限</Text>
        )}
      </Card>

      <Divider />
      <div style={{ textAlign: 'center', color: '#999' }}>
        <Text type="secondary">
          如需升级订阅或获取更多权限，请访问{' '}
          <a href="/subscription">订阅管理页面</a>
        </Text>
      </div>
    </div>
  );
}