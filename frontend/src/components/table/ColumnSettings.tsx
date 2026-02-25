import { useState } from 'react';
import {
  Modal,
  Checkbox,
  Space,
  Button,
  Typography,
  Divider,
  Input,
  Row,
  Col,
} from 'antd';
import { DragOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

interface ColumnItem {
  key: string;
  title: string;
  visible: boolean;
  width?: number;
}

interface ColumnSettingsProps {
  columns: any[];
  visibleColumns: string[];
  onChange: (visibleColumns: string[]) => void;
}

// 简化的列设置组件
function SimpleColumnSettings({
  columns,
  visibleColumns,
  onChange,
}: ColumnSettingsProps) {
  const [open, setOpen] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [tempVisibleColumns, setTempVisibleColumns] = useState<string[]>(visibleColumns);

  const allColumns = columns
    .filter((col: any) => col.dataIndex && col.title)
    .map((col: any) => ({
      key: col.dataIndex,
      title: col.title as string,
      visible: tempVisibleColumns.includes(col.dataIndex),
      width: col.width,
    })) as ColumnItem[];

  const filteredColumns = allColumns.filter(col =>
    col.title.toLowerCase().includes(searchKeyword.toLowerCase())
  );

  const handleOk = () => {
    onChange(tempVisibleColumns);
    setOpen(false);
  };

  const handleCancel = () => {
    setTempVisibleColumns(visibleColumns);
    setOpen(false);
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setTempVisibleColumns(allColumns.map(col => col.key));
    } else {
      setTempVisibleColumns([]);
    }
  };

  const handleColumnChange = (key: string, checked: boolean) => {
    if (checked) {
      setTempVisibleColumns([...tempVisibleColumns, key]);
    } else {
      setTempVisibleColumns(tempVisibleColumns.filter(k => k !== key));
    }
  };

  const selectedCount = tempVisibleColumns.length;
  const totalCount = allColumns.length;

  return (
    <>
      <Button
        icon={<DragOutlined />}
        onClick={() => setOpen(true)}
        style={{ marginBottom: 16 }}
      >
        列设置 ({selectedCount}/{totalCount})
      </Button>

      <Modal
        title={
          <Space>
            <DragOutlined />
            列设置
          </Space>
        }
        open={open}
        onOk={handleOk}
        onCancel={handleCancel}
        width={600}
        destroyOnClose
      >
        <div style={{ marginBottom: 20 }}>
          <Row justify="space-between" align="middle">
            <Col>
              <Title level={5} style={{ margin: 0 }}>
                选择要显示的列
              </Title>
            </Col>
            <Col>
              <Space>
                <Checkbox
                  checked={selectedCount === totalCount}
                  indeterminate={selectedCount > 0 && selectedCount < totalCount}
                  onChange={(e) => handleSelectAll(e.target.checked)}
                >
                  全选
                </Checkbox>
                <Text type="secondary" style={{ marginLeft: 8 }}>
                  已选择 {selectedCount}/{totalCount} 项
                </Text>
              </Space>
            </Col>
          </Row>
        </div>

        <Input
          placeholder="搜索列名称..."
          value={searchKeyword}
          onChange={(e) => setSearchKeyword(e.target.value)}
          style={{ marginBottom: 16 }}
        />

        <div style={{ maxHeight: 300, overflowY: 'auto' }}>
          {filteredColumns.map((col) => (
            <div
              key={col.key}
              style={{
                padding: '8px 12px',
                border: '1px solid #f0f0f0',
                borderRadius: '4px',
                marginBottom: '4px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <Space>
                <Checkbox
                  checked={col.visible}
                  onChange={(e) => handleColumnChange(col.key, e.target.checked)}
                >
                  {col.title}
                </Checkbox>
                {col.width && (
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    {col.width}px
                  </Text>
                )}
              </Space>
            </div>
          ))}
        </div>

        {filteredColumns.length === 0 && (
          <div style={{ textAlign: 'center', padding: '20px' }}>
            <Text type="secondary">没有找到匹配的列</Text>
          </div>
        )}

        <Divider />

        <div style={{ textAlign: 'center' }}>
          <Space>
            <Button onClick={() => setTempVisibleColumns(visibleColumns)}>
              重置
            </Button>
            <Button onClick={() => setTempVisibleColumns(['code', 'name'])}>
              最小化（仅显示基本信息）
            </Button>
            <Button onClick={() => setTempVisibleColumns(allColumns.map(col => col.key))}>
              全部显示
            </Button>
          </Space>
        </div>
      </Modal>
    </>
  );
}

export default SimpleColumnSettings;