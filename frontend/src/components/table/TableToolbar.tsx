import React from 'react';
import {
  Space,
  Button,
  Tooltip,
  Badge,
  Popconfirm,
} from 'antd';
import {
  ReloadOutlined,
  SettingOutlined,
  ClearOutlined,
} from '@ant-design/icons';
import { TableExportButton } from './TableExportButton';
import { ColumnsType } from 'antd/es/table';

interface TableToolbarProps<T> {
  data: T[];
  columns: ColumnsType<T>;
  tableType: string;
  loading?: boolean;
  selectedRows?: T[];
  selectedRowKeys?: React.Key[];
  onRefresh?: () => void;
  onClearSelection?: () => void;
  onOpenColumnSettings?: () => void;
  extraActions?: React.ReactNode;
}

export function TableToolbar<T extends object>({
  data,
  columns,
  tableType,
  loading = false,
  selectedRows = [],
  onRefresh,
  onClearSelection,
  onOpenColumnSettings,
  extraActions,
}: TableToolbarProps<T>) {
  const hasSelection = selectedRows.length > 0;

  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 16,
        padding: '8px 0',
        borderBottom: '1px solid #f0f0f0',
      }}
    >
      <Space>
        {hasSelection && (
          <>
            <Badge
              count={selectedRows.length}
              style={{ backgroundColor: '#1890ff' }}
            />
            <span style={{ color: '#666' }}>已选择</span>
            <Popconfirm
              title="确认清空选择？"
              onConfirm={onClearSelection}
              okText="确认"
              cancelText="取消"
            >
              <Button
                size="small"
                icon={<ClearOutlined />}
                disabled={!hasSelection}
              >
                清空
              </Button>
            </Popconfirm>
          </>
        )}
        {!hasSelection && (
          <span style={{ color: '#999' }}>
            共 <strong style={{ color: '#1890ff' }}>{data.length}</strong> 条数据
          </span>
        )}
      </Space>

      <Space>
        {extraActions}
        
        <Tooltip title="刷新数据">
          <Button
            icon={<ReloadOutlined spin={loading} />}
            onClick={onRefresh}
            loading={loading}
          >
            刷新
          </Button>
        </Tooltip>

        <TableExportButton
          data={data}
          columns={columns}
          tableType={tableType}
          disabled={data.length === 0}
          selectedRows={hasSelection ? selectedRows : undefined}
        />

        {onOpenColumnSettings && (
          <Tooltip title="列设置">
            <Button
              icon={<SettingOutlined />}
              onClick={onOpenColumnSettings}
            >
              列设置
            </Button>
          </Tooltip>
        )}
      </Space>
    </div>
  );
}

export default TableToolbar;
