import { useState, useEffect } from 'react';
import {
  Typography,
  Card,
  Row,
  Col,
  Tag,
  Space,
  Button,
  Input,
  Select,
  Pagination,
  message,
  Spin,
  Empty,
  Drawer,
  Divider,
} from 'antd';
import {
  FireOutlined,
  EyeOutlined,
  LikeOutlined,
  CommentOutlined,
  SearchOutlined,
  FilterOutlined,
  TagOutlined,
} from '@ant-design/icons';
import dayjs from 'dayjs';
import { getHotNewsList, getHotNewsDetail, likeHotNews, getNewsStatistics } from '../api/news';
import { mockNewsStatistics } from '../mocks/news';
import type { HotNews, HotNewsQuery } from '../types/news';
import { NewsCategory, HotLevel } from '../types/news';
import HotNewsCard from '../components/news/HotNewsCard';

const { Title, Text, Paragraph } = Typography;
const { Search } = Input;
const { Option } = Select;

// 热度等级颜色映射
const hotLevelColors: Record<HotLevel, string> = {
  [HotLevel.High]: '#ff4d4f',
  [HotLevel.Medium]: '#faad14',
  [HotLevel.Low]: '#52c41a',
};

// 热度等级文本映射
const hotLevelTexts: Record<HotLevel, string> = {
  [HotLevel.High]: '高热度',
  [HotLevel.Medium]: '中热度',
  [HotLevel.Low]: '低热度',
};

// 分类颜色映射
const categoryColors: Record<NewsCategory, string> = {
  [NewsCategory.Policy]: 'red',
  [NewsCategory.Market]: 'blue',
  [NewsCategory.Company]: 'green',
  [NewsCategory.Sector]: 'orange',
  [NewsCategory.International]: 'purple',
  [NewsCategory.Technology]: 'cyan',
};

// 分类文本映射
const categoryTexts: Record<NewsCategory, string> = {
  [NewsCategory.Policy]: '政策',
  [NewsCategory.Market]: '市场',
  [NewsCategory.Company]: '公司',
  [NewsCategory.Sector]: '行业',
  [NewsCategory.International]: '国际',
  [NewsCategory.Technology]: '科技',
};

function NewsHotPage() {
  const [newsList, setNewsList] = useState<HotNews[]>([]);
  const [statistics, setStatistics] = useState(mockNewsStatistics);
  const [loading, setLoading] = useState(false);
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 12,
    total: 0,
  });

  // 筛选条件
  const [filters, setFilters] = useState<{
    keyword?: string;
    category?: NewsCategory;
    hot_level?: HotLevel;
    tag?: string;
  }>({});

  // 详情相关
  const [selectedNews, setSelectedNews] = useState<HotNews | null>(null);
  const [detailVisible, setDetailVisible] = useState(false);
  const [liked, setLiked] = useState<Set<string>>(new Set());

  // 获取统计数据
  useEffect(() => {
    const fetchStatistics = async () => {
      try {
        const data = await getNewsStatistics();
        setStatistics(data);
      } catch (error) {
        console.error('获取统计数据失败:', error);
      }
    };
    fetchStatistics();
  }, []);

  // 获取新闻列表
  const fetchNewsList = async (page = pagination.current, pageSize = pagination.pageSize) => {
    setLoading(true);
    try {
      const query: HotNewsQuery = {
        page,
        page_size: pageSize,
        sort_by: 'publish_time',
        sort_order: 'desc',
      };

      // 添加筛选条件
      if (filters.keyword) query.keyword = filters.keyword;
      if (filters.category) query.category = filters.category;
      if (filters.hot_level) query.hot_level = filters.hot_level;
      if (filters.tag) query.tag = filters.tag;

      const response = await getHotNewsList(query);
      setNewsList(response.items);
      setPagination({
        current: response.page,
        pageSize: response.page_size,
        total: response.total,
      });
    } catch (error) {
      message.error('获取热点新闻失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  // 初始加载
  useEffect(() => {
    fetchNewsList();
  }, [filters]);

  // 搜索
  const handleSearch = (keyword: string) => {
    setFilters({ ...filters, keyword });
    fetchNewsList(1, pagination.pageSize);
  };

  // 筛选变化
  const handleFilterChange = (key: string, value: any) => {
    setFilters({ ...filters, [key]: value });
    fetchNewsList(1, pagination.pageSize);
  };

  // 重置筛选
  const handleReset = () => {
    setFilters({});
    fetchNewsList(1, pagination.pageSize);
  };

  // 查看详情
  const handleViewDetail = async (news: HotNews) => {
    try {
      const detail = await getHotNewsDetail(news.id);
      setSelectedNews(detail);
      setDetailVisible(true);
    } catch (error) {
      message.error('获取新闻详情失败');
      console.error(error);
    }
  };

  // 点赞
  const handleLike = async (news: HotNews) => {
    try {
      if (liked.has(news.id)) {
        await likeHotNews(news.id); // 取消点赞
        setLiked(new Set([...liked].filter((id) => id !== news.id)));
        message.success('已取消点赞');
      } else {
        await likeHotNews(news.id);
        setLiked(new Set([...liked, news.id]));
        message.success('点赞成功');
      }
      // 刷新列表
      fetchNewsList();
    } catch (error) {
      message.error('操作失败');
      console.error(error);
    }
  };

  // 分页变化
  const handlePageChange = (page: number, pageSize: number) => {
    fetchNewsList(page, pageSize);
  };

  // 热门标签（从统计数据中获取）
  const hotTags = statistics.hot_tags || [];

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>热点聚焦</Title>

      {/* 统计卡片 */}
      <Row gutter={16} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div>
                <Text type="secondary">今日热点</Text>
                <div style={{ fontSize: 24, fontWeight: 'bold', color: '#ff4d4f' }}>
                  {statistics.today_hot_count}
                </div>
              </div>
              <FireOutlined style={{ fontSize: 32, color: '#ff4d4f' }} />
            </div>
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <div>
              <Text type="secondary">总热点数</Text>
              <div style={{ fontSize: 24, fontWeight: 'bold' }}>
                {statistics.total_hot_count}
              </div>
            </div>
          </Card>
        </Col>
        <Col span={12}>
          <Card>
            <div>
              <Text type="secondary">热门标签</Text>
              <div style={{ marginTop: 8 }}>
                <Space wrap>
                  {hotTags.slice(0, 6).map((tag) => (
                    <Tag
                      key={tag}
                      color={filters.tag === tag ? 'red' : 'default'}
                      style={{ cursor: 'pointer', marginBottom: 4 }}
                      onClick={() => handleFilterChange('tag', filters.tag === tag ? undefined : tag)}
                    >
                      {tag}
                    </Tag>
                  ))}
                </Space>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      {/* 筛选栏 */}
      <Card style={{ marginBottom: 16 }}>
        <Space wrap>
          <Search
            placeholder="搜索新闻标题或内容"
            allowClear
            enterButton={<SearchOutlined />}
            style={{ width: 300 }}
            onSearch={handleSearch}
          />
          <Select
            placeholder="选择分类"
            value={filters.category}
            onChange={(value) => handleFilterChange('category', value)}
            style={{ width: 120 }}
            allowClear
          >
            <Option value={NewsCategory.Policy}>政策</Option>
            <Option value={NewsCategory.Market}>市场</Option>
            <Option value={NewsCategory.Company}>公司</Option>
            <Option value={NewsCategory.Sector}>行业</Option>
            <Option value={NewsCategory.International}>国际</Option>
            <Option value={NewsCategory.Technology}>科技</Option>
          </Select>
          <Select
            placeholder="热度等级"
            value={filters.hot_level}
            onChange={(value) => handleFilterChange('hot_level', value)}
            style={{ width: 120 }}
            allowClear
          >
            <Option value={HotLevel.High}>高热度</Option>
            <Option value={HotLevel.Medium}>中热度</Option>
            <Option value={HotLevel.Low}>低热度</Option>
          </Select>
          <Button icon={<FilterOutlined />} onClick={handleReset}>
            重置筛选
          </Button>
        </Space>
      </Card>

      {/* 新闻列表 */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: '40px' }}>
          <Spin size="large" />
        </div>
      ) : newsList.length === 0 ? (
        <Card>
          <Empty description="暂无数据" />
        </Card>
      ) : (
        <>
          <Row gutter={[16, 16]}>
            {newsList.map((news) => (
              <Col key={news.id} xs={24} sm={12} md={8} lg={6}>
                <HotNewsCard
                  news={news}
                  onViewDetail={handleViewDetail}
                  onLike={handleLike}
                  isLiked={liked.has(news.id)}
                />
              </Col>
            ))}
          </Row>

          {/* 分页 */}
          <div style={{ marginTop: 24, textAlign: 'center' }}>
            <Pagination
              current={pagination.current}
              pageSize={pagination.pageSize}
              total={pagination.total}
              onChange={handlePageChange}
              showSizeChanger
              showTotal={(total) => `共 ${total} 条`}
              pageSizeOptions={['12', '24', '48', '96']}
            />
          </div>
        </>
      )}

      {/* 详情抽屉 */}
      <Drawer
        title={selectedNews?.title}
        placement="right"
        width={720}
        open={detailVisible}
        onClose={() => setDetailVisible(false)}
      >
        {selectedNews && (
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            {/* 头部信息 */}
            <div>
              <Space wrap>
                <Tag
                  color={categoryColors[selectedNews.category]}
                  icon={<TagOutlined />}
                >
                  {categoryTexts[selectedNews.category]}
                </Tag>
                <Tag
                  color={hotLevelColors[selectedNews.hot_level]}
                  icon={<FireOutlined />}
                >
                  {hotLevelTexts[selectedNews.hot_level]}
                </Tag>
              </Space>
              <div style={{ marginTop: 12 }}>
                {selectedNews.tags.map((tag) => (
                  <Tag key={tag} style={{ marginBottom: 4 }}>
                    {tag}
                  </Tag>
                ))}
              </div>
            </div>

            {/* 统计信息 */}
            <Row gutter={16}>
              <Col span={8}>
                <Card size="small">
                  <div style={{ textAlign: 'center' }}>
                    <EyeOutlined style={{ fontSize: 20, color: '#1890ff' }} />
                    <div style={{ marginTop: 8, fontSize: 20, fontWeight: 'bold' }}>
                      {selectedNews.views}
                    </div>
                    <Text type="secondary">浏览</Text>
                  </div>
                </Card>
              </Col>
              <Col span={8}>
                <Card size="small">
                  <div style={{ textAlign: 'center' }}>
                    <LikeOutlined style={{ fontSize: 20, color: '#ff4d4f' }} />
                    <div style={{ marginTop: 8, fontSize: 20, fontWeight: 'bold' }}>
                      {selectedNews.likes}
                    </div>
                    <Text type="secondary">点赞</Text>
                  </div>
                </Card>
              </Col>
              <Col span={8}>
                <Card size="small">
                  <div style={{ textAlign: 'center' }}>
                    <CommentOutlined style={{ fontSize: 20, color: '#52c41a' }} />
                    <div style={{ marginTop: 8, fontSize: 20, fontWeight: 'bold' }}>
                      {selectedNews.comments_count}
                    </div>
                    <Text type="secondary">评论</Text>
                  </div>
                </Card>
              </Col>
            </Row>

            <Divider />

            {/* 摘要 */}
            <div>
              <Text strong>摘要</Text>
              <Paragraph style={{ marginTop: 8, lineHeight: 1.8 }}>
                {selectedNews.summary}
              </Paragraph>
            </div>

            {/* 完整内容 */}
            {selectedNews.content && (
              <div>
                <Text strong>详细内容</Text>
                <div
                  style={{
                    marginTop: 8,
                    lineHeight: 1.8,
                    maxHeight: 400,
                    overflow: 'auto',
                  }}
                  dangerouslySetInnerHTML={{ __html: selectedNews.content }}
                />
              </div>
            )}

            {/* 相关股票 */}
            {selectedNews.related_stocks && selectedNews.related_stocks.length > 0 && (
              <div>
                <Text strong>相关股票</Text>
                <div style={{ marginTop: 8 }}>
                  <Space wrap>
                    {selectedNews.related_stocks.map((code) => (
                      <Tag key={code} color="blue">
                        {code}
                      </Tag>
                    ))}
                  </Space>
                </div>
              </div>
            )}

            {/* 底部信息 */}
            <div>
              <Space split={<Divider type="vertical" />}>
                <Text type="secondary">
                  发布时间：{dayjs(selectedNews.publish_time).format('YYYY-MM-DD HH:mm')}
                </Text>
                <Text type="secondary">来源：{selectedNews.source}</Text>
                {selectedNews.author && (
                  <Text type="secondary">作者：{selectedNews.author}</Text>
                )}
              </Space>
            </div>

            {/* 操作按钮 */}
            <Space>
              <Button type="primary" onClick={() => setDetailVisible(false)}>
                关闭
              </Button>
            </Space>
          </Space>
        )}
      </Drawer>
    </div>
  );
}

export default NewsHotPage;
