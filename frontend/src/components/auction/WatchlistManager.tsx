import { useState, useEffect } from 'react';
import {
  Card,
  Form,
  Input,
  Button,
  Table,
  message,
  Tag,
  Popconfirm,
} from 'antd';
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons';
import {
  getWatchlist,
  addToWatchlist,
  removeFromWatchlist,
  type WatchlistItem,
} from '../../api/watchlist';

function WatchlistManager() {
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    fetchWatchlist();
  }, []);

  const fetchWatchlist = async () => {
    setLoading(true);
    try {
      const data = await getWatchlist();
      setWatchlist(data);
    } catch (error) {
      console.error('Failed to fetch watchlist:', error);
      message.error('加载自选股失败');
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = async () => {
    try {
      const values = await form.validateFields();
      await addToWatchlist(values.code, values.name);
      message.success('股票已添加到自选股');
      form.resetFields();
      fetchWatchlist();
    } catch (error: any) {
      if (error.errorFields) {
        return; // 表单验证错误
      }
      console.error('Failed to add to watchlist:', error);
      message.error(error.response?.data?.message || '添加自选股失败');
    }
  };

  const handleRemove = async (code: string) => {
    try {
      await removeFromWatchlist(code);
      message.success('股票已从自选股中移除');
      fetchWatchlist();
    } catch (error) {
      console.error('Failed to remove from watchlist:', error);
      message.error('移除自选股失败');
    }
  };

  const columns = [
    {
      title: '股票代码',
      dataIndex: 'code',
      key: 'code',
      width: 120,
      render: (code: string) => <Tag color="blue">{code}</Tag>,
    },
    {
      title: '股票名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
    },
    {
      title: '添加时间',
      dataIndex: 'added_at',
      key: 'added_at',
      render: (time: string) => new Date(time).toLocaleString('zh-CN'),
    },
    {
      title: '操作',
      key: 'action',
      width: 100,
      render: (_: any, record: WatchlistItem) => (
        <Popconfirm
          title="确认移除"
          description="确定要将此股票从自选中移除吗？"
          onConfirm={() => handleRemove(record.code)}
          okText="确定"
          cancelText="取消"
        >
          <Button type="link" danger icon={<DeleteOutlined />}>
            移除
          </Button>
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      <Card title="添加股票到自选股" style={{ marginBottom: 16 }}>
        <Form form={form} layout="inline">
          <Form.Item
            name="code"
            label="股票代码"
            rules={[{ required: true, message: '请输入股票代码' }]}
          >
            <Input placeholder="如: 600519" style={{ width: 150 }} />
          </Form.Item>

          <Form.Item
            name="name"
            label="股票名称"
            rules={[{ required: true, message: '请输入股票名称' }]}
          >
            <Input placeholder="如: 贵州茅台" style={{ width: 200 }} />
          </Form.Item>

          <Form.Item>
            <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
              添加
            </Button>
          </Form.Item>
        </Form>
      </Card>

      <Card title="自选股列表" extra={<Tag color="processing">共 {watchlist.length} 只</Tag>}>
        <Table
          dataSource={watchlist}
          columns={columns}
          rowKey="code"
          loading={loading}
          pagination={{ pageSize: 20 }}
          size="small"
        />
      </Card>
    </div>
  );
}

export default WatchlistManager;
