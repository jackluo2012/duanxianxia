/**
 * 高级筛选功能守卫组件
 * 用于保护需要权限的筛选器功能
 */

import { ReactNode, useState } from 'react';
import { Modal, Button, Space, Typography, Alert } from 'antd';
import { CrownOutlined, RocketOutlined } from '@ant-design/icons';
import { useAuthStore } from '../stores/authStore';

const { Title, Text, Paragraph } = Typography;

interface AdvancedFilterGuardProps {
  children: ReactNode;
  /**
   * 需要的权限代码
   */
  permission?: string;
  /**
   * 自定义无权限提示
   */
  unauthorizedContent?: ReactNode;
  /**
   * 点击时的处理模式
   * - modal: 显示升级提示弹窗
   * - inline: 显示内联提示
   * - disable: 禁用功能
   */
  mode?: 'modal' | 'inline' | 'disable';
}

/**
 * 高级筛选守卫组件
 * 检查用户是否有使用高级筛选的权限
 */
export function AdvancedFilterGuard({
  children,
  permission = 'screener:advanced:use',
  unauthorizedContent,
  mode = 'modal',
}: AdvancedFilterGuardProps) {
  const { hasPermission } = useAuthStore();
  const [upgradeModalVisible, setUpgradeModalVisible] = useState(false);

  const hasAccess = hasPermission(permission);

  const handleClick = (e: React.MouseEvent) => {
    if (!hasAccess) {
      e.preventDefault();
      e.stopPropagation();

      if (mode === 'modal') {
        setUpgradeModalVisible(true);
      }
    }
  };

  const renderUnauthorizedContent = () => {
    if (unauthorizedContent) {
      return <div onClick={handleClick}>{unauthorizedContent}</div>;
    }

    if (mode === 'inline') {
      return (
        <Alert
          message="高级筛选功能"
          description={
            <Space direction="vertical">
              <Text>您正在使用高级筛选功能，需要高级版或企业版权限。</Text>
              <Button
                type="primary"
                icon={<RocketOutlined />}
                onClick={() => (window.location.href = '/subscription')}
              >
                立即升级
              </Button>
            </Space>
          }
          type="info"
          showIcon
          icon={<CrownOutlined />}
          style={{ margin: '16px 0' }}
        />
      );
    }

    return null;
  };

  // 如果有权限，正常显示子组件
  if (hasAccess) {
    return <>{children}</>;
  }

  // 无权限时的处理
  if (mode === 'disable') {
    return (
      <div
        onClick={handleClick}
        style={{ cursor: 'not-allowed', opacity: 0.6, position: 'relative' }}
      >
        {children}
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(255, 255, 255, 0.7)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1,
          }}
        >
          <Text type="secondary" style={{ background: 'white', padding: '8px 16px', borderRadius: 4 }}>
            <CrownOutlined /> 需要高级版权限
          </Text>
        </div>
      </div>
    );
  }

  return (
    <>
      {renderUnauthorizedContent()}

      {mode === 'modal' && (
        <Modal
          title={
            <Space>
              <CrownOutlined style={{ color: '#faad14' }} />
              <span>升级到高级版</span>
            </Space>
          }
          open={upgradeModalVisible}
          onCancel={() => setUpgradeModalVisible(false)}
          footer={[
            <Button key="cancel" onClick={() => setUpgradeModalVisible(false)}>
              取消
            </Button>,
            <Button
              key="upgrade"
              type="primary"
              icon={<RocketOutlined />}
              onClick={() => {
                setUpgradeModalVisible(false);
                window.location.href = '/subscription';
              }}
            >
              立即升级
            </Button>,
          ]}
          width={480}
        >
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            <div>
              <Title level={5}>解锁高级筛选功能</Title>
              <Paragraph type="secondary">
                高级筛选功能提供了更强大的筛选条件，帮助您精准找到目标股票。
              </Paragraph>
            </div>

            <div>
              <Title level={5}>高级版功能包括：</Title>
              <ul style={{ paddingLeft: 20 }}>
                <li>
                  <Space>
                    <CrownOutlined style={{ color: '#faad14' }} />
                    <Text>高级筛选条件</Text>
                  </Space>
                </li>
                <li>
                  <Space>
                    <CrownOutlined style={{ color: '#faad14' }} />
                    <Text>实时WebSocket数据</Text>
                  </Space>
                </li>
                <li>
                  <Space>
                    <CrownOutlined style={{ color: '#faad14' }} />
                    <Text>高级技术指标</Text>
                  </Space>
                </li>
                <li>
                  <Space>
                    <CrownOutlined style={{ color: '#faad14' }} />
                    <Text>竞价分析深度数据</Text>
                  </Space>
                </li>
                <li>
                  <Space>
                    <CrownOutlined style={{ color: '#faad14' }} />
                    <Text>数据导出功能</Text>
                  </Space>
                </li>
              </ul>
            </div>

            <div>
              <Title level={5}>选择您的计划：</Title>
              <Space direction="vertical" style={{ width: '100%' }}>
                <div
                  style={{
                    border: '1px solid #faad14',
                    borderRadius: 8,
                    padding: 16,
                    background: '#fffbe6',
                  }}
                >
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Text strong>高级版</Text>
                      <Text strong style={{ color: '#faad14' }}>
                        ¥299/月
                      </Text>
                    </div>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      适合专业交易者，包含所有高级功能
                    </Text>
                  </Space>
                </div>

                <div
                  style={{
                    border: '1px solid #722ed1',
                    borderRadius: 8,
                    padding: 16,
                    background: '#f9f0ff',
                  }}
                >
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Text strong>企业版</Text>
                      <Text strong style={{ color: '#722ed1' }}>
                        ¥999/月
                      </Text>
                    </div>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      适合机构和团队，包含API访问和团队协作
                    </Text>
                  </Space>
                </div>
              </Space>
            </div>
          </Space>
        </Modal>
      )}
    </>
  );
}

/**
 * 高级筛选按钮守卫
 * 专门用于按钮的高级筛选功能保护
 */
export function AdvancedFilterButtonGuard({
  children,
  permission = 'screener:advanced:use',
}: {
  children: ReactNode;
  permission?: string;
}) {
  return (
    <AdvancedFilterGuard permission={permission} mode="modal">
      {children}
    </AdvancedFilterGuard>
  );
}

export default AdvancedFilterGuard;