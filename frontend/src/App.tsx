import { Routes, Route } from 'react-router-dom';
import { Layout, Menu, Dropdown, Avatar, Space, Typography } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  DashboardOutlined,
  LineChartOutlined,
  TrophyOutlined,
  AppstoreOutlined,
  FundOutlined,
  RiseOutlined,
  LogoutOutlined,
  UserOutlined,
  CrownOutlined,
  FileTextOutlined,
  SoundOutlined,
  FireOutlined,
} from '@ant-design/icons';
import Dashboard from './pages/Dashboard';
import Login from './pages/Login';
import AuctionDashboard from './pages/AuctionDashboard';
import ScreenerPage from './pages/ScreenerPage';
import SectorsPage from './pages/SectorsPage';
import IndicatorsPage from './pages/IndicatorsPage';
import LeaderPage from './pages/LeaderPage';
import ResearchPage from './pages/ResearchPage';
import NewsVoicePage from './pages/NewsVoicePage';
import NewsHotPage from './pages/NewsHotPage';
import UserProfilePage from './pages/UserProfilePage';
import SubscriptionPage from './pages/SubscriptionPage';
import ProtectedRoute from './components/ProtectedRoute';
import { TokenRefreshProvider } from './components/TokenRefreshProvider';
import { setupTokenRefreshInterceptor } from './utils/tokenRefresh';
import { axiosInstance } from './utils/request';
import { useAuthStore } from './stores/authStore';

const { Content, Sider, Header } = Layout;
const { Text } = Typography;

// 设置Token刷新拦截器
setupTokenRefreshInterceptor(axiosInstance);

function AppContent() {
  const navigate = useNavigate();
  const location = useLocation();
  const { user, logout } = useAuthStore();

  const menuItems = [
    {
      key: '/',
      icon: <DashboardOutlined />,
      label: '实时行情',
    },
    {
      key: '/auction',
      icon: <LineChartOutlined />,
      label: '竞价分析',
      permissions: ['market:websocket:connect'], // 需要实时数据权限
    },
    {
      key: '/screener',
      icon: <TrophyOutlined />,
      label: '个股挖掘',
    },
    {
      key: '/sectors',
      icon: <AppstoreOutlined />,
      label: '概念板块',
    },
    {
      key: '/indicators',
      icon: <FundOutlined />,
      label: '技术指标',
    },
    {
      key: '/leader',
      icon: <RiseOutlined />,
      label: '龙头高度',
      permissions: ['screener:advanced:use'], // 需要高级筛选权限
    },
    {
      key: '/research',
      icon: <FileTextOutlined />,
      label: '研报检索',
    },
    {
      key: '/news-voice',
      icon: <SoundOutlined />,
      label: '语音快讯',
    },
    {
      key: '/news-hot',
      icon: <FireOutlined />,
      label: '热点聚焦',
    },
  ];

  const handleMenuClick = ({ key }: { key: string }) => {
    navigate(key);
  };

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  const userMenuItems = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: '用户中心',
      onClick: () => navigate('/profile'),
    },
    {
      key: 'subscription',
      icon: <CrownOutlined />,
      label: '订阅管理',
      onClick: () => navigate('/subscription'),
    },
    {
      type: 'divider' as const,
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: '退出登录',
      onClick: handleLogout,
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider width={200} style={{ background: '#fff' }}>
        <div style={{ padding: '16px', textAlign: 'center', fontSize: 18, fontWeight: 'bold' }}>
          短线侠
        </div>
        <Menu
          mode="inline"
          selectedKeys={[location.pathname]}
          items={menuItems}
          onClick={handleMenuClick}
          style={{ height: '100%', borderRight: 0 }}
        />
      </Sider>
      <Layout>
        <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Text type="secondary" style={{ fontSize: 14 }}>
            A股短线交易分析平台
          </Text>
          <Dropdown menu={{ items: userMenuItems }} placement="bottomRight">
            <Space style={{ cursor: 'pointer' }}>
              <Avatar icon={<UserOutlined />} />
              <Text strong>{user?.username || '用户'}</Text>
            </Space>
          </Dropdown>
        </Header>
        <Content style={{ background: '#fff', margin: 0 }}>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route
              path="/"
              element={
                <ProtectedRoute>
                  <Dashboard />
                </ProtectedRoute>
              }
            />
            <Route
              path="/auction"
              element={
                <ProtectedRoute>
                  <AuctionDashboard />
                </ProtectedRoute>
              }
            />
            <Route
              path="/screener"
              element={
                <ProtectedRoute>
                  <ScreenerPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/sectors"
              element={
                <ProtectedRoute>
                  <SectorsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/indicators"
              element={
                <ProtectedRoute>
                  <IndicatorsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/leader"
              element={
                <ProtectedRoute permission="screener:advanced:use">
                  <LeaderPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/profile"
              element={
                <ProtectedRoute>
                  <UserProfilePage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/subscription"
              element={
                <ProtectedRoute>
                  <SubscriptionPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/research"
              element={
                <ProtectedRoute>
                  <ResearchPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/news-voice"
              element={
                <ProtectedRoute>
                  <NewsVoicePage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/news-hot"
              element={
                <ProtectedRoute>
                  <NewsHotPage />
                </ProtectedRoute>
              }
            />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}

function App() {
  return (
    <TokenRefreshProvider>
      <AppContent />
    </TokenRefreshProvider>
  );
}

export default App;
