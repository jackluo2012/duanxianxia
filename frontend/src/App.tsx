import { Routes, Route } from 'react-router-dom';
import { Layout, Menu } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  DashboardOutlined,
  LineChartOutlined,
  TrophyOutlined,
  AppstoreOutlined,
  FundOutlined,
} from '@ant-design/icons';
import Dashboard from './pages/Dashboard';
import Login from './pages/Login';
import AuctionDashboard from './pages/AuctionDashboard';
import ScreenerPage from './pages/ScreenerPage';
import SectorsPage from './pages/SectorsPage';
import IndicatorsPage from './pages/IndicatorsPage';

const { Content, Sider } = Layout;

function App() {
  const navigate = useNavigate();
  const location = useLocation();

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
  ];

  const handleMenuClick = ({ key }: { key: string }) => {
    navigate(key);
  };

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
        <Content style={{ background: '#fff', margin: 0 }}>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route path="/" element={<Dashboard />} />
            <Route path="/auction" element={<AuctionDashboard />} />
            <Route path="/screener" element={<ScreenerPage />} />
            <Route path="/sectors" element={<SectorsPage />} />
            <Route path="/indicators" element={<IndicatorsPage />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}

export default App;
