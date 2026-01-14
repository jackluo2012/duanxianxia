import { http, HttpResponse } from 'msw';
import { mockLeaderBoardResponse, mockLeaderDetail } from './leader';

export const handlers = [
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
];
