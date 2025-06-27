import React, { useEffect, useState, useCallback } from 'react';
import { Form, message } from 'antd';
import { useApi } from '../context/AppContext';
import { useNavigate, useParams } from 'react-router-dom';
import type {ClipRequest} from '../types';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './EditClipPage.css';

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

  const handleCancel = () => {
    navigate('/clips');
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

  const backButton = (
    <button
      className="form-btn secondary"
      onClick={() => navigate('/clips')}
    >
      返回列表
    </button>
  );

  return (
    <PageContainer title="编辑切片" extra={backButton}>
      <ContentCard loading={loading}>
        {!loading && (
          <div className="edit-form-card">
            <Form
              form={form}
              layout="vertical"
              onFinish={handleSubmit}
            >
              <div className="form-group">
                <label className="form-label">标题 *</label>
                <Form.Item
                  name="title"
                  rules={[{ required: true, message: '请输入标题' }]}
                >
                  <input
                    className="form-input required"
                    placeholder="请输入切片标题"
                  />
                </Form.Item>
              </div>

              <div className="form-group">
                <label className="form-label">VUP</label>
                <Form.Item name="vup">
                  <input
                    className="form-input"
                    placeholder="请输入VUP名称（可选）"
                  />
                </Form.Item>
              </div>

              <div className="form-group">
                <label className="form-label">歌曲</label>
                <Form.Item name="song">
                  <input
                    className="form-input"
                    placeholder="请输入歌曲名称（可选）"
                  />
                </Form.Item>
              </div>

              <div className="form-actions">
                <button
                  type="submit"
                  className="form-btn primary"
                  disabled={submitting}
                >
                  {submitting ? '保存中...' : '保存'}
                </button>
                <button
                  type="button"
                  className="form-btn secondary"
                  onClick={handleCancel}
                >
                  取消
                </button>
              </div>
            </Form>
          </div>
        )}
      </ContentCard>
    </PageContainer>
  );
};

export default EditClipPage;
