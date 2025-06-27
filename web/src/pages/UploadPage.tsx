import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Form, message } from 'antd';
import { useApi } from '../context/AppContext';
import { useNavigate } from 'react-router-dom';
import type { ClipRequest, ServerConfig } from '../types';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './UploadPage.css';

const UploadPage: React.FC = () => {
  const [form] = Form.useForm();
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [dragOver, setDragOver] = useState(false);
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const api = useApi();
  const navigate = useNavigate();

  useEffect(() => {
    fetchServerConfig();
  }, []);

  const fetchServerConfig = async () => {
    try {
      const config = await api.getServerConfig();
      setServerConfig(config);
    } catch (error) {
      console.error('获取服务器配置失败:', error);
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const handleFileSelect = (selectedFile: File) => {
    if (serverConfig && selectedFile.size > serverConfig.max_file_size) {
      message.error(`文件大小超过限制（${formatFileSize(serverConfig.max_file_size)}）`);
      return;
    }

    if (!selectedFile.type.startsWith('video/')) {
      message.error('请选择视频文件');
      return;
    }

    setFile(selectedFile);
  };

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);

    const droppedFile = e.dataTransfer.files[0];
    if (droppedFile) {
      handleFileSelect(droppedFile);
    }
  }, [serverConfig]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      handleFileSelect(selectedFile);
    }
  };

  const handleUpload = async () => {
    if (!file) {
      message.error('请选择文件');
      return;
    }

    try {
      const values = await form.validateFields();
      setUploading(true);
      setUploadProgress(0);

      const metadata: ClipRequest = {
        title: values.title,
        vup: values.vup || '',
        song: values.song || '',
      };

      await api.uploadClip(file, metadata, (progress: number) => {
        setUploadProgress(progress);
      });

      setUploadProgress(100);
      message.success('上传成功');
      navigate('/clips');
    } catch (error) {
      console.error('上传失败:', error);
      message.error('上传失败');
    } finally {
      setUploading(false);
    }
  };

  const resetForm = () => {
    setFile(null);
    form.resetFields();
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleCancel = () => {
    navigate('/clips');
  };

  return (
    <PageContainer title="上传切片">
      <ContentCard>
        <div className="upload-form-card">
          {serverConfig && (
            <div className="size-limit-notice">
              📋 文件大小限制: {formatFileSize(serverConfig.max_file_size)}
            </div>
          )}

          {/* 文件上传区域 */}
          <div
            className={`upload-area ${dragOver ? 'dragover' : ''}`}
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onClick={() => fileInputRef.current?.click()}
          >
            <div className="upload-icon">📁</div>
            <div className="upload-text">
              {file ? file.name : '点击选择文件或拖拽到此处'}
            </div>
            <div className="upload-hint">
              支持常见视频格式（MP4、AVI、MOV等）
            </div>
          </div>

          <input
            ref={fileInputRef}
            type="file"
            className="file-input"
            accept="video/*"
            onChange={handleFileInputChange}
          />

          {/* 文件信息 */}
          {file && (
            <div className="file-info">
              <div className="file-info-title">📄 文件信息</div>
              <div className="file-detail">
                <span className="file-detail-label">文件名:</span>
                <span className="file-detail-value">{file.name}</span>
              </div>
              <div className="file-detail">
                <span className="file-detail-label">文件大小:</span>
                <span className="file-detail-value">{formatFileSize(file.size)}</span>
              </div>
              <div className="file-detail">
                <span className="file-detail-label">文件类型:</span>
                <span className="file-detail-value">{file.type}</span>
              </div>
            </div>
          )}

          {/* 上传进度 */}
          {uploading && (
            <div className="upload-progress">
              <div className="progress-bar">
                <div
                  className="progress-fill"
                  style={{ width: `${uploadProgress}%` }}
                ></div>
              </div>
              <div className="progress-text">上传中... {uploadProgress}%</div>
            </div>
          )}

          {/* 表单区域 */}
          <div className="form-section">
            <div className="form-section-title">📝 切片信息</div>

            <Form form={form} layout="vertical">
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
            </Form>
          </div>

          {/* 操作按钮 */}
          <div className="form-actions">
            <button
              className="form-btn primary"
              onClick={handleUpload}
              disabled={!file || uploading}
            >
              {uploading ? '上传中...' : '📤 开始上传'}
            </button>
            <button
              className="form-btn secondary"
              onClick={resetForm}
              disabled={uploading}
            >
              🔄 重置
            </button>
            <button
              className="form-btn secondary"
              onClick={handleCancel}
              disabled={uploading}
            >
              ↩️ 取消
            </button>
          </div>
        </div>
      </ContentCard>
    </PageContainer>
  );
};

export default UploadPage;
