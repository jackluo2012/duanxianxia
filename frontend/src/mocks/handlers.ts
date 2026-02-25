import { http, HttpResponse } from 'msw';
import { mockLeaderBoardResponse, mockLeaderDetail } from './leader';
import {
  mockResearchListResponse,
  mockResearchDetail,
  mockResearchStatistics,
  mockResearchFilterOptions,
} from './research';
import {
  mockVoiceNewsData,
  mockVoiceNewsTimeline,
  mockHotNewsData,
  mockHotNewsDetail,
  mockNewsStatistics,
} from './news';

export const handlers = [
  // ========== 龙头高度 API ==========
  // 获取连板排行榜
  http.get('/api/review/leader-board', () => {
    return HttpResponse.json(mockLeaderBoardResponse);
  }),

  // 获取股票详情
  http.get('/api/review/leader-detail', ({ request }) => {
    const url = new URL(request.url);
    const code = url.searchParams.get('code');

    if (code === '000001') {
      return HttpResponse.json(mockLeaderDetail);
    }

    return HttpResponse.json(mockLeaderDetail);
  }),

  // ========== 研报检索 API ==========
  // 获取研报列表
  http.get('/api/research/reports', ({ request }) => {
    const url = new URL(request.url);
    const page = parseInt(url.searchParams.get('page') || '1');
    const page_size = parseInt(url.searchParams.get('page_size') || '20');
    const keyword = url.searchParams.get('keyword');

    let filteredItems = mockResearchListResponse.items;

    // 关键词过滤
    if (keyword) {
      filteredItems = filteredItems.filter(
        (item) =>
          item.title.includes(keyword) ||
          item.summary.includes(keyword) ||
          item.stock_name.includes(keyword) ||
          item.stock_code.includes(keyword)
      );
    }

    const start = (page - 1) * page_size;
    const end = start + page_size;
    const pageItems = filteredItems.slice(start, end);

    return HttpResponse.json({
      total: filteredItems.length,
      page,
      page_size,
      items: pageItems,
    });
  }),

  // 获取研报详情
  http.get('/api/research/reports/:id', () => {
    return HttpResponse.json(mockResearchDetail);
  }),

  // 获取研报统计信息
  http.get('/api/research/statistics', () => {
    return HttpResponse.json(mockResearchStatistics);
  }),

  // 获取筛选选项
  http.get('/api/research/filter-options', () => {
    return HttpResponse.json(mockResearchFilterOptions);
  }),

  // 搜索研报
  http.get('/api/research/search', ({ request }) => {
    const url = new URL(request.url);
    const keyword = url.searchParams.get('keyword') || '';
    const page = parseInt(url.searchParams.get('page') || '1');
    const page_size = parseInt(url.searchParams.get('page_size') || '20');

    const filteredItems = mockResearchListResponse.items.filter(
      (item) =>
        item.title.includes(keyword) ||
        item.summary.includes(keyword) ||
        item.stock_name.includes(keyword) ||
        item.stock_code.includes(keyword)
    );

    const start = (page - 1) * page_size;
    const end = start + page_size;
    const pageItems = filteredItems.slice(start, end);

    return HttpResponse.json({
      total: filteredItems.length,
      page,
      page_size,
      items: pageItems,
    });
  }),

  // 增加研报浏览次数
  http.post('/api/research/reports/:id/view', () => {
    return HttpResponse.json({ views: mockResearchDetail.views + 1 });
  }),

  // ========== 资讯模块 API ==========
  // 获取语音快讯列表
  http.get('/api/news/voice', ({ request }) => {
    const url = new URL(request.url);
    const page = parseInt(url.searchParams.get('page') || '1');
    const page_size = parseInt(url.searchParams.get('page_size') || '20');

    const start = (page - 1) * page_size;
    const end = start + page_size;

    return HttpResponse.json({
      total: mockVoiceNewsData.length,
      page,
      page_size,
      items: mockVoiceNewsData.slice(start, end),
    });
  }),

  // 获取语音快讯时间线
  http.get('/api/news/voice/timeline', () => {
    return HttpResponse.json({
      total: mockVoiceNewsData.length,
      groups: mockVoiceNewsTimeline,
    });
  }),

  // 获取语音快讯详情
  http.get('/api/news/voice/:id', ({ params }) => {
    const news = mockVoiceNewsData.find((item) => item.id === params.id);
    return HttpResponse.json(news || mockVoiceNewsData[0]);
  }),

  // 增加语音快讯浏览次数
  http.post('/api/news/voice/:id/view', () => {
    return HttpResponse.json({ views: 100 });
  }),

  // 获取热点新闻列表
  http.get('/api/news/hot', ({ request }) => {
    const url = new URL(request.url);
    const page = parseInt(url.searchParams.get('page') || '1');
    const page_size = parseInt(url.searchParams.get('page_size') || '12');

    const start = (page - 1) * page_size;
    const end = start + page_size;

    return HttpResponse.json({
      total: mockHotNewsData.length,
      page,
      page_size,
      items: mockHotNewsData.slice(start, end),
    });
  }),

  // 获取热点新闻详情
  http.get('/api/news/hot/:id', () => {
    return HttpResponse.json(mockHotNewsDetail);
  }),

  // 增加热点新闻浏览次数
  http.post('/api/news/hot/:id/view', () => {
    return HttpResponse.json({ views: 1000 });
  }),

  // 点赞热点新闻
  http.post('/api/news/hot/:id/like', () => {
    return HttpResponse.json({ likes: mockHotNewsDetail.likes + 1 });
  }),

  // 取消点赞热点新闻
  http.post('/api/news/hot/:id/unlike', () => {
    return HttpResponse.json({ likes: mockHotNewsDetail.likes - 1 });
  }),

  // 获取资讯统计信息
  http.get('/api/news/statistics', () => {
    return HttpResponse.json(mockNewsStatistics);
  }),

  // 搜索资讯
  http.get('/api/news/search', ({ request }) => {
    const url = new URL(request.url);
    const keyword = url.searchParams.get('keyword') || '';

    const filteredVoice = mockVoiceNewsData.filter(
      (item) =>
        item.title.includes(keyword) || item.content.includes(keyword)
    );

    const filteredHot = mockHotNewsData.filter(
      (item) =>
        item.title.includes(keyword) || item.summary.includes(keyword)
    );

    return HttpResponse.json({
      voice: {
        total: filteredVoice.length,
        page: 1,
        page_size: 20,
        items: filteredVoice.slice(0, 20),
      },
      hot: {
        total: filteredHot.length,
        page: 1,
        page_size: 12,
        items: filteredHot.slice(0, 12),
      },
    });
  }),
];
