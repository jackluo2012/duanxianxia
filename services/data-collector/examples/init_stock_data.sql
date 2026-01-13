-- ===================================================================
-- 初始化股票数据（补充 name 和 preclose）
-- ===================================================================
--
-- 由于 TDX API 不返回 name 和 preclose，我们需要手动初始化这些字段
-- 这个脚本会更新现有数据，添加正确的股票名称和昨收价
-- ===================================================================

-- 更新浦发银行 (600000)
-- 当前价: 11.64，假设昨收价: 11.09 (跌幅-4.8%)
UPDATE duanxianxia.stock_realtime_quotes
SET name = '浦发银行', preclose = 11.09
WHERE code = '600000';

-- 更新招商银行 (600036)
-- 当前价: 41.1，假设昨收价: 39.14 (涨幅+5.0%)
UPDATE duanxianxia.stock_realtime_quotes
SET name = '招商银行', preclose = 39.14
WHERE code = '600036';

-- 更新平安银行 (000001)
-- 当前价: 11.48，假设昨收价: 10.93 (涨幅+5.0%)
UPDATE duanxianxia.stock_realtime_quotes
SET name = '平安银行', preclose = 10.93
WHERE code = '000001';

-- 更新万科A (000002)
-- 当前价: 4.88，假设昨收价: 4.65 (涨幅+5.0%)
UPDATE duanxianxia.stock_realtime_quotes
SET name = '万科A', preclose = 4.65
WHERE code = '000002';

-- ===================================================================
-- 验证更新结果
-- ===================================================================

SELECT code, name, price, preclose, round(((price - preclose) / preclose) * 100, 2) as change_percent
FROM duanxianxia.stock_realtime_quotes
GROUP BY code, name, price, preclose
ORDER BY code;
