import React, { useEffect, useState, useCallback } from 'react';
import { Form, Input, Button, message, Typography, Card, Spin } from 'antd';
import { useApi } from '../context/AppContext';
import { useNavigate, useParams } from 'react-router-dom';
import type {ClipRequest} from '../types';

const { Title } = Typography;

const EditClipPage: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const api = useApi();
  const navigate = useNavigate();
  const { uuid } = useParams<{ uuid: string }>();

  useEffect(() => {
    const fetchClip = async () => {
      if (!uuid) return;

      try {
        setLoading(true);
        const clips = await api.listClips();
        const clip = clips.find(c => c.uuid === uuid);

        if (clip) {
          form.setFieldsValue({
            title: clip.title,
            vup: clip.vup,
            song: clip.song,
          });
        } else {
          message.error('找不到指定的切片');
          navigate('/clips');
        }
      } catch (error) {
        console.error('获取切片信息失败:', error);
        message.error('获取切片信息失败');
      } finally {
        setLoading(false);
      }
    };

    fetchClip();
  }, [uuid, api, form, navigate]);

  const handleSubmit = async () => {
    if (!uuid) return;

    try {
      const values = await form.validateFields();
      setSubmitting(true);

      const data: ClipRequest = {
        title: values.title,
        vup: values.vup || '',
        song: values.song || '',
      };

      await api.updateClip(uuid, data);
      message.success('更新成功');
      navigate('/clips');
    } catch (error) {
      console.error('更新失败:', error);
      message.error('更新失败');
    } finally {
      setSubmitting(false);
    }
  };

  // 处理回车键确认的函数
  const handleEnterKeyPress = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey && !submitting && !loading) {
      // 防止在文本区域内换行时触发提交
      if (e.target instanceof HTMLTextAreaElement) {
        return;
      }
      e.preventDefault();
      handleSubmit();
    }
  }, [submitting, loading]);

  // 添加和移除键盘事件监听
  useEffect(() => {
    document.addEventListener('keydown', handleEnterKeyPress);
    return () => {
      document.removeEventListener('keydown', handleEnterKeyPress);
    };
  }, [handleEnterKeyPress]);

  return (
    <div>
      <Title level={2}>编辑切片</Title>
      <Card style={{ maxWidth: '600px', margin: '0 auto' }}>
        {loading ? (
          <div style={{ textAlign: 'center', padding: '30px' }}>
            <Spin size="large" />
            <p style={{ marginTop: '16px' }}>加载中...</p>
          </div>
        ) : (
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

            <Form.Item>
              <Button
                type="primary"
                onClick={handleSubmit}
                loading={submitting}
              >
                保存
              </Button>
              <Button
                style={{ marginLeft: '10px' }}
                onClick={() => navigate('/clips')}
              >
                取消
              </Button>
            </Form.Item>
          </Form>
        )}
      </Card>
    </div>
  );
};

export default EditClipPage;
