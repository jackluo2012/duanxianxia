import { Card, Select, DatePicker, Form } from 'antd';
import dayjs, { Dayjs } from 'dayjs';
import { useLeaderFilters } from '../../hooks/useLeader';

const { RangePicker } = DatePicker;

interface FilterBarProps {
  onFilterChange?: () => void;
}

function FilterBar({ onFilterChange }: FilterBarProps) {
  const { filters, handleMarketChange, handleDateRangeChange, handleMinConsecutiveChange } = useLeaderFilters();

  const marketOptions = [
    { label: '全部市场', value: undefined },
    { label: '沪市', value: 1 },
    { label: '深市', value: 0 },
  ];

  const minConsecutiveOptions = [
    { label: '3板及以上', value: 3 },
    { label: '5板及以上', value: 5 },
    { label: '7板及以上', value: 7 },
    { label: '10板及以上', value: 10 },
  ];

  const handleDateChange = (dates: null | [Dayjs | null, Dayjs | null]) => {
    if (dates && dates[0] && dates[1]) {
      const dateRange: [string, string] = [
        dates[0].format('YYYY-MM-DD'),
        dates[1].format('YYYY-MM-DD'),
      ];
      handleDateRangeChange(dateRange);
      onFilterChange?.();
    }
  };

  const handleMarketSelect = (market: number | undefined) => {
    handleMarketChange(market);
    onFilterChange?.();
  };

  const handleMinConsecutiveSelect = (min: number) => {
    handleMinConsecutiveChange(min);
    onFilterChange?.();
  };

  return (
    <Card size="small" style={{ marginBottom: 16 }}>
      <Form layout="inline">
        <Form.Item label="市场">
          <Select
            style={{ width: 120 }}
            value={filters.market}
            onChange={handleMarketSelect}
            options={marketOptions}
          />
        </Form.Item>

        <Form.Item label="连板天数">
          <Select
            style={{ width: 140 }}
            value={filters.min_consecutive}
            onChange={handleMinConsecutiveSelect}
            options={minConsecutiveOptions}
          />
        </Form.Item>

        <Form.Item label="日期范围">
          <RangePicker
            value={[
              dayjs(filters.date_range[0]),
              dayjs(filters.date_range[1]),
            ]}
            onChange={handleDateChange}
            format="YYYY-MM-DD"
          />
        </Form.Item>
      </Form>
    </Card>
  );
}

export default FilterBar;
