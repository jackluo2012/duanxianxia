import type {
  ResearchReport,
  ResearchListResponse,
  ResearchDetailResponse,
  ResearchStatistics,
  ResearchFilterOptions,
} from '../types/research';
import {
  InstituteType,
  ReportType,
  RatingType,
} from '../types/research';

// 生成随机研报数据
const generateMockReports = (count: number): ResearchReport[] => {
  const institutes = [
    '中信证券', '国泰君安', '华泰证券', '海通证券', '招商证券',
    '中金公司', '广发证券', '申万宏源', '银河证券', '东方证券',
    '兴业证券', '光大证券', '长江证券', '中泰证券', '国信证券'
  ];

  const analysts = [
    '张三', '李四', '王五', '赵六', '陈七',
    '刘八', '孙九', '周十', '吴十一', '郑十二'
  ];

  const sectors = [
    '人工智能', '新能源汽车', '半导体', '医药生物', '消费电子',
    '光伏设备', '锂电池', '白酒', '银行', '地产',
    '军工', '化工', '钢铁', '有色金属', '电力设备'
  ];

  const stockCodes = [
    '000001', '000002', '000063', '000333', '000651',
    '000725', '002415', '002594', '300015', '300750',
    '600000', '600036', '600519', '600900', '601012',
    '601318', '601398', '601857', '601988', '688981'
  ];

  const stockNames = [
    '平安银行', '万科A', '中兴通讯', '美的集团', '格力电器',
    '京东方A', '海康威视', '比亚迪', '爱尔眼科', '宁德时代',
    '浦发银行', '招商银行', '贵州茅台', '长江电力', '紫金矿业',
    '中国平安', '工商银行', '中国石油', '中国银行', '石头科技'
  ];

  const reportTypes: ReportType[] = [
    ReportType.Research,
    ReportType.Comment,
    ReportType.Deep,
    ReportType.Industry,
    ReportType.Strategy
  ];

  const ratingTypes: RatingType[] = [
    RatingType.Buy,
    RatingType.Overweight,
    RatingType.Neutral,
    RatingType.Underweight,
    RatingType.Sell
  ];

  const instituteTypes: InstituteType[] = [
    InstituteType.Securities,
    InstituteType.Fund,
    InstituteType.Other
  ];

  const titles = [
    '深度研究报告：行业龙头地位稳固，长期价值凸显',
    '投资价值分析：业绩超预期，维持买入评级',
    '事件点评：政策利好频出，行业迎来发展机遇',
    '首次覆盖报告：优质标的，具备核心竞争优势',
    '业绩预告点评：Q4业绩亮眼，看好明年增长',
    '行业动态跟踪：市场需求旺盛，景气度持续提升',
    '投资策略报告：把握结构性机会，关注细分领域',
    '风险提示报告：短期波动加大，建议谨慎配置',
    '基本面分析：盈利能力改善，估值修复空间大',
    '技术面分析：突破关键阻力位，上涨趋势确立'
  ];

  const summaries = [
    '公司作为行业龙头，具备显著的技术和品牌优势。受益于行业景气度提升，预计未来三年营收复合增长率将达到25%以上。当前估值处于历史低位，具备较高的安全边际，建议重点关注。',
    '本次财报显示，公司Q4单季度实现营收XX亿元，同比增长XX%，超出市场预期。主要得益于新产品线的快速放量以及海外市场的拓展。维持"买入"评级，目标价XX元。',
    '受近期政策利好影响，公司所处行业迎来重要发展机遇。公司提前布局相关领域，有望充分受益于政策红利。预计明年业绩将迎来显著改善，建议积极配置。',
    '公司深耕XX领域多年，积累了丰富的技术经验和客户资源。凭借强大的研发实力和完善的销售网络，公司在细分市场具备明显的竞争优势。首次覆盖给予"买入"评级。',
    '公司发布业绩预告，预计全年净利润同比增长XX%-XX%。主要驱动因素包括：1）主营业务稳健增长；2）新产品贡献显著；3）费用控制良好。看好公司长期发展潜力。',
    '近期行业调研显示，市场需求持续旺盛，产业链各环节景气度不断提升。公司作为行业龙头，将充分受益于行业景气上行。预计明年业绩将保持快速增长。',
    '当前市场环境下，建议关注结构性机会。公司在细分领域具备独特优势，有望在行业整合中受益。建议采取分批建仓策略，控制风险。',
    '受宏观经济环境影响，行业面临一定挑战。公司短期业绩可能承压，但长期发展逻辑不变。建议关注公司业绩拐点信号，择机布局。',
    '公司盈利能力持续改善，毛利率稳步提升。随着规模效应显现和运营效率提升，公司ROE有望进一步改善。当前估值水平较低，具备估值修复空间。',
    '从技术面看，公司股价已突破关键阻力位，多头排列形态明确。量价配合良好，上涨趋势确立。建议积极配置，目标价位XX元。'
  ];

  const reports: ResearchReport[] = [];

  for (let i = 0; i < count; i++) {
    const instituteIndex = Math.floor(Math.random() * institutes.length);
    const stockIndex = Math.floor(Math.random() * stockCodes.length);
    const sectorIndex = Math.floor(Math.random() * sectors.length);
    const publishDate = new Date();
    publishDate.setDate(publishDate.getDate() - Math.floor(Math.random() * 90));

    const report: ResearchReport = {
      id: `report_${i + 1}`,
      title: titles[Math.floor(Math.random() * titles.length)],
      institute: institutes[instituteIndex],
      institute_type: instituteTypes[Math.floor(Math.random() * instituteTypes.length)],
      analyst: analysts[Math.floor(Math.random() * analysts.length)],
      report_type: reportTypes[Math.floor(Math.random() * reportTypes.length)],
      rating: ratingTypes[Math.floor(Math.random() * ratingTypes.length)],
      target_price: Math.floor(Math.random() * 200) + 10,
      stock_code: stockCodes[stockIndex],
      stock_name: stockNames[stockIndex],
      sector: sectors[sectorIndex],
      publish_date: publishDate.toISOString().split('T')[0],
      summary: summaries[Math.floor(Math.random() * summaries.length)],
      pdf_url: `https://example.com/reports/report_${i + 1}.pdf`,
      views: Math.floor(Math.random() * 10000),
      created_at: publishDate.toISOString().split('T')[0],
      updated_at: publishDate.toISOString().split('T')[0],
    };

    reports.push(report);
  }

  return reports.sort((a, b) => new Date(b.publish_date).getTime() - new Date(a.publish_date).getTime());
};

// 生成50+条研报数据
export const mockResearchReports = generateMockReports(55);

// 研报列表响应
export const mockResearchListResponse: ResearchListResponse = {
  total: mockResearchReports.length,
  page: 1,
  page_size: 20,
  items: mockResearchReports.slice(0, 20),
};

// 研报详情（取第一条数据）
export const mockResearchDetail: ResearchDetailResponse = {
  ...mockResearchReports[0],
  content: `
## 投资要点

### 1. 行业背景
公司所处行业正处于快速发展阶段，受益于政策支持和技术进步，市场规模持续扩大。根据行业研究数据，预计未来三年市场复合增长率将达到25%以上。

### 2. 公司核心竞争力
- **技术优势**：公司拥有强大的研发团队和核心技术专利，技术实力行业领先
- **品牌优势**：经过多年发展，公司品牌已成为行业标杆，客户认可度高
- **渠道优势**：公司建立了完善的销售网络和渠道体系，市场覆盖面广
- **规模优势**：作为行业龙头，公司具备显著的规模效应和成本优势

### 3. 财务分析
公司财务状况良好，营收和利润保持稳定增长。2023年前三季度，公司实现营收XX亿元，同比增长XX%；净利润XX亿元，同比增长XX%。盈利能力持续改善，毛利率稳步提升。

### 4. 估值分析
采用PE估值法，给予公司2024年25倍PE，对应目标价XX元。当前股价对应2024年PE仅为XX倍，估值处于历史低位，具备较高的安全边际。

### 5. 风险提示
- 宏观经济下行风险
- 行业竞争加剧风险
- 原材料价格波动风险
- 政策变化风险

## 投资建议
我们看好公司的长期发展潜力，维持"买入"评级，目标价XX元。
  `,
};

// 研报统计信息
export const mockResearchStatistics: ResearchStatistics = {
  total_reports: mockResearchReports.length,
  today_reports: Math.floor(Math.random() * 10) + 1,
  institute_count: 15,
  hot_sectors: ['人工智能', '新能源汽车', '半导体', '医药生物', '消费电子'],
};

// 筛选选项
export const mockResearchFilterOptions: ResearchFilterOptions = {
  institutes: [
    '中信证券', '国泰君安', '华泰证券', '海通证券', '招商证券',
    '中金公司', '广发证券', '申万宏源', '银河证券', '东方证券',
    '兴业证券', '光大证券', '长江证券', '中泰证券', '国信证券'
  ],
  analysts: [
    '张三', '李四', '王五', '赵六', '陈七',
    '刘八', '孙九', '周十', '吴十一', '郑十二'
  ],
  sectors: [
    '人工智能', '新能源汽车', '半导体', '医药生物', '消费电子',
    '光伏设备', '锂电池', '白酒', '银行', '地产',
    '军工', '化工', '钢铁', '有色金属', '电力设备'
  ],
  report_types: [
    ReportType.Research,
    ReportType.Comment,
    ReportType.Deep,
    ReportType.Industry,
    ReportType.Strategy
  ],
  rating_types: [
    RatingType.Buy,
    RatingType.Overweight,
    RatingType.Neutral,
    RatingType.Underweight,
    RatingType.Sell
  ],
};
