import React, { useState, useEffect, useCallback } from 'react';
import { Form, Input, Button, Upload, message, Typography, Card } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { useApi } from '../context/AppContext';
import { useNavigate } from 'react-router-dom';
import type {RcFile} from 'antd/es/upload';
import type { ServerConfig } from '../types';

const { Title } = Typography;

const UploadPage: React.FC = () => {
  const [form] = Form.useForm();
  const [fileList, setFileList] = useState<RcFile[]>([]);
  const [uploading, setUploading] = useState(false);
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);
  const [, setLoading] = useState(true);
  const api = useApi();
  const navigate = useNavigate();

  useEffect(() => {
    const fetchServerConfig = async () => {
      try {
        const config = await api.getServerConfig();
        setServerConfig(config);
      } catch (error) {
        console.error('获取服务器配置失败:', error);
        message.error('获取服务器配置失败');
      } finally {
        setLoading(false);
      }
    };

    fetchServerConfig();
  }, [api]);

  const handleUpload = async () => {
    try {
      const values = await form.validateFields();

      if (fileList.length === 0) {
        message.error('请选择要上传的视频文件');
        return;
      }

      setUploading(true);
      const file = fileList[0];
      const metadata = {
        title: values.title,
        vup: values.vup || '',
        song: values.song || '',
      };

      await api.uploadClip(file, metadata);
      message.success('上传成功');
      navigate('/clips');
    } catch (error) {
      console.error('���传失败:', error);
      message.error('上传失败');
    } finally {
      setUploading(false);
    }
  };

  const beforeUpload = (file: RcFile) => {
    const isVideo = file.type.startsWith('video/');
    if (!isVideo) {
      message.error('只能上传视频文件!');
      return false;
    }

    if (!serverConfig) {
      message.error('正在加载配置，请稍后再试!');
      return false;
    }

    // 将字节大小转换为MB进行比较
    const maxSizeMB = serverConfig.max_file_size / (1024 * 1024);
    const fileSizeMB = file.size / (1024 * 1024);
    const isValidSize = fileSizeMB < maxSizeMB;

    if (!isValidSize) {
      message.error(`视频文件必须小于${maxSizeMB.toFixed(0)}MB!`);
      return false;
    }

    setFileList([file]);
    return false; // 阻止自动上传
  };

  const handleRemove = () => {
    setFileList([]);
  };

  // 处理回车键确认的函数
  const handleEnterKeyPress = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey && !uploading) {
      // 防止在文本区域内换行时触发提交
      if (e.target instanceof HTMLTextAreaElement) {
        return;
      }
      e.preventDefault();
      handleUpload();
    }
  }, [uploading]);

  // 添加和移除键盘事件监听
  useEffect(() => {
    document.addEventListener('keydown', handleEnterKeyPress);
    return () => {
      document.removeEventListener('keydown', handleEnterKeyPress);
    };
  }, [handleEnterKeyPress]);

  return (
    <div>
      <Title level={2}>上传切片</Title>
      <Card style={{ maxWidth: '600px', margin: '0 auto' }}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ title: '', vup: '', song: '' }}
        >
          <Form.Item
            name="title"
            label="标题"
            rules={[{ required: true, message: '请输入标题' }]}
          >
            <Input placeholder="请输入切片标题" />
          </Form.Item>

          <Form.Item name="vup" label="VUP">
            <Input placeholder="请输入VUP名称（可选）" />
          </Form.Item>

          <Form.Item name="song" label="歌曲">
            <Input placeholder="请输入歌曲名称（可选）" />
          </Form.Item>

          <Form.Item label="视频文件">
            <Upload
              beforeUpload={beforeUpload}
              onRemove={handleRemove}
              fileList={fileList.map(file => ({
                uid: file.uid,
                name: file.name,
                status: 'done',
                url: URL.createObjectURL(file),
              }))}
              maxCount={1}
            >
              <Button icon={<UploadOutlined />}>选择视频文件</Button>
            </Upload>
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              onClick={handleUpload}
              loading={uploading}
              style={{ marginTop: '16px' }}
            >
              上传
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default UploadPage;
