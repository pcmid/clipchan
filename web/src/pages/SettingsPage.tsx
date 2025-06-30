import React, { useState, useCallback, useEffect } from 'react';
import { Card, Typography, Form, Input, Button, message } from 'antd';
import { SaveOutlined, ReloadOutlined } from '@ant-design/icons';
import { useConfig } from '../context/AppContext';

const { Title } = Typography;

const SettingsPage: React.FC = () => {
  const { config, updateConfig } = useConfig();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  // 初始化表单值
  React.useEffect(() => {
    form.setFieldsValue({
      apiBaseUrl: config.apiBaseUrl,
    });
  }, [config, form]);

  const handleSave = async () => {
    try {
      setLoading(true);
      const values = await form.validateFields();

      updateConfig({
        apiBaseUrl: values.apiBaseUrl,
      });

      message.success('设置已保存');
    } catch (error) {
      console.error('保存设置失败:', error);
      message.error('保存设置失败');
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    form.setFieldsValue({
      apiBaseUrl: config.defaultApiBaseUrl,
    });
  };

  // 添加回车键确认功能
  const handleEnterKey = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleSave();
      }
    },
    [handleSave]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleEnterKey);
    return () => {
      window.removeEventListener('keydown', handleEnterKey);
    };
  }, [handleEnterKey]);

  return (
    <div>
      <Title level={2}>系统设置</Title>
      <Card style={{ maxWidth: '600px' }}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ apiBaseUrl: config.apiBaseUrl }}
        >
          <Form.Item
            name="apiBaseUrl"
            label="后端API地址"
            rules={[
              { required: true, message: '请输入后端API地址' },
              { type: 'url', message: '请输入有效的URL' }
            ]}
            extra="设置后端服务器的API基础URL，例如: http://localhost:3000"
          >
            <Input placeholder="请输入后端API地址" />
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              icon={<SaveOutlined />}
              onClick={handleSave}
              loading={loading}
              style={{ marginRight: '10px' }}
            >
              保存设置
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={handleReset}
            >
              恢复默认
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default SettingsPage;
