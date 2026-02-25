import type {
  VoiceNews,
  HotNews,
  TimelineGroup,
} from '../types/news';
import {
  NewsCategory,
  HotLevel,
} from '../types/news';

// ========== 语音快讯 Mock 数据 ==========

const voiceNewsTitles = [
  '早盘策略：关注AI芯片方向，该板块有望成为今日主线',
  '午间点评：半导体板块强势拉升，多只个股冲击涨停',
  '收盘总结：三大指数集体收涨，成交量突破万亿',
  '盘前速递：央行今日开展5000亿元MLF操作',
  '热点解读：新能源汽车销量数据超预期，产业链受益',
  '机构观点：多家券商上调半导体行业评级至"增持"',
  '资金流向：北向资金净流入超50亿元，重点加仓科技股',
  '个股掘金：某龙头公司获机构调研，业务增长亮点多',
  '风险提示：短线追高风险加大，建议控制仓位',
  '行业动态：光伏行业大会召开，多家企业发布新品',
];

const voiceNewsContents = [
  '各位投资者大家好，今日早盘建议重点关注AI芯片方向。受相关政策利好影响，该板块有望成为今日市场主线。建议关注行业龙头标的，注意风险控制，合理配置仓位。',
  '午盘市场回顾：半导体板块表现强势，多只个股冲击涨停板。主要驱动因素包括行业景气度提升和国产替代加速。建议关注细分领域龙头，把握结构性机会。',
  '收盘情况：三大指数集体收涨，沪指涨1.2%，深成指涨1.5%，创业板指涨1.8%。两市成交金额突破万亿，北向资金净流入超50亿元。市场情绪明显回暖。',
  '重要消息：央行今日开展5000亿元MLF操作，利率维持不变。此举有助于保持流动性合理充裕，支持实体经济发展。市场预计后续仍将保持稳健货币政策。',
  '行业解读：新能源汽车最新销量数据超市场预期，多家车企销量创历史新高。受益于销量增长，整个产业链有望迎来投资机会，建议关注电池、零部件等细分领域。',
  '机构观点：近期多家券商发布研报，上调半导体行业评级至"增持"。主要理由包括：政策支持力度加大、国产替代加速、行业景气度提升。预计板块将迎来估值修复。',
  '资金监测：今日北向资金大幅净流入，金额超50亿元。从持仓变动来看，重点加仓科技股方向，包括半导体、消费电子等板块。外资持续流入显示对A股市场信心增强。',
  '个股机会：某龙头公司昨日接待多家机构调研，公司管理层透露业务增长亮点。具体包括：新产品推出进展顺利、海外市场拓展成效显著、订单量饱满。值得关注。',
  '风险提示：近期市场情绪高涨，部分个股涨幅较大，短线追高风险加大。建议投资者保持理性，控制整体仓位，关注基本面优质的标的，避免盲目追涨杀跌。',
  '行业事件：今日光伏行业大会召开，多家龙头企业发布新品。新技术路线引发市场关注，有望带来行业格局变化。建议关注技术领先、成本控制能力强的企业。',
];

const voiceNewsSources = ['短线侠资讯', '财经快讯', '投资参考', '市场观察', '股市动态'];

const voiceNewsCategories: NewsCategory[] = [
  NewsCategory.Market,
  NewsCategory.Sector,
  NewsCategory.Company,
  NewsCategory.Policy,
  NewsCategory.Technology,
];

// 生成语音快讯数据
const generateVoiceNewsData = (count: number): VoiceNews[] => {
  const news: VoiceNews[] = [];
  const now = new Date();

  for (let i = 0; i < count; i++) {
    const publishTime = new Date(now);
    publishTime.setHours(publishTime.getHours() - i * 2);

    news.push({
      id: `voice_${i + 1}`,
      title: voiceNewsTitles[i % voiceNewsTitles.length],
      content: voiceNewsContents[i % voiceNewsContents.length],
      audio_url: `https://example.com/audio/voice_${i + 1}.mp3`,
      duration: Math.floor(Math.random() * 180) + 60, // 60-240秒
      publish_time: publishTime.toISOString(),
      source: voiceNewsSources[Math.floor(Math.random() * voiceNewsSources.length)],
      category: voiceNewsCategories[Math.floor(Math.random() * voiceNewsCategories.length)],
      tags: ['A股', '投资', '股市'].slice(0, Math.floor(Math.random() * 3) + 1),
      views: Math.floor(Math.random() * 5000),
      created_at: publishTime.toISOString(),
    });
  }

  return news;
};

export const mockVoiceNewsData = generateVoiceNewsData(30);

// 按日期分组的语音快讯时间线
export const mockVoiceNewsTimeline: TimelineGroup[] = (() => {
  const groups: { [key: string]: VoiceNews[] } = {};
  const today = new Date();

  // 生成最近7天的数据
  for (let day = 0; day < 7; day++) {
    const date = new Date(today);
    date.setDate(date.getDate() - day);
    const dateStr = date.toISOString().split('T')[0];
    groups[dateStr] = [];

    const dailyCount = Math.floor(Math.random() * 8) + 3; // 每天3-10条
    for (let i = 0; i < dailyCount; i++) {
      const hour = Math.floor(Math.random() * 10) + 8; // 8-18点
      const minute = Math.floor(Math.random() * 60);
      const publishTime = new Date(date);
      publishTime.setHours(hour, minute, 0);

      const newsIndex = Math.floor(Math.random() * voiceNewsTitles.length);

      groups[dateStr].push({
        id: `voice_${dateStr}_${i}`,
        title: voiceNewsTitles[newsIndex],
        content: voiceNewsContents[newsIndex],
        audio_url: `https://example.com/audio/${dateStr}_${i}.mp3`,
        duration: Math.floor(Math.random() * 180) + 60,
        publish_time: publishTime.toISOString(),
        source: voiceNewsSources[Math.floor(Math.random() * voiceNewsSources.length)],
        category: voiceNewsCategories[Math.floor(Math.random() * voiceNewsCategories.length)],
        tags: ['A股', '投资', '财经', '市场'].slice(0, Math.floor(Math.random() * 3) + 1),
        views: Math.floor(Math.random() * 5000),
        created_at: publishTime.toISOString(),
      });
    }
  }

  return Object.entries(groups)
    .map(([date, items]) => ({
      date,
      items: items.sort((a, b) => new Date(b.publish_time).getTime() - new Date(a.publish_time).getTime()),
    }))
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
})();

// ========== 热点新闻 Mock 数据 ==========

const hotNewsTitles = [
  'AI芯片巨头发布新一代产品，性能提升300%引发市场震动',
  '新能源车企12月销量大增，全年交付量创历史新高',
  '央行最新货币政策报告：保持流动性合理充裕',
  '半导体行业迎来政策利好，国产替代加速推进',
  '光伏行业大会召开，多家企业发布高效新品',
  '消费电子展会亮点纷呈，AI手机成新趋势',
  '医药生物板块获机构密集调研，创新药受青睐',
  '白酒行业提价预期升温，高端白酒需求强劲',
  '军工企业订单饱满，行业景气度持续提升',
  '银行股估值处于历史低位，配置价值凸显',
  '房地产政策持续放松，市场信心逐步恢复',
  '化工行业整合加速，龙头企业受益',
];

const hotNewsSummaries = [
  '全球知名AI芯片厂商今日发布了最新一代处理器产品，性能相比上一代提升300%，能效比大幅改善。业内分析认为，这将加速AI技术在各行业的应用落地，相关产业链有望受益。',
  '多家新能源车企公布12月销量数据，均实现大幅增长，全年交付量创历史新高。其中，某龙头车企12月销量突破10万辆，全年销量超过50万辆。市场需求持续旺盛，产业链公司订单饱满。',
  '央行发布最新货币政策执行报告，强调要保持流动性合理充裕，支持实体经济发展。市场分析认为，这表明货币政策将继续保持稳健中性，不会出现急转弯。',
  '最新消息显示，半导体行业将获得更多政策支持，国产替代进程有望加速。多家券商发布研报，上调半导体板块评级。分析师认为，行业迎来重要发展机遇。',
  '一年一度的光伏行业大会今日召开，多家龙头企业发布了高效光伏新品。新技术路线引发市场关注，有望带来行业格局变化。机构建议关注技术领先的企业。',
  '在近期举办的消费电子展会上，AI手机成为最大亮点。多家手机厂商展示了集成AI功能的新品，引发市场广泛关注。业内分析认为，AI将成为手机行业的重要发展方向。',
  '近期，医药生物板块获得机构密集调研，尤其是创新药领域最受青睐。分析人士指出，随着医保政策优化和创新药审评加速，创新药企业迎来发展良机。',
  '临近春节，白酒行业提价预期升温。多家酒企表示，由于高端白酒需求强劲，可能考虑适当提高产品价格。机构认为，白酒板块具备配置价值。',
  '军工行业景气度持续提升，多家企业订单饱满。业内分析认为，受益于国防现代化建设，军工行业将保持快速增长。建议关注具备核心技术的龙头企业。',
  '当前银行股估值处于历史低位，具备较高的安全边际。分析人士指出，随着经济复苏和利率企稳，银行股盈利能力有望改善，配置价值凸显。',
  '房地产政策持续放松，多地出台支持措施，市场信心逐步恢复。业内分析认为，政策底已经出现，市场底有望逐步确立。建议关注优质房企和物管公司。',
  '化工行业整合加速，龙头企业凭借规模和技术优势，市场份额持续提升。分析人士认为，行业集中度提高将改善竞争格局，利好头部企业。',
];

const hotNewsCategories: NewsCategory[] = [
  NewsCategory.Technology,
  NewsCategory.Sector,
  NewsCategory.Policy,
  NewsCategory.Company,
  NewsCategory.Market,
];

const hotNewsHotLevels: HotLevel[] = [
  HotLevel.High,
  HotLevel.Medium,
  HotLevel.Low,
];

const hotNewsTags = [
  ['AI', '芯片', '科技'],
  ['新能源', '汽车', '销量'],
  ['央行', '货币', '政策'],
  ['半导体', '国产替代'],
  ['光伏', '新能源'],
  ['消费电子', 'AI手机'],
  ['医药', '创新药'],
  ['白酒', '消费'],
  ['军工', '国防'],
  ['银行', '金融'],
  ['房地产', '政策'],
  ['化工', '行业'],
];

// 生成热点新闻数据
const generateHotNewsData = (count: number): HotNews[] => {
  const news: HotNews[] = [];
  const now = new Date();

  for (let i = 0; i < count; i++) {
    const publishTime = new Date(now);
    publishTime.setHours(publishTime.getHours() - i * 4);

    const relatedStocks = [
      '000001', '600036', '000002', '600519', '000651',
      '002415', '300750', '688981', '601012', '000725'
    ].slice(0, Math.floor(Math.random() * 5));

    news.push({
      id: `hot_${i + 1}`,
      title: hotNewsTitles[i % hotNewsTitles.length],
      summary: hotNewsSummaries[i % hotNewsSummaries.length],
      cover_image: `https://example.com/images/news_${i + 1}.jpg`,
      hot_level: hotNewsHotLevels[Math.floor(Math.random() * hotNewsHotLevels.length)],
      category: hotNewsCategories[i % hotNewsCategories.length],
      tags: hotNewsTags[i % hotNewsTags.length],
      related_stocks: relatedStocks,
      publish_time: publishTime.toISOString(),
      source: '短线侠资讯',
      author: `分析师${i + 1}`,
      views: Math.floor(Math.random() * 50000) + 1000,
      likes: Math.floor(Math.random() * 5000) + 100,
      comments_count: Math.floor(Math.random() * 500) + 10,
      created_at: publishTime.toISOString(),
      updated_at: publishTime.toISOString(),
    });
  }

  return news;
};

export const mockHotNewsData = generateHotNewsData(25);

// 热点新闻详情示例
export const mockHotNewsDetail = {
  ...mockHotNewsData[0],
  content: `
## ${hotNewsTitles[0]}

### 正文内容

全球知名AI芯片厂商今日正式发布了最新一代处理器产品，这款备受期待的芯片在性能和能效方面都实现了重大突破。

#### 技术突破

根据官方介绍，新一代芯片相比上一代产品性能提升300%，能效比改善50%。这一突破主要得益于：

1. **先进制程工艺**：采用最新的3nm制程工艺，晶体管密度大幅提升
2. **创新架构设计**：全新的芯片架构，优化了数据传输和处理流程
3. **专用AI加速单元**：集成了专用的AI计算单元，大幅提升AI运算效率

#### 市场影响

业内分析认为，这一技术突破将对多个行业产生深远影响：

- **AI应用加速落地**：更强大的算力将加速AI技术在各行业的应用
- **产业链受益**：芯片设计、制造、封测等产业链环节有望受益
- **竞争格局变化**：技术领先的企业将进一步扩大市场份额

#### 投资建议

多家券商发布研报，对相关概念股维持"买入"评级。建议关注：
- 芯片设计企业
- 半导体设备企业
- AI应用企业
- 数据中心企业

#### 风险提示

- 技术落地进度不及预期
- 行业竞争加剧
- 宏观经济波动影响

---

*免责声明：以上内容仅供参考，不构成投资建议。投资有风险，入市需谨慎。*
  `,
};

// 资讯统计信息
export const mockNewsStatistics = {
  today_voice_count: 8,
  today_hot_count: 5,
  total_voice_count: mockVoiceNewsData.length,
  total_hot_count: mockHotNewsData.length,
  hot_categories: [
    NewsCategory.Technology,
    NewsCategory.Sector,
    NewsCategory.Market,
    NewsCategory.Policy,
  ],
  hot_tags: ['AI', '芯片', '新能源', '半导体', '光伏'],
};
