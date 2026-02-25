import { useState, useEffect } from 'react';
import {
  Typography,
  Card,
  Table,
  Tag,
  Button,
  Space,
  DatePicker,
  Select,
  Input,
  Pagination,
  Drawer,
  message,
  Row,
  Col,
  Statistic,
} from 'antd';
import {
  SearchOutlined,
  FilterOutlined,
  EyeOutlined,
  DownloadOutlined,
  FileTextOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import {
  getResearchList,
  getResearchDetail,
  getResearchStatistics,
  incrementResearchViews,
  downloadResearchPDF,
} from '../api/research';
import type {
  ResearchReport,
  ResearchQuery,
  ResearchStatistics,
} from '../types/research';
import {
  ReportType,
  RatingType,
} from '../types/research';

const { Title, Text, Paragraph } = Typography;
const { RangePicker } = DatePicker;
const { Option } = Select;

// 评级颜色映射
const ratingColors: Record<RatingType, string> = {
  [RatingType.Buy]: 'green',
  [RatingType.Overweight]: 'blue',
  [RatingType.Neutral]: 'default',
  [RatingType.Underweight]: 'orange',
  [RatingType.Sell]: 'red',
};

// 评级文本映射
const ratingTexts: Record<RatingType, string> = {
  [RatingType.Buy]: '买入',
  [RatingType.Overweight]: '增持',
  [RatingType.Neutral]: '中性',
  [RatingType.Underweight]: '减持有',
  [RatingType.Sell]: '卖出',
};

// 报告类型文本映射
const reportTypeTexts: Record<ReportType, string> = {
  [ReportType.Research]: '研究报告',
  [ReportType.Comment]: '点评报告',
  [ReportType.Deep]: '深度报告',
  [ReportType.Industry]: '行业报告',
  [ReportType.Strategy]: '策略报告',
};

function ResearchPage() {
  // 数据状态
  const [reports, setReports] = useState<ResearchReport[]>([]);
  const [statistics, setStatistics] = useState<ResearchStatistics | null>(null);
  const [selectedReport, setSelectedReport] = useState<ResearchReport | null>(null);
  const [loading, setLoading] = useState(false);

  // 分页和筛选状态
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 20,
    total: 0,
  });

  const [filters, setFilters] = useState<{
    keyword?: string;
    sector?: string;
    institute?: string;
    report_type?: ReportType;
    rating?: RatingType;
    date_range?: [dayjs.Dayjs, dayjs.Dayjs];
  }>({});

  const [detailVisible, setDetailVisible] = useState(false);

  // 获取统计数据
  useEffect(() => {
    const fetchStatistics = async () => {
      try {
        const data = await getResearchStatistics();
        setStatistics(data);
      } catch (error) {
        console.error('获取统计数据失败:', error);
      }
    };
    fetchStatistics();
  }, []);

  // 获取研报列表
  const fetchReports = async (page = pagination.current, pageSize = pagination.pageSize) => {
    setLoading(true);
    try {
      const query: ResearchQuery = {
        page,
        page_size: pageSize,
        sort_by: 'publish_date',
        sort_order: 'desc',
      };

      // 添加筛选条件
      if (filters.keyword) query.keyword = filters.keyword;
      if (filters.sector) query.sector = filters.sector;
      if (filters.institute) query.institute = filters.institute;
      if (filters.report_type) query.report_type = filters.report_type;
      if (filters.rating) query.rating = filters.rating;
      if (filters.date_range) {
        query.date_range = [
          filters.date_range[0].format('YYYY-MM-DD'),
          filters.date_range[1].format('YYYY-MM-DD'),
        ];
      }

      const response = await getResearchList(query);
      setReports(response.items);
      setPagination({
        current: response.page,
        pageSize: response.page_size,
        total: response.total,
      });
    } catch (error) {
      message.error('获取研报列表失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  // 初始加载
  useEffect(() => {
    fetchReports();
  }, []);

  // 搜索
  const handleSearch = () => {
    fetchReports(1, pagination.pageSize);
  };

  // 重置筛选
  const handleReset = () => {
    setFilters({});
    fetchReports(1, pagination.pageSize);
  };

  // 查看详情
  const handleViewDetail = async (report: ResearchReport) => {
    try {
      const detail = await getResearchDetail(report.id);
      setSelectedReport(detail);
      setDetailVisible(true);

      // 增加浏览次数
      await incrementResearchViews(report.id);
    } catch (error) {
      message.error('获取研报详情失败');
      console.error(error);
    }
  };

  // 下载PDF
  const handleDownload = async (report: ResearchReport) => {
    try {
      message.loading('正在下载...', 0);
      const blob = await downloadResearchPDF(report.id);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${report.title}.pdf`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.destroy();
      message.success('下载成功');
    } catch (error) {
      message.destroy();
      message.error('下载失败');
      console.error(error);
    }
  };

  // 分页变化
  const handlePageChange = (page: number, pageSize: number) => {
    fetchReports(page, pageSize);
  };

  // 表格列定义
  const columns: ColumnsType<ResearchReport> = [
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
      width: 300,
      ellipsis: true,
      render: (title: string) => (
        <Space direction="vertical" size={0}>
          <Text strong>{title}</Text>
        </Space>
      ),
    },
    {
      title: '股票',
      key: 'stock',
      width: 150,
      render: (_, record) => (
        <Space direction="vertical" size={0}>
          <Text strong>{record.stock_name}</Text>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {record.stock_code}
          </Text>
        </Space>
      ),
    },
    {
      title: '机构',
      dataIndex: 'institute',
      key: 'institute',
      width: 120,
      ellipsis: true,
    },
    {
      title: '分析师',
      dataIndex: 'analyst',
      key: 'analyst',
      width: 80,
    },
    {
      title: '类型',
      dataIndex: 'report_type',
      key: 'report_type',
      width: 100,
      render: (type: ReportType) => (
        <Tag color="blue">{reportTypeTexts[type]}</Tag>
      ),
    },
    {
      title: '评级',
      dataIndex: 'rating',
      key: 'rating',
      width: 80,
      render: (rating: RatingType) => (
        <Tag color={ratingColors[rating]}>{ratingTexts[rating]}</Tag>
      ),
    },
    {
      title: '目标价',
      dataIndex: 'target_price',
      key: 'target_price',
      width: 80,
      render: (price?: number) => (price ? `¥${price.toFixed(2)}` : '-'),
    },
    {
      title: '板块',
      dataIndex: 'sector',
      key: 'sector',
      width: 100,
      render: (sector: string) => <Tag>{sector}</Tag>,
    },
    {
      title: '发布日期',
      dataIndex: 'publish_date',
      key: 'publish_date',
      width: 100,
      render: (date: string) => dayjs(date).format('YYYY-MM-DD'),
    },
    {
      title: '浏览',
      dataIndex: 'views',
      key: 'views',
      width: 60,
      render: (views: number) => <Text type="secondary">{views}</Text>,
    },
    {
      title: '操作',
      key: 'action',
      width: 150,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          <Button
            type="link"
            size="small"
            icon={<EyeOutlined />}
            onClick={() => handleViewDetail(record)}
          >
            详情
          </Button>
          <Button
            type="link"
            size="small"
            icon={<DownloadOutlined />}
            onClick={() => handleDownload(record)}
          >
            下载
          </Button>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>研报检索</Title>

      {/* 统计卡片 */}
      {statistics && (
        <Row gutter={16} style={{ marginBottom: 24 }}>
          <Col span={6}>
            <Card>
              <Statistic
                title="总研报数"
                value={statistics.total_reports}
                prefix={<FileTextOutlined />}
              />
            </Card>
          </Col>
          <Col span={6}>
            <Card>
              <Statistic
                title="今日新增"
                value={statistics.today_reports}
                valueStyle={{ color: '#3f8600' }}
              />
            </Card>
          </Col>
          <Col span={6}>
            <Card>
              <Statistic title="研究机构" value={statistics.institute_count} />
            </Card>
          </Col>
          <Col span={6}>
            <Card>
              <Statistic
                title="热门板块"
                value={statistics.hot_sectors[0] || '-'}
                valueStyle={{ fontSize: 16 }}
              />
            </Card>
          </Col>
        </Row>
      )}

      {/* 筛选栏 */}
      <Card style={{ marginBottom: 16 }}>
        <Space wrap>
          <Input
            placeholder="搜索标题、摘要、股票名称/代码"
            value={filters.keyword}
            onChange={(e) => setFilters({ ...filters, keyword: e.target.value })}
            onPressEnter={handleSearch}
            style={{ width: 300 }}
            prefix={<SearchOutlined />}
            allowClear
          />
          <Select
            placeholder="选择板块"
            value={filters.sector}
            onChange={(value) => setFilters({ ...filters, sector: value })}
            style={{ width: 150 }}
            allowClear
          >
            {statistics?.hot_sectors.map((sector) => (
              <Option key={sector} value={sector}>
                {sector}
              </Option>
            ))}
          </Select>
          <Select
            placeholder="选择机构"
            value={filters.institute}
            onChange={(value) => setFilters({ ...filters, institute: value })}
            style={{ width: 150 }}
            allowClear
          >
            <Option value="中信证券">中信证券</Option>
            <Option value="国泰君安">国泰君安</Option>
            <Option value="华泰证券">华泰证券</Option>
            <Option value="海通证券">海通证券</Option>
            <Option value="招商证券">招商证券</Option>
          </Select>
          <Select
            placeholder="报告类型"
            value={filters.report_type}
            onChange={(value) => setFilters({ ...filters, report_type: value })}
            style={{ width: 130 }}
            allowClear
          >
            <Option value={ReportType.Research}>研究报告</Option>
            <Option value={ReportType.Comment}>点评报告</Option>
            <Option value={ReportType.Deep}>深度报告</Option>
            <Option value={ReportType.Industry}>行业报告</Option>
            <Option value={ReportType.Strategy}>策略报告</Option>
          </Select>
          <Select
            placeholder="评级"
            value={filters.rating}
            onChange={(value) => setFilters({ ...filters, rating: value })}
            style={{ width: 120 }}
            allowClear
          >
            <Option value={RatingType.Buy}>买入</Option>
            <Option value={RatingType.Overweight}>增持</Option>
            <Option value={RatingType.Neutral}>中性</Option>
            <Option value={RatingType.Underweight}>减持</Option>
            <Option value={RatingType.Sell}>卖出</Option>
          </Select>
          <RangePicker
            value={filters.date_range}
            onChange={(dates) =>
              setFilters({ ...filters, date_range: dates as [dayjs.Dayjs, dayjs.Dayjs] })
            }
            placeholder={['开始日期', '结束日期']}
          />
          <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
            搜索
          </Button>
          <Button icon={<FilterOutlined />} onClick={handleReset}>
            重置
          </Button>
        </Space>
      </Card>

      {/* 研报列表 */}
      <Card>
        <Table
          columns={columns}
          dataSource={reports}
          rowKey="id"
          loading={loading}
          pagination={false}
          scroll={{ x: 1500 }}
          size="middle"
        />
        <div style={{ marginTop: 16, textAlign: 'right' }}>
          <Pagination
            current={pagination.current}
            pageSize={pagination.pageSize}
            total={pagination.total}
            onChange={handlePageChange}
            showSizeChanger
            showTotal={(total) => `共 ${total} 条`}
            pageSizeOptions={['10', '20', '50', '100']}
          />
        </div>
      </Card>

      {/* 详情抽屉 */}
      <Drawer
        title={selectedReport?.title}
        placement="right"
        width={720}
        open={detailVisible}
        onClose={() => setDetailVisible(false)}
      >
        {selectedReport && (
          <Space direction="vertical" size="large" style={{ width: '100%' }}>
            {/* 基本信息 */}
            <Card title="基本信息" size="small">
              <Space direction="vertical" style={{ width: '100%' }}>
                <Row>
                  <Col span={8}>
                    <Text type="secondary">股票：</Text>
                    <Text strong>
                      {selectedReport.stock_name}（{selectedReport.stock_code}）
                    </Text>
                  </Col>
                  <Col span={8}>
                    <Text type="secondary">机构：</Text>
                    <Text>{selectedReport.institute}</Text>
                  </Col>
                  <Col span={8}>
                    <Text type="secondary">分析师：</Text>
                    <Text>{selectedReport.analyst}</Text>
                  </Col>
                </Row>
                <Row>
                  <Col span={8}>
                    <Text type="secondary">类型：</Text>
                    <Tag color="blue">{reportTypeTexts[selectedReport.report_type]}</Tag>
                  </Col>
                  <Col span={8}>
                    <Text type="secondary">评级：</Text>
                    <Tag color={ratingColors[selectedReport.rating]}>
                      {ratingTexts[selectedReport.rating]}
                    </Tag>
                  </Col>
                  <Col span={8}>
                    <Text type="secondary">目标价：</Text>
                    <Text style={{ color: '#cf1322', fontWeight: 'bold' }}>
                      {selectedReport.target_price ? `¥${selectedReport.target_price.toFixed(2)}` : '-'}
                    </Text>
                  </Col>
                </Row>
                <Row>
                  <Col span={12}>
                    <Text type="secondary">板块：</Text>
                    <Tag>{selectedReport.sector}</Tag>
                  </Col>
                  <Col span={12}>
                    <Text type="secondary">发布日期：</Text>
                    <Text>{dayjs(selectedReport.publish_date).format('YYYY-MM-DD')}</Text>
                  </Col>
                </Row>
              </Space>
            </Card>

            {/* 摘要 */}
            <Card title="摘要" size="small">
              <Paragraph>{selectedReport.summary}</Paragraph>
            </Card>

            {/* 完整内容 */}
            {selectedReport.content && (
              <Card title="完整内容" size="small">
                <div
                  style={{
                    maxHeight: 400,
                    overflow: 'auto',
                    lineHeight: 1.8,
                  }}
                  dangerouslySetInnerHTML={{ __html: selectedReport.content }}
                />
              </Card>
            )}

            {/* 操作按钮 */}
            <Space>
              <Button
                type="primary"
                icon={<DownloadOutlined />}
                onClick={() => handleDownload(selectedReport)}
              >
                下载PDF
              </Button>
              <Button onClick={() => setDetailVisible(false)}>关闭</Button>
            </Space>
          </Space>
        )}
      </Drawer>
    </div>
  );
}

export default ResearchPage;
