import { List, Tag, Typography, Space, Button, Tooltip } from 'antd';
import { ReloadOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { useAuctionRanking } from '../../hooks/useAuctionRanking';
import { AuctionStock } from '../../api/auction';

const { Text } = Typography;

interface AuctionRankingListProps {
  rankingType: string;
  onStockSelect: (stock: AuctionStock) => void;
  selectedCode?: string;
}

function AuctionRankingList({
  rankingType,
  onStockSelect,
  selectedCode,
}: AuctionRankingListProps) {
  const { data, loading, error, lastUpdate, refresh } = useAuctionRanking({
    rankingType,
    limit: 50,
    autoRefresh: true,
    refreshInterval: 5000,
  });

  const formatAmount = (amount: number) => {
    if (amount >= 100000000) {
      return `${(amount / 100000000).toFixed(2)}亿`;
    } else if (amount >= 10000) {
      return `${(amount / 10000).toFixed(2)}万`;
    }
    return amount.toFixed(2);
  };

  const getRankingTitle = () => {
    switch (rankingType) {
      case 'buy_sealed':
        return '买封金额排行';
      case 'intensity':
        return '抢筹强度排行';
      case 'change':
        return '涨幅排行';
      case 'anomaly':
        return '异动检测排行';
      default:
        return '排行榜';
    }
  };

  const getRankingDescription = () => {
    switch (rankingType) {
      case 'buy_sealed':
        return '按涨停板买封金额排序，反映资金抢筹意愿';
      case 'intensity':
        return '综合封单金额、涨幅、委比计算的强度评分';
      case 'change':
        return '按竞价涨幅排序';
      case 'anomaly':
        return '检测异常波动和异动股票';
      default:
        return '';
    }
  };

  const renderRankingValue = (item: AuctionStock, index: number) => {
    switch (rankingType) {
      case 'buy_sealed':
        return (
          <Space>
            <Text type="secondary">买封:</Text>
            <Text strong style={{ color: '#cf1322', fontSize: 15 }}>
              {formatAmount(item.sealed_amount_buy)}
            </Text>
            {index === 0 && (
              <Tag color="red">最强</Tag>
            )}
          </Space>
        );
      case 'intensity':
        return (
          <Space>
            <Text type="secondary">强度:</Text>
            <Text strong style={{ color: '#faad14', fontSize: 15 }}>
              {item.intensity_score?.toFixed(1)}
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              /100
            </Text>
            {index === 0 && (
              <Tag color="orange">抢筹</Tag>
            )}
          </Space>
        );
      case 'change':
        return (
          <Space>
            <Tag
              color={item.change_percent >= 0 ? 'red' : 'green'}
              style={{ fontSize: 14, padding: '2px 8px' }}
            >
              {item.change_percent >= 0 ? '+' : ''}
              {item.change_percent.toFixed(2)}%
            </Tag>
            {item.change_percent > 5 && (
              <Tag color="red">强势</Tag>
            )}
          </Space>
        );
      case 'anomaly':
        return (
          <Space>
            <Tag color="purple" icon={<InfoCircleOutlined />}>
              异动
            </Tag>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {item.change_percent > 3 ? '拉升' : '震荡'}
            </Text>
          </Space>
        );
      default:
        return null;
    }
  };

  return (
    <div>
      {/* 标题和操作栏 */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 16,
        padding: '8px 12px',
        background: '#fafafa',
        borderRadius: 4,
      }}>
        <Space direction="vertical" size={0}>
          <Text strong style={{ fontSize: 14 }}>
            {getRankingTitle()}
          </Text>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {getRankingDescription()}
          </Text>
        </Space>
        <Space>
          {lastUpdate && (
            <Text type="secondary" style={{ fontSize: 12 }}>
              更新: {lastUpdate.toLocaleTimeString()}
            </Text>
          )}
          <Button
            icon={<ReloadOutlined spin={loading} />}
            onClick={refresh}
            size="small"
            type="primary"
            ghost
          >
            刷新
          </Button>
        </Space>
      </div>

      {/* 错误提示 */}
      {error && (
        <div style={{
          padding: '8px 12px',
          background: '#fff2f0',
          border: '1px solid #ffccc7',
          borderRadius: 4,
          marginBottom: 12,
          color: '#cf1322',
          fontSize: 12,
        }}>
          ⚠️ {error}
        </div>
      )}

      {/* 排行榜列表 */}
      <List
        loading={loading}
        dataSource={data}
        renderItem={(item, index) => (
          <List.Item
            key={item.code}
            style={{
              cursor: 'pointer',
              backgroundColor: selectedCode === item.code ? '#e6f7ff' : 'transparent',
              padding: '12px 16px',
              borderRadius: 4,
              marginBottom: 8,
              border: selectedCode === item.code ? '1px solid #1890ff' : '1px solid transparent',
              transition: 'all 0.2s',
            }}
            onClick={() => onStockSelect(item)}
            onMouseEnter={(e) => {
              if (selectedCode !== item.code) {
                e.currentTarget.style.backgroundColor = '#fafafa';
                e.currentTarget.style.transform = 'translateX(4px)';
              }
            }}
            onMouseLeave={(e) => {
              if (selectedCode !== item.code) {
                e.currentTarget.style.backgroundColor = 'transparent';
                e.currentTarget.style.transform = 'translateX(0)';
              }
            }}
          >
            <List.Item.Meta
              avatar={
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: '50%',
                    background: index < 3
                      ? index === 0 ? 'linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%)'
                      : index === 1 ? 'linear-gradient(135deg, #faad14 0%, #ffc53d 100%)'
                      : 'linear-gradient(135deg, #52c41a 0%, #73d13d 100%)'
                      : '#d9d9d9',
                    color: index < 3 ? 'white' : '#666',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontWeight: 'bold',
                    fontSize: 16,
                    boxShadow: index < 3 ? '0 2px 8px rgba(0,0,0,0.15)' : 'none',
                  }}
                >
                  {index + 1}
                </div>
              }
              title={
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <Text strong style={{ fontSize: 15 }}>
                      {item.name}
                    </Text>
                    <Text type="secondary" style={{ marginLeft: 8, fontSize: 13 }}>
                      {item.code}
                    </Text>
                  </div>
                  <Space>
                    <Text strong style={{ fontSize: 18, color: item.change_percent >= 0 ? '#cf1322' : '#3f8600' }}>
                      ¥{item.price.toFixed(2)}
                    </Text>
                    <Tag
                      color={item.change_percent >= 0 ? 'red' : 'green'}
                      style={{ margin: 0 }}
                    >
                      {item.change_percent >= 0 ? '+' : ''}
                      {item.change_percent.toFixed(2)}%
                    </Tag>
                  </Space>
                </div>
              }
              description={
                <div style={{ marginTop: 4 }}>
                  {renderRankingValue(item, index)}
                </div>
              }
            />
          </List.Item>
        )}
        locale={{
          emptyText: loading ? null : (
            <div style={{ padding: '40px 0', textAlign: 'center' }}>
              <div style={{ fontSize: 48, marginBottom: 16 }}>📊</div>
              <div style={{ color: '#999', fontSize: 14 }}>
                暂无竞价数据
              </div>
              <div style={{ color: '#bbb', fontSize: 12, marginTop: 8 }}>
                竞价时段: 9:15-9:25
              </div>
            </div>
          )
        }}
      />
    </div>
  );
}

export default AuctionRankingList;
