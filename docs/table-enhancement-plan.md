# 复盘表格功能完善方案

## 一、现有功能分析

### 1. 当前已实现的表格功能

#### **LimitMatrixTable（涨停板梯队矩阵）**
- ✅ 按连板天数（板数）和题材展示股票分布
- ✅ 自动计算每行合计
- ✅ 响应式布局
- ✅ 单元格内股票列表展示（最多3只）

#### **ScreenerPage（个股挖掘页面）**
- ✅ 龙头高度表格
  - 可视化进度条显示龙头高度
  - 按龙头高度排序
  - 行样式高亮（>90%、>80%）
- ✅ 连板统计表格
  - 按连板天数排序
  - 标签展示连板天数
  - 行样式高亮（>=5天、>=3天）
- ✅ 涨跌停表格
  - 涨停/跌停分类展示
  - 首板标识
  - 涨跌幅颜色标识

#### **LeaderPage（龙头高度页面）**
- ✅ 筛选栏（市场、连板天数、日期范围）
- ✅ 虚拟滚动优化（react-window）
- ✅ 股票选择功能
- ✅ 对比功能基础

### 2. 当前缺失的功能

| 功能类别 | 缺失功能 | 优先级 |
|---------|---------|--------|
| 数据导出 | Excel/CSV导出 | P0 - 高 |
| 列定制 | 列显示/隐藏、列宽调整 | P1 - 中 |
| 交互增强 | 固定列、行选择 | P1 - 中 |
| 筛选排序 | 多列组合筛选、高级筛选 | P2 - 中 |
| 可视化 | 表格数据图表展示 | P3 - 低 |
| 性能优化 | 大数据量虚拟滚动 | P1 - 中 |
| 快捷键 | 键盘导航、快捷键 | P3 - 低 |

---

## 二、完善方案详细设计

### 功能1: 表格导出功能 (Excel/CSV) 【P0】

**需求描述:**
- 支持将表格数据导出为 Excel (.xlsx) 或 CSV 格式
- 支持导出当前页或全部数据
- 导出时保持表格样式和格式

**实现方案:**
```typescript
// 新增文件: frontend/src/utils/tableExport.ts

interface ExportOptions {
  filename?: string;
  format: 'xlsx' | 'csv';
  sheetName?: string;
  includeHeaders?: boolean;
  selectedRowsOnly?: boolean;
}

export function exportTable<T>(
  data: T[],
  columns: ColumnType<T>[],
  options: ExportOptions
): void;

// 使用示例
import { exportTable } from '@/utils/tableExport';

const handleExport = (format: 'xlsx' | 'csv') => {
  exportTable(tableData, columns, {
    filename: `涨停复盘_${dayjs().format('YYYY-MM-DD')}`,
    format,
    sheetName: '涨停数据',
    includeHeaders: true,
  });
};
```

**UI设计:**
- 在表格右上角添加导出按钮组
- 下拉菜单选择导出格式（Excel/CSV）
- 可选导出范围（当前页/全部/选中行）

---

### 功能2: 列自定义显示/隐藏 【P1】

**需求描述:**
- 用户可自定义显示哪些列
- 支持拖拽调整列顺序
- 记住用户偏好设置（LocalStorage）

**实现方案:**
```typescript
// 新增文件: frontend/src/components/ColumnSettings.tsx

interface ColumnSettingsProps<T> {
  columns: ColumnType<T>[];
  visibleColumns: string[];
  onChange: (visibleColumns: string[], columnOrder: string[]) => void;
}

// 列设置组件
const ColumnSettings = <T extends object>({
  columns,
  visibleColumns,
  onChange,
}: ColumnSettingsProps<T>) => {
  // 实现列选择和排序逻辑
};

// 在表格组件中使用
const [columnSettings, setColumnSettings] = useLocalStorage(
  'screener-table-columns',
  defaultColumnSettings
);
```

**UI设计:**
- 表格右上角设置图标按钮
- 弹出列设置抽屉/弹窗
- 可拖拽排序的列列表
- 复选框控制列显示/隐藏

---

### 功能3: 列宽调整和固定列 【P1】

**需求描述:**
- 支持拖拽调整列宽
- 支持固定左侧/右侧列（冻结列）
- 固定列在横向滚动时保持可见

**实现方案:**
```typescript
// 使用 antd Table 的 scroll 和 fixed 属性
const columns: ColumnsType<LeaderItem> = [
  {
    title: '代码',
    dataIndex: 'code',
    key: 'code',
    width: 100,
    fixed: 'left', // 固定左侧
  },
  {
    title: '名称',
    dataIndex: 'name',
    key: 'name',
    width: 120,
    fixed: 'left',
  },
  // ... 其他列
  {
    title: '操作',
    key: 'action',
    width: 100,
    fixed: 'right', // 固定右侧
  },
];

// 表格配置
<Table
  columns={columns}
  scroll={{ x: 'max-content', y: 500 }} // 横向和纵向滚动
  resizable // 启用列宽调整
/>
```

---

### 功能4: 行选择和批量操作 【P1】

**需求描述:**
- 支持单行选择和多行选择（复选框）
- 支持全选/反选
- 批量操作：导出、对比、收藏

**实现方案:**
```typescript
// 在表格组件中
const [selectedRows, setSelectedRows] = useState<LeaderItem[]>([]);
const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

const rowSelection = {
  type: 'checkbox' as const,
  selectedRowKeys,
  onChange: (keys: React.Key[], rows: LeaderItem[]) => {
    setSelectedRowKeys(keys);
    setSelectedRows(rows);
  },
  getCheckboxProps: (record: LeaderItem) => ({
    disabled: record.status === 'disabled',
  }),
};

// 批量操作栏
const BatchActions = () => (
  <Space>
    <span>已选择 {selectedRows.length} 项</span>
    <Button onClick={() => exportSelected(selectedRows)}>导出选中</Button>
    <Button onClick={() => addToCompare(selectedRows)}>批量对比</Button>
    <Button onClick={() => addToFavorites(selectedRows)}>批量收藏</Button>
    <Button onClick={() => clearSelection()}>清空选择</Button>
  </Space>
);
```

**UI设计:**
- 表格左上角显示批量操作栏（有选择时显示）
- 每行前面显示复选框
- 表头复选框支持全选/反选

---

### 功能5: 高级筛选功能 【P2】

**需求描述:**
- 支持多条件组合筛选
- 支持数值范围筛选（涨幅、成交额等）
- 支持文本模糊搜索
- 支持日期范围筛选

**实现方案:**
```typescript
// 新增文件: frontend/src/components/AdvancedFilter.tsx

interface FilterConfig {
  field: string;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'between' | 'contains';
  value: any;
}

interface AdvancedFilterProps<T> {
  columns: ColumnType<T>[];
  filters: FilterConfig[];
  onChange: (filters: FilterConfig[]) => void;
}

// 在表格组件中使用
const [filters, setFilters] = useState<FilterConfig[]>([]);

const filteredData = useMemo(() => {
  return data.filter(item => {
    return filters.every(filter => {
      const value = item[filter.field];
      switch (filter.operator) {
        case 'eq': return value === filter.value;
        case 'gt': return value > filter.value;
        case 'between': return value >= filter.value[0] && value <= filter.value[1];
        case 'contains': return String(value).includes(filter.value);
        // ... 其他操作符
      }
    });
  });
}, [data, filters]);
```

**UI设计:**
- 筛选栏区域
- 可添加多个筛选条件
- 条件之间支持"与"/"或"关系
- 支持保存筛选模板

---

### 功能6: 表格数据可视化 【P3】

**需求描述:**
- 在表格上方显示关键指标统计卡片
- 支持图表展示（柱状图、饼图、趋势图）
- 图表与表格联动（点击图表筛选表格）

**实现方案:**
```typescript
// 统计卡片
const StatCards = ({ data }: { data: LeaderItem[] }) => {
  const stats = useMemo(() => ({
    total: data.length,
    avgHeight: data.reduce((sum, d) => sum + d.leader_height, 0) / data.length,
    maxConsecutive: Math.max(...data.map(d => d.consecutive_days)),
    upCount: data.filter(d => d.change_percent > 0).length,
  }), [data]);

  return (
    <Row gutter={16}>
      <Col span={6}>
        <Statistic title="总数量" value={stats.total} />
      </Col>
      {/* ... 其他统计卡片 */}
    </Row>
  );
};

// 图表组件
const LimitUpChart = ({ data }: { data: LeaderItem[] }) => {
  const chartData = useMemo(() => {
    // 按板块分组统计
    const sectorCount = _.groupBy(data, 'sector');
    return Object.entries(sectorCount).map(([sector, items]) => ({
      name: sector,
      value: items.length,
    }));
  }, [data]);

  return (
    <PieChart
      data={chartData}
      onSliceClick={(sector) => filterBySector(sector)}
    />
  );
};
```

---

### 功能7: 性能优化 - 虚拟滚动 【P1】

**需求描述:**
- 大数据量（>1000条）时保持流畅
- 只渲染可视区域数据
- 支持动态高度行

**实现方案:**
```typescript
// 使用 react-window 或 react-virtualized
import { FixedSizeList } from 'react-window';
import { Table } from 'antd';

// 对于大数据量表格，使用虚拟滚动
const VirtualTable = ({ data, columns }: VirtualTableProps) => {
  const [tableWidth, setTableWidth] = useState(0);
  
  return (
    <Table
      dataSource={data}
      columns={columns}
      pagination={false}
      scroll={{ y: 500 }}
      components={{
        body: renderVirtualList,
      }}
    />
  );
};
```

---

### 功能8: 键盘快捷键 【P3】

**需求描述:**
- 键盘导航表格
- 快捷键操作（刷新、导出、搜索）
- 无障碍支持

**快捷键映射:**
| 快捷键 | 功能 |
|-------|------|
| Ctrl + R | 刷新数据 |
| Ctrl + E | 导出Excel |
| Ctrl + F | 聚焦搜索框 |
| ↑ / ↓ | 上下移动选择 |
| Space | 选择/取消选择行 |
| Ctrl + A | 全选 |
| Escape | 取消选择/关闭弹窗 |

---

## 三、实施计划

### 第一阶段（高优先级）- 预计 2-3 天
1. ✅ 表格导出功能（Excel/CSV）
2. ✅ 列自定义显示/隐藏
3. ✅ 列宽调整和固定列

### 第二阶段（中优先级）- 预计 3-4 天
4. ✅ 行选择和批量操作
5. ✅ 高级筛选功能
6. ✅ 性能优化（虚拟滚动）

### 第三阶段（低优先级）- 预计 2-3 天
7. ✅ 表格数据可视化
8. ✅ 键盘快捷键支持

---

## 四、技术选型

| 功能 | 技术方案 | 说明 |
|-----|---------|-----|
| Excel导出 | xlsx + FileSaver.js | 纯前端导出，无需后端 |
| CSV导出 | PapaParse | 处理CSV解析和生成 |
| 列拖拽排序 | @dnd-kit/sortable | 现代化拖拽库 |
| 虚拟滚动 | react-window | 轻量级，性能好 |
| 图表 | ECharts / Recharts | 已有ECharts依赖 |
| 快捷键 | react-hotkeys-hook | 简洁的快捷键管理 |

---

## 五、文件结构规划

```
frontend/src/
├── components/
│   ├── table/
│   │   ├── ColumnSettings.tsx      # 列设置组件
│   │   ├── AdvancedFilter.tsx      # 高级筛选组件
│   │   ├── BatchActions.tsx        # 批量操作栏
│   │   ├── ExportButton.tsx        # 导出按钮
│   │   ├── VirtualTable.tsx        # 虚拟表格封装
│   │   └── TableToolbar.tsx        # 表格工具栏
│   └── limit/
│       └── LimitMatrixTable.tsx    # 现有文件增强
├── utils/
│   └── tableExport.ts              # 表格导出工具
├── hooks/
│   ├── useTableColumns.ts          # 表格列管理hook
│   ├── useTableSelection.ts        # 表格选择hook
│   └── useTableFilter.ts           # 表格筛选hook
└── types/
    └── table.ts                    # 表格类型定义
```

---

## 六、接口变更

### 新增接口

```typescript
// 导出接口
POST /api/export/table
Request: {
  tableType: 'leader' | 'consecutive' | 'limit';
  format: 'xlsx' | 'csv';
  filters?: FilterConfig[];
  selectedIds?: string[];
}
Response: Blob

// 批量操作接口
POST /api/batch/compare
POST /api/batch/favorite
```

---

## 七、验收标准

### 功能1 - 表格导出
- [ ] 支持导出 Excel 格式
- [ ] 支持导出 CSV 格式
- [ ] 导出文件名包含日期
- [ ] 导出数据与显示数据一致
- [ ] 大文件导出有进度提示

### 功能2 - 列自定义
- [ ] 可显示/隐藏列
- [ ] 可拖拽调整列顺序
- [ ] 用户偏好持久化
- [ ] 支持重置默认配置

### 功能3 - 行选择
- [ ] 支持单行选择
- [ ] 支持多行选择（复选框）
- [ ] 支持全选/反选
- [ ] 批量操作功能正常

### 功能4 - 性能优化
- [ ] 1000+ 数据流畅滚动
- [ ] 无卡顿或白屏
- [ ] 内存占用合理

---

*文档版本: v1.0*
*创建日期: 2026-02-12*
*作者: Claude Code*
