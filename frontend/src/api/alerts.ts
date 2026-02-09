import request from './request';

export interface AlertRuleType {
  change_percent?: { threshold: number };
  sealed_amount?: { threshold: number };
  intensity_score?: { threshold: number };
  anomaly?: { volume_change_ratio: number; price_change_ratio: number };
}

export interface AlertRule {
  id: string;
  name: string;
  rule_type: 'change_percent' | 'sealed_amount' | 'intensity_score' | 'anomaly';
  rule_config: AlertRuleType;
  enabled: boolean;
  created_at: string;
}

export interface AlertEvent {
  id: string;
  rule_id: string;
  rule_name: string;
  stock_code: string;
  stock_name: string;
  message: string;
  severity: 'info' | 'warning' | 'critical';
  triggered_at: string;
  metadata: {
    price: number;
    change_percent: number;
    sealed_amount_buy: number;
    sealed_amount_sell: number;
  };
}

// 获取告警规则列表
export async function getAlertRules(): Promise<AlertRule[]> {
  const response = await request.get<{rules: AlertRule[]}>('/auction/alerts');
  return response.rules || [];
}

// 创建告警规则
export async function createAlertRule(
  name: string,
  ruleType: AlertRuleType,
  enabled: boolean = true
): Promise<AlertRule> {
  return request.post('/auction/alerts', {
    name,
    rule_type: ruleType,
    enabled,
  });
}

// 删除告警规则
export async function deleteAlertRule(ruleId: string): Promise<void> {
  return request.delete(`/auction/alerts/${ruleId}`);
}

// 获取告警历史
export async function getAlertHistory(limit: number = 100): Promise<AlertEvent[]> {
  const response = await request.get<{alerts: AlertEvent[]}>(
    `/auction/alerts/history?limit=${limit}`
  );
  return response.alerts || [];
}
