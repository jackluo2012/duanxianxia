import { Select } from 'antd';
import { StockOption } from '../types';

const OPTIONS: StockOption[] = [
  { code: '000001', name: '平安银行' },
  { code: '600000', name: '浦发银行' },
];

interface StockSelectorProps {
  value: string;
  onChange: (code: string) => void;
  disabled?: boolean;
  loading?: boolean;
}

export default function StockSelector({
  value,
  onChange,
  disabled = false,
  loading = false,
}: StockSelectorProps) {
  return (
    <Select
      value={value}
      onChange={(code) => onChange(code)}
      disabled={disabled || loading}
      loading={loading}
      style={{ width: 200 }}
      placeholder="选择股票"
    >
      {OPTIONS.map((option) => (
        <Select.Option key={option.code} value={option.code}>
          {option.code} - {option.name}
        </Select.Option>
      ))}
    </Select>
  );
}
