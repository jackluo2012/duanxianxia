import { Radio } from 'antd';

interface PeriodSelectorProps {
  value: string;
  onChange: (period: string) => void;
  disabled?: boolean;
}

export default function PeriodSelector({
  value,
  onChange,
  disabled = false,
}: PeriodSelectorProps) {
  return (
    <Radio.Group value={value} onChange={(e) => onChange(e.target.value)} disabled={disabled}>
      <Radio.Button value="1m">分时</Radio.Button>
      <Radio.Button value="5m">5分</Radio.Button>
      <Radio.Button value="1d">日K</Radio.Button>
    </Radio.Group>
  );
}
