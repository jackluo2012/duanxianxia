import { Button, Dropdown, Space, message } from 'antd';
import { DownloadOutlined, FileExcelOutlined, FileTextOutlined } from '@ant-design/icons';
import { ColumnsType } from 'antd/es/table';
import { exportTable, getDefaultFilename } from '../../utils/tableExport';
import PermissionButton from '../PermissionButton';

interface TableExportButtonProps<T> {
  data: T[];
  columns: ColumnsType<T>;
  tableType?: string;
  filename?: string;
  disabled?: boolean;
  selectedRows?: T[];
  /**
   * 是否启用权限控制
   * @default true
   */
  enablePermissionCheck?: boolean;
}

export function TableExportButton<T extends object>({
  data,
  columns,
  tableType = 'data',
  filename,
  disabled = false,
  selectedRows,
  enablePermissionCheck = true,
}: TableExportButtonProps<T>) {
  const handleExport = (format: 'xlsx' | 'csv', selectedOnly?: boolean) => {
    try {
      const exportData = selectedOnly && selectedRows ? selectedRows : data;

      if (!exportData || exportData.length === 0) {
        message.warning('没有可导出的数据');
        return;
      }

      exportTable(exportData, columns, {
        filename: filename || getDefaultFilename(tableType),
        format,
        sheetName: getDefaultFilename(tableType),
        includeHeaders: true,
        selectedRowsOnly: selectedOnly,
      });

      message.success(`成功导出 ${exportData.length} 条数据`);
    } catch (error) {
      message.error('导出失败: ' + (error instanceof Error ? error.message : '未知错误'));
    }
  };

  const items = [
    {
      key: 'excel-all',
      label: '导出 Excel (全部)',
      icon: <FileExcelOutlined />,
      onClick: () => handleExport('xlsx', false),
    },
    {
      key: 'excel-selected',
      label: '导出 Excel (选中行)',
      icon: <FileExcelOutlined />,
      onClick: () => handleExport('xlsx', true),
      disabled: !selectedRows || selectedRows.length === 0,
    },
    {
      key: 'csv-all',
      label: '导出 CSV (全部)',
      icon: <FileTextOutlined />,
      onClick: () => handleExport('csv', false),
    },
    {
      key: 'csv-selected',
      label: '导出 CSV (选中行)',
      icon: <FileTextOutlined />,
      onClick: () => handleExport('csv', true),
      disabled: !selectedRows || selectedRows.length === 0,
    },
  ];

  // 如果启用权限检查，使用PermissionButton
  if (enablePermissionCheck) {
    return (
      <PermissionButton
        permission="screener:export:use"
        mode="disable"
        disabledTooltip="请升级到高级版或企业版以使用数据导出功能"
      >
        <Dropdown menu={{ items }} placement="bottomRight">
          <Button icon={<DownloadOutlined />} disabled={disabled}>
            <Space>
              导出
              <span style={{ fontSize: '10px' }}>▼</span>
            </Space>
          </Button>
        </Dropdown>
      </PermissionButton>
    );
  }

  // 否则使用普通的导出按钮
  return (
    <Dropdown menu={{ items }} placement="bottomRight">
      <Button icon={<DownloadOutlined />} disabled={disabled}>
        <Space>
          导出
          <span style={{ fontSize: '10px' }}>▼</span>
        </Space>
      </Button>
    </Dropdown>
  );
}

export default TableExportButton;
