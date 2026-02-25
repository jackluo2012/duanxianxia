import React from 'react';
import Table from 'antd/es/table';
import type { ColumnsType } from 'antd/es/table';
import { TableToolbar } from './TableToolbar';

interface EnhancedTableProps<T> {
  data: T[];
  columns: ColumnsType<T>;
  tableType: string;
  loading?: boolean;
  selectedRows?: T[];
  selectedRowKeys?: React.Key[];
  onRefresh?: () => void;
  onSelectionChange?: (keys: React.Key[], rows: T[]) => void;
  rowSelection?: any;
  storageKey?: string;
  scroll?: any;
  pagination?: any;
  rowClassName?: any;
  extraActions?: React.ReactNode;
}

export function EnhancedTable<T extends object>({
  data,
  columns,
  tableType,
  loading = false,
  selectedRows = [],
  selectedRowKeys = [],
  onRefresh,
  onSelectionChange,
  rowSelection,
  scroll = { x: 'max-content' },
  pagination = { pageSize: 20, size: 'small' },
  rowClassName,
  extraActions,
}: EnhancedTableProps<T>) {
  const [visibleColumnDefinitions, setVisibleColumnDefinitions] = React.useState<ColumnsType<T>>(columns);

  const handleClearSelection = () => {
    onSelectionChange?.([], []);
  };

  const handleOpenColumnSettings = () => {
    const newVisibleColumns = visibleColumnDefinitions.map(col => col.key as string);
    const filteredColumns = columns.filter((col: any) =>
      col.dataIndex && newVisibleColumns.includes(col.dataIndex as string)
    );
    setVisibleColumnDefinitions(filteredColumns);
  };

  return (
    <>
      <TableToolbar
        data={data}
        columns={visibleColumnDefinitions}
        tableType={tableType}
        loading={loading}
        selectedRows={selectedRows}
        selectedRowKeys={selectedRowKeys}
        onRefresh={onRefresh}
        onClearSelection={handleClearSelection}
        onOpenColumnSettings={handleOpenColumnSettings}
        extraActions={extraActions}
      />
      
      <Table
        columns={visibleColumnDefinitions}
        dataSource={data}
        loading={loading}
        rowKey={rowSelection?.rowKey || 'id'}
        size="small"
        scroll={scroll}
        pagination={pagination}
        rowClassName={rowClassName}
        rowSelection={rowSelection}
      />
    </>
  );
}

export default EnhancedTable;
