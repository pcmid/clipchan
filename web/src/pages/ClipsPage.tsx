import React, { useEffect, useState, useCallback } from 'react';
import { message, Modal, Table, Popconfirm } from 'antd';
import {useApi, useAuth} from '../context/AppContext';
import { useNavigate } from 'react-router-dom';
import type { Clip, Playlist } from '../types';
import { EditOutlined, CheckCircleOutlined, PlaySquareOutlined, DeleteOutlined, EyeOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './ClipsPage.css';

const ClipsPage: React.FC = () => {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [isPreviewModalVisible, setIsPreviewModalVisible] = useState(false);
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<number | null>(null);
  const [currentClipUuid, setCurrentClipUuid] = useState<string>('');
  const [previewClipTitle, setPreviewClipTitle] = useState<string>('');
  const [previewVideoUrl, setPreviewVideoUrl] = useState<string>('');
  const [loadingPlaylists, setLoadingPlaylists] = useState(false);
  const [loadingPreview, setLoadingPreview] = useState(false);
  const api = useApi();
  const navigate = useNavigate();
  const { isAdmin, canStream } = useAuth();

  const fetchClips = async () => {
    try {
      setLoading(true);
      const data = await api.listClips();
      setClips(data);
    } catch (error) {
      console.error('获取切片失败:', error);
      message.error('获取切片失败');
    } finally {
      setLoading(false);
    }
  };

  const fetchPlaylists = async () => {
    try {
      setLoadingPlaylists(true);
      const data = await api.listPlaylists();
      setPlaylists(data);
    } catch (error) {
      console.error('获取播放列表失败:', error);
      message.error('获取播放列表失败');
    } finally {
      setLoadingPlaylists(false);
    }
  };

  useEffect(() => {
    fetchClips();
  }, []);

  // Handle review clip action
  const handleReviewClip = async (uuid: string) => {
    try {
      await api.reviewedClip(uuid);
      message.success('标记为已审核');
      fetchClips();
    } catch (error) {
      console.error('标记审核失败:', error);
      message.error('标记审核失败');
    }
  };

  // Handle add to playlist action
  const handleAddToPlaylistClick = (uuid: string) => {
    setCurrentClipUuid(uuid);
    fetchPlaylists();
    setIsModalVisible(true);
  };

  // Handle delete clip action
  const handleDeleteClip = async (uuid: string) => {
    try {
      await api.deleteClip(uuid);
      message.success('删除成功');
      fetchClips();
    } catch (error) {
      console.error('删除失败:', error);
      message.error('删除失败');
    }
  };

  // Handle preview clip action (admin only)
  const handlePreviewClip = async (uuid: string, title: string) => {
    setPreviewClipTitle(title);
    setLoadingPreview(true);
    try {
      // 直接获取支持Range请求的URL，而不是下载整个文件
      const url = api.getClipPreviewUrl(uuid);
      setPreviewVideoUrl(url);
      setIsPreviewModalVisible(true);
    } catch (error) {
      console.error('获取预览视频失败:', error);
      message.error('获取预览视频失败');
    } finally {
      setLoadingPreview(false);
    }
  };

  const handlePreviewModalCancel = () => {
    setIsPreviewModalVisible(false);
    setPreviewClipTitle('');
    // 不再需要清理blob URL，因为我们使用的是直接URL
    setPreviewVideoUrl('');
  };


  const handleModalCancel = () => {
    setIsModalVisible(false);
    setSelectedPlaylistId(null);
  };

  const handleAddToPlaylist = async () => {
    if (!selectedPlaylistId) {
      message.warning('请选择一个播放列表');
      return;
    }

    try {
      await api.addClipToPlaylist({
        playlist_id: selectedPlaylistId,
        clip_uuid: currentClipUuid
      });
      message.success('成功添加到播放列表');
      setIsModalVisible(false);
      setSelectedPlaylistId(null);
    } catch (error) {
      console.error('添加到播放列表失败:', error);
      message.error('添加到播放列表失败');
    }
  };

  // 处理回车键确认的函数
  const handleEnterKeyPress = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey && isModalVisible && selectedPlaylistId && !loadingPlaylists) {
      e.preventDefault();
      handleAddToPlaylist();
    }
  }, [isModalVisible, selectedPlaylistId, loadingPlaylists]);

  // 添加和移除键盘事件监听
  useEffect(() => {
    if (isModalVisible) {
      document.addEventListener('keydown', handleEnterKeyPress);
    }
    return () => {
      document.removeEventListener('keydown', handleEnterKeyPress);
    };
  }, [isModalVisible, handleEnterKeyPress]);

  const getStatusTag = (status: string) => {
    switch (status) {
      case 'processing':
        return <span className="status-tag status-processing">处理中</span>;
      case 'failed':
        return <span className="status-tag status-failed">失败</span>;
      case 'reviewing':
        return <span className="status-tag status-reviewing">审核中</span>;
      case 'reviewed':
        return <span className="status-tag status-reviewed">已审核</span>;
      default:
        return <span className="status-tag">{status}</span>;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const columns: ColumnsType<Clip> = [
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
      width: '25%',
    },
    {
      title: 'VUP',
      dataIndex: 'vup',
      key: 'vup',
      width: '15%',
    },
    {
      title: '歌曲',
      dataIndex: 'song',
      key: 'song',
      width: '15%',
    },
    {
      title: '上传时间',
      dataIndex: 'upload_time',
      key: 'upload_time',
      width: '15%',
      render: (text: number) => formatDate(text),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: '10%',
      render: (text: string) => getStatusTag(text),
    },
    {
      title: '操作',
      key: 'action',
      width: '20%',
      render: (record: Clip) => (
        <div className="action-buttons">
          {(record.status!="reviewed" || isAdmin ) && (
          <button
            className="action-btn"
            onClick={() => navigate(`/clip/edit/${record.uuid}`)}
          >
            <EditOutlined />
            编辑
          </button>
          )}
          { (isAdmin) && (record.status === 'reviewing') && (
            <button
              className="action-btn success"
              onClick={() => handleReviewClip(record.uuid)}
            >
              <CheckCircleOutlined />
              通过审核
            </button>
          )}
          { (canStream && record.status === 'reviewed') && (
          <button
            className="action-btn"
            onClick={() => handleAddToPlaylistClick(record.uuid)}
          >
            <PlaySquareOutlined />
            添加到播放列表
          </button>
              )}
          <Popconfirm
            title="确定删除这个切片吗？"
            onConfirm={() => handleDeleteClip(record.uuid)}
            okText="是"
            cancelText="否"
          >
            <button className="action-btn danger">
              <DeleteOutlined />
              删除
            </button>
          </Popconfirm>
          {isAdmin && (
            <button
              className="action-btn"
              onClick={() => handlePreviewClip(record.uuid, record.title)}
            >
              <EyeOutlined />
              预览
            </button>
          )}
        </div>
      ),
    },
  ];

  const refreshButton = (
    <button
      className="action-btn"
      onClick={() => fetchClips()}
    >
      刷新
    </button>
  );

  return (
    <PageContainer title="我的切片" extra={refreshButton}>
      <ContentCard>
        <div className="clips-table-container">
          <Table
            columns={columns}
            dataSource={clips.map(clip => ({ ...clip, key: clip.uuid }))}
            loading={loading}
            pagination={{ pageSize: 10 }}
          />
        </div>
      </ContentCard>

      {/* 添加到播放列表的模态框 */}
      <Modal
        title="添加到播放列表"
        open={isModalVisible}
        onCancel={handleModalCancel}
        footer={null}
      >
        <div className="modal-content">
          <select
            className="modal-select"
            value={selectedPlaylistId || ''}
            onChange={(e) => setSelectedPlaylistId(Number(e.target.value))}
            disabled={loadingPlaylists}
          >
            <option value="">选择播放列表</option>
            {playlists.map(playlist => (
              <option key={playlist.id} value={playlist.id}>
                {playlist.name}
              </option>
            ))}
          </select>
          <button
            className="modal-btn"
            onClick={handleAddToPlaylist}
            disabled={!selectedPlaylistId || loadingPlaylists}
          >
            添加
          </button>
        </div>
      </Modal>

      {/* 视频预览模态框（仅限管理员） */}
      <Modal
        title={`视频预览 - ${previewClipTitle}`}
        open={isPreviewModalVisible}
        onCancel={handlePreviewModalCancel}
        footer={null}
        width={800}
        centered
      >
        <div className="preview-modal-content">
          {loadingPreview ? (
            <p>加载中...</p>
          ) : (
            previewVideoUrl && (
              <video
                className="preview-video"
                controls
                preload="metadata"
                style={{ width: '100%', maxHeight: '400px' }}
                src={previewVideoUrl}
                onError={(e) => {
                  console.error('视频加载失败:', e);
                  message.error('视频加载失败，请稍后重试');
                }}
                onLoadStart={() => {
                  console.log('视频开始加载');
                }}
                onCanPlay={() => {
                  console.log('视频可以开始播放');
                }}
              >
                您的浏览器不支持视频播放
              </video>
            )
          )}
        </div>
      </Modal>
    </PageContainer>
  );
};

export default ClipsPage;
