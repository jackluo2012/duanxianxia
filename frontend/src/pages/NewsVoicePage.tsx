import { useState, useEffect, useRef } from 'react';
import {
  Typography,
  Card,
  Timeline,
  DatePicker,
  Tag,
  Space,
  Button,
  Empty,
  Spin,
  message,
  Row,
  Col,
  Statistic,
} from 'antd';
import {
  ClockCircleOutlined,
  SoundOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
} from '@ant-design/icons';
import dayjs from 'dayjs';
import { getVoiceNewsTimeline, getNewsStatistics } from '../api/news';
import { mockNewsStatistics } from '../mocks/news';
import type { TimelineGroup } from '../types/news';
import { NewsCategory } from '../types/news';
import AudioPlayer from '../components/news/AudioPlayer';

const { Title, Text, Paragraph } = Typography;
const { RangePicker } = DatePicker;

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

function NewsVoicePage() {
  const [timelineData, setTimelineData] = useState<TimelineGroup[]>([]);
  const [statistics, setStatistics] = useState(mockNewsStatistics);
  const [loading, setLoading] = useState(false);
  const [selectedDate, setSelectedDate] = useState<[dayjs.Dayjs, dayjs.Dayjs]>([
    dayjs().subtract(7, 'day'),
    dayjs(),
  ]);
  const [playingId, setPlayingId] = useState<string | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // 获取时间线数据
  const fetchTimeline = async () => {
    setLoading(true);
    try {
      const response = await getVoiceNewsTimeline({
        date_range: [
          selectedDate[0].format('YYYY-MM-DD'),
          selectedDate[1].format('YYYY-MM-DD'),
        ],
        page_size: 100,
      });
      setTimelineData(response.groups);
    } catch (error) {
      message.error('获取语音快讯失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

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

  // 初始加载
  useEffect(() => {
    fetchTimeline();
  }, [selectedDate]);

  // 音频播放控制
  const handlePlay = (newsId: string, audioUrl: string) => {
    if (playingId === newsId) {
      // 暂停当前播放
      audioRef.current?.pause();
      setPlayingId(null);
    } else {
      // 播放新的音频
      if (audioRef.current) {
        audioRef.current.src = audioUrl;
        audioRef.current.play();
        setPlayingId(newsId);
      }
    }
  };

  const handleAudioEnded = () => {
    setPlayingId(null);
  };

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (dateStr: string) => {
    const date = dayjs(dateStr);
    const today = dayjs();
    const yesterday = dayjs().subtract(1, 'day');

    if (date.isSame(today, 'day')) {
      return '今天';
    } else if (date.isSame(yesterday, 'day')) {
      return '昨天';
    } else {
      return date.format('MM月DD日');
    }
  };

  const formatTime = (dateTimeStr: string) => {
    return dayjs(dateTimeStr).format('HH:mm');
  };

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>语音快讯</Title>

      {/* 统计卡片 */}
      <Row gutter={16} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="今日快讯"
              value={statistics.today_voice_count}
              prefix={<SoundOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="总快讯数"
              value={statistics.total_voice_count}
            />
          </Card>
        </Col>
        <Col span={12}>
          <Card>
            <Statistic
              title="热门分类"
              value={statistics.hot_categories.map((c) => categoryTexts[c]).join('、')}
              valueStyle={{ fontSize: 14 }}
            />
          </Card>
        </Col>
      </Row>

      {/* 日期选择器 */}
      <Card style={{ marginBottom: 16 }}>
        <Space>
          <Text strong>选择日期范围：</Text>
          <RangePicker
            value={selectedDate}
            onChange={(dates) => {
              if (dates && dates[0] && dates[1]) {
                setSelectedDate([dates[0], dates[1]]);
              }
            }}
            allowClear={false}
          />
          <Button type="primary" onClick={fetchTimeline}>
            刷新
          </Button>
        </Space>
      </Card>

      {/* 时间线 */}
      <Card>
        {loading ? (
          <div style={{ textAlign: 'center', padding: '40px' }}>
            <Spin size="large" />
          </div>
        ) : timelineData.length === 0 ? (
          <Empty description="暂无数据" />
        ) : (
          <Timeline mode="left">
            {timelineData.map((group) => (
              <Timeline.Item
                key={group.date}
                label={
                  <div style={{ textAlign: 'right', minWidth: 80 }}>
                    <Text strong style={{ fontSize: 16 }}>
                      {formatDate(group.date)}
                    </Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {group.items.length} 条
                    </Text>
                  </div>
                }
                dot={<ClockCircleOutlined style={{ fontSize: '16px' }} />}
              >
                <Space direction="vertical" style={{ width: '100%' }} size="middle">
                  {group.items.map((news) => (
                    <Card
                      key={news.id}
                      size="small"
                      style={{
                        backgroundColor:
                          playingId === news.id ? '#f6ffed' : 'transparent',
                        borderColor:
                          playingId === news.id ? '#52c41a' : '#f0f0f0',
                      }}
                      hoverable
                    >
                      <Space direction="vertical" style={{ width: '100%' }} size="small">
                        {/* 标题和标签 */}
                        <div>
                          <Space>
                            <Text strong style={{ fontSize: 16 }}>
                              {news.title}
                            </Text>
                            <Tag color={categoryColors[news.category]}>
                              {categoryTexts[news.category]}
                            </Tag>
                          </Space>
                          <div style={{ marginTop: 4 }}>
                            {news.tags.map((tag) => (
                              <Tag key={tag} style={{ marginBottom: 4 }}>
                                {tag}
                              </Tag>
                            ))}
                          </div>
                        </div>

                        {/* 内容 */}
                        <Paragraph
                          style={{
                            marginBottom: 8,
                            color: '#595959',
                            lineHeight: 1.6,
                          }}
                        >
                          {news.content}
                        </Paragraph>

                        {/* 底部信息 */}
                        <Row justify="space-between" align="middle">
                          <Col>
                            <Space split={<span style={{ color: '#d9d9d9' }}>|</span>}>
                              <Text type="secondary" style={{ fontSize: 12 }}>
                                <ClockCircleOutlined /> {formatTime(news.publish_time)}
                              </Text>
                              <Text type="secondary" style={{ fontSize: 12 }}>
                                来源：{news.source}
                              </Text>
                              <Text type="secondary" style={{ fontSize: 12 }}>
                                时长：{formatDuration(news.duration)}
                              </Text>
                              <Text type="secondary" style={{ fontSize: 12 }}>
                                播放：{news.views} 次
                              </Text>
                            </Space>
                          </Col>
                          <Col>
                            <Button
                              type={
                                playingId === news.id ? 'primary' : 'default'
                              }
                              size="small"
                              icon={
                                playingId === news.id ? (
                                  <PauseCircleOutlined />
                                ) : (
                                  <PlayCircleOutlined />
                                )
                              }
                              onClick={() => handlePlay(news.id, news.audio_url)}
                            >
                              {playingId === news.id ? '暂停' : '播放'}
                            </Button>
                          </Col>
                        </Row>
                      </Space>
                    </Card>
                  ))}
                </Space>
              </Timeline.Item>
            ))}
          </Timeline>
        )}
      </Card>

      {/* 音频播放器组件 */}
      <AudioPlayer
        audioRef={audioRef}
        onEnded={handleAudioEnded}
      />
    </div>
  );
}

export default NewsVoicePage;
