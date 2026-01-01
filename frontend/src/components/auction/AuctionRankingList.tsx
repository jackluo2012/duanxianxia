import { Empty, List, Tag, Typography } from 'antd';
import { useEffect, useState } from 'react';

const { Text } = Typography;

interface AuctionStock {
  code: string;
  name: string;
  price: number;
  change_percent: number;
  sealed_amount_buy: number;
  sealed_amount_sell: number;
  intensity_score?: number;
}

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
  const [data, setData] = useState<AuctionStock[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const fetchRankings = async () => {
      setLoading(true);
      try {
        // TODO: 实际调用后端 API
        // const response = await fetch(`http://localhost:8084/api/auction/rankings?type=${rankingType}&limit=50`);
        // const result = await response.json();
        // setData(result.data);

        // 模拟数据
        const mockData: AuctionStock[] = [
          {
            code: '600519',
            name: '贵州茅台',
            price: 1850.0,
            change_percent: 1.65,
            sealed_amount_buy: 55530000.0,
            sealed_amount_sell: 37040000.0,
            intensity_score: 85.2,
          },
          {
            code: '000001',
            name: '平安银行',
            price: 12.5,
            change_percent: 1.63,
            sealed_amount_buy: 625500.0,
            sealed_amount_sell: 375600.0,
            intensity_score: 75.5,
          },
          {
            code: '000002',
            name: '万科A',
            price: 8.45,
            change_percent: 1.32,
            sealed_amount_buy: 456000.0,
            sealed_amount_sell: 234000.0,
            intensity_score: 68.8,
          },
        ];
        setData(mockData);
      } catch (error) {
        console.error('Failed to fetch rankings:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchRankings();

    // 定时刷新 (每5秒)
    const interval = setInterval(fetchRankings, 5000);
    return () => clearInterval(interval);
  }, [rankingType]);

  const formatAmount = (amount: number) => {
    if (amount >= 100000000) {
      return `${(amount / 100000000).toFixed(2)}亿`;
    } else if (amount >= 10000) {
      return `${(amount / 10000).toFixed(2)}万`;
    }
    return amount.toFixed(2);
  };

  const renderRankingValue = (item: AuctionStock) => {
    switch (rankingType) {
      case 'buy_sealed':
        return (
          <Text type="secondary">
            买封:
            <Text strong style={{ marginLeft: 4, color: '#cf1322' }}>
              {formatAmount(item.sealed_amount_buy)}
            </Text>
          </Text>
        );
      case 'intensity':
        return (
          <Text type="secondary">
            强度:
            <Text strong style={{ marginLeft: 4, color: '#faad14' }}>
              {item.intensity_score?.toFixed(1)}
            </Text>
          </Text>
        );
      case 'change':
        return (
          <Tag color={item.change_percent >= 0 ? 'red' : 'green'}>
            {item.change_percent >= 0 ? '+' : ''}
            {item.change_percent.toFixed(2)}%
          </Tag>
        );
      case 'anomaly':
        return (
          <Text type="secondary">
            异动:
            <Tag color="purple" style={{ marginLeft: 4 }}>
              关注
            </Tag>
          </Text>
        );
      default:
        return null;
    }
  };

  return (
    <List
      loading={loading}
      dataSource={data}
      renderItem={(item, index) => (
        <List.Item
          key={item.code}
          style={{
            cursor: 'pointer',
            backgroundColor: selectedCode === item.code ? '#f0f0f0' : 'transparent',
            padding: '12px 16px',
            borderRadius: 4,
            transition: 'background-color 0.2s',
          }}
          onClick={() => onStockSelect(item)}
          onMouseEnter={(e) => {
            if (selectedCode !== item.code) {
              e.currentTarget.style.backgroundColor = '#fafafa';
            }
          }}
          onMouseLeave={(e) => {
            if (selectedCode !== item.code) {
              e.currentTarget.style.backgroundColor = 'transparent';
            }
          }}
        >
          <List.Item.Meta
            avatar={
              <div
              style={{
                width: 32,
                height: 32,
                borderRadius: '50%',
                backgroundColor: index < 3 ? '#ff4d4f' : '#d9d9d9',
                color: 'white',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontWeight: 'bold',
                fontSize: 14,
              }}
            >
              {index + 1}
            </div>
            }
            title={
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <Text strong>{item.name}</Text>
                  <Text type="secondary" style={{ marginLeft: 8 }}>
                    {item.code}
                  </Text>
                </div>
                <Text strong style={{ fontSize: 16 }}>
                  ¥{item.price.toFixed(2)}
                </Text>
              </div>
            }
            description={renderRankingValue(item)}
          />
        </List.Item>
      )}
      locale={{ emptyText: <Empty description="暂无竞价数据" /> }}
    />
  );
}

export default AuctionRankingList;
