/**
 * 增强的登录页面
 * 支持登录、记住密码、错误提示
 */

import { Form, Input, Button, Card, Checkbox, message, Typography, Space } from 'antd';
import { UserOutlined, LockOutlined, RocketOutlined } from '@ant-design/icons';
import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '../stores/authStore';

const { Title, Text } = Typography;

interface LoginForm {
  username: string;
  password: string;
  remember?: boolean;
}

function Login() {
  const navigate = useNavigate();
  const location = useLocation();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  const { login, isAuthenticated } = useAuthStore();

  // 如果已登录，跳转到首页
  useEffect(() => {
    if (isAuthenticated) {
      const from = (location.state as any)?.from?.pathname || '/';
      navigate(from, { replace: true });
    }
  }, [isAuthenticated, navigate, location]);

  const onFinish = async (values: LoginForm) => {
    setLoading(true);
    try {
      await login(values.username, values.password);

      message.success('登录成功！');

      // 记住密码
      if (values.remember) {
        localStorage.setItem('remembered_username', values.username);
      } else {
        localStorage.removeItem('remembered_username');
      }

      // 跳转到来源页面或首页
      const from = (location.state as any)?.from?.pathname || '/';
      navigate(from, { replace: true });
    } catch (error: any) {
      console.error('[Login] Login error:', error);
      const errorMessage = error?.response?.data?.message || error?.message || '登录失败，请检查用户名和密码';
      message.error(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  // 加载记住的用户名
  useEffect(() => {
    const rememberedUsername = localStorage.getItem('remembered_username');
    if (rememberedUsername) {
      form.setFieldsValue({ username: rememberedUsername, remember: true });
    }
  }, [form]);

  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: '100vh',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        padding: 24,
      }}
    >
      <Card
        style={{
          width: 420,
          borderRadius: 16,
          boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
        }}
        styles={{
          body: {
            padding: '40px 48px 32px',
          }
        }}
      >
        {/* Logo和标题 */}
        <div style={{ textAlign: 'center', marginBottom: 32 }}>
          <div
            style={{
              fontSize: 64,
              marginBottom: 16,
              animation: 'float 3s ease-in-out infinite',
            }}
          >
            🚀
          </div>
          <Title level={2} style={{ marginBottom: 8, margin: 0 }}>
            短线侠
          </Title>
          <Text type="secondary" style={{ fontSize: 14 }}>
            A股短线交易分析平台
          </Text>
        </div>

        {/* 登录表单 */}
        <Form
          form={form}
          name="login"
          onFinish={onFinish}
          initialValues={{ remember: true }}
          size="large"
        >
          <Form.Item
            name="username"
            rules={[
              { required: true, message: '请输入用户名' },
              { min: 3, message: '用户名至少3个字符' },
            ]}
          >
            <Input
              prefix={<UserOutlined style={{ color: 'rgba(0,0,0,.25)' }} />}
              placeholder="用户名"
              autoComplete="username"
            />
          </Form.Item>

          <Form.Item
            name="password"
            rules={[
              { required: true, message: '请输入密码' },
              { min: 6, message: '密码至少6个字符' },
            ]}
          >
            <Input.Password
              prefix={<LockOutlined style={{ color: 'rgba(0,0,0,.25)' }} />}
              placeholder="密码"
              autoComplete="current-password"
            />
          </Form.Item>

          <Form.Item>
            <Form.Item name="remember" valuePropName="checked" noStyle>
              <Checkbox>记住用户名</Checkbox>
            </Form.Item>
          </Form.Item>

          <Form.Item style={{ marginBottom: 16 }}>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              block
              size="large"
              icon={<RocketOutlined />}
              style={{
                height: 44,
                fontSize: 16,
                fontWeight: 'bold',
              }}
            >
              登录
            </Button>
          </Form.Item>
        </Form>

        {/* 底部信息 */}
        <div style={{ textAlign: 'center', marginTop: 24 }}>
          <Space direction="vertical" size={4}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              还没有账号？ <a href="#" style={{ color: '#1890ff' }}>立即注册</a>
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              忘记密码？ <a href="#" style={{ color: '#1890ff' }}>找回密码</a>
            </Text>
          </Space>
        </div>

        {/* 版本信息 */}
        <div style={{ textAlign: 'center', marginTop: 32, paddingTop: 16, borderTop: '1px solid #f0f0f0' }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            v1.0.0
          </Text>
        </div>
      </Card>

      {/* 浮动动画 */}
      <style>{`
        @keyframes float {
          0%, 100% { transform: translateY(0px); }
          50% { transform: translateY(-10px); }
        }
      `}</style>
    </div>
  );
}

export default Login;
