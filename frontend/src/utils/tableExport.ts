/**
 * 表格导出工具
 * 支持导出为 Excel (.xlsx) 和 CSV 格式
 */

import * as XLSX from 'xlsx';
import { saveAs } from 'file-saver';
import Papa from 'papaparse';
import type { ColumnsType } from 'antd/es/table';

export interface ExportOptions {
  filename?: string;
  format: 'xlsx' | 'csv';
  sheetName?: string;
  includeHeaders?: boolean;
  selectedRowsOnly?: boolean;
}

/**
 * 从 Ant Design 表格列定义中提取数据
 */
function extractDataFromColumns<T>(
  data: T[],
  columns: ColumnsType<T>,
  selectedRows?: T[]
): { headers: string[]; rows: any[][] } {
  // 过滤掉没有 dataIndex 的列（如操作列）
  const validColumns = columns.filter(
    (col: any) => col.dataIndex && col.title
  ) as Array<{ dataIndex: string; title: string }>;

  const headers = validColumns.map((col) => col.title);
  
  const sourceData = selectedRows || data;
  
  const rows = sourceData.map((item: any) => {
    return validColumns.map((col) => {
      const value = item[col.dataIndex];
      
      // 处理不同类型的值
      if (value === null || value === undefined) {
        return '';
      }
      if (typeof value === 'boolean') {
        return value ? '是' : '否';
      }
      if (typeof value === 'number') {
        // 保留最多4位小数
        return Number.isInteger(value) ? value : Number(value.toFixed(4));
      }
      return String(value);
    });
  });

  return { headers, rows };
}

/**
 * 导出为 Excel 格式
 */
function exportToExcel<T>(
  data: T[],
  columns: ColumnsType<T>,
  options: ExportOptions
): void {
  const { headers, rows } = extractDataFromColumns(
    data,
    columns,
    options.selectedRowsOnly ? data : undefined
  );

  // 创建工作簿
  const wb = XLSX.utils.book_new();
  
  // 创建工作表数据
  const wsData = options.includeHeaders !== false ? [headers, ...rows] : rows;
  const ws = XLSX.utils.aoa_to_sheet(wsData);

  // 设置列宽（根据内容自动调整）
  const colWidths = headers.map((header, index) => {
    const maxLength = Math.max(
      header.length,
      ...rows.map((row) => String(row[index] || '').length)
    );
    return { wch: Math.min(maxLength + 2, 30) };
  });
  ws['!cols'] = colWidths;

  // 添加工作表到工作簿
  XLSX.utils.book_append_sheet(
    wb,
    ws,
    options.sheetName || 'Sheet1'
  );

  // 生成文件并下载
  const wbout = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });
  const blob = new Blob([wbout], { type: 'application/octet-stream' });
  
  const filename = `${options.filename || 'export'}_${formatDate(new Date())}.xlsx`;
  saveAs(blob, filename);
}

/**
 * 导出为 CSV 格式
 */
function exportToCSV<T>(
  data: T[],
  columns: ColumnsType<T>,
  options: ExportOptions
): void {
  const { headers, rows } = extractDataFromColumns(
    data,
    columns,
    options.selectedRowsOnly ? data : undefined
  );

  // 准备 CSV 数据
  const csvData = options.includeHeaders !== false
    ? [headers, ...rows]
    : rows;

  // 转换为 CSV 字符串
  const csv = Papa.unparse(csvData, {
    delimiter: ',',
    header: false,
    newline: '\n',
    quotes: true,
  });

  // 添加 BOM 以支持 Excel 中文显示
  const BOM = '\uFEFF';
  const blob = new Blob([BOM + csv], { type: 'text/csv;charset=utf-8;' });
  
  const filename = `${options.filename || 'export'}_${formatDate(new Date())}.csv`;
  saveAs(blob, filename);
}

/**
 * 格式化日期为文件名格式
 */
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hour = String(date.getHours()).padStart(2, '0');
  const minute = String(date.getMinutes()).padStart(2, '0');
  return `${year}${month}${day}_${hour}${minute}`;
}

/**
 * 通用表格导出函数
 */
export function exportTable<T>(
  data: T[],
  columns: ColumnsType<T>,
  options: ExportOptions
): void {
  if (!data || data.length === 0) {
    throw new Error('没有可导出的数据');
  }

  if (!columns || columns.length === 0) {
    throw new Error('没有可导出的列');
  }

  try {
    switch (options.format) {
      case 'xlsx':
        exportToExcel(data, columns, options);
        break;
      case 'csv':
        exportToCSV(data, columns, options);
        break;
      default:
        throw new Error(`不支持的导出格式: ${options.format}`);
    }
  } catch (error) {
    console.error('导出失败:', error);
    throw error;
  }
}

/**
 * 根据表格类型获取默认文件名
 */
export function getDefaultFilename(tableType: string): string {
  const typeMap: Record<string, string> = {
    leader: '龙头高度排行',
    consecutive: '连板统计',
    limit: '涨跌停分析',
    matrix: '涨停板梯队矩阵',
  };
  return typeMap[tableType] || '表格数据';
}

export default {
  exportTable,
  getDefaultFilename,
};
