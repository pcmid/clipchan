import React, { useEffect, useState, useCallback } from 'react';
import { Table, Typography, Tag, Button, Space, message, Modal, Select } from 'antd';
import { useApi } from '../context/AppContext';
import type { Clip, Playlist } from '../types';
import { EditOutlined, CheckCircleOutlined, PlaySquareOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

const { Title } = Typography;
const { Option } = Select;

const ClipsPage: React.FC = () => {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<number | null>(null);
  const [currentClipUuid, setCurrentClipUuid] = useState<string>('');
  const [loadingPlaylists, setLoadingPlaylists] = useState(false);
  const api = useApi();
  const navigate = useNavigate();

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

  useEffect(() => {
    fetchClips();
  }, []);

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

  const showAddToPlaylistModal = (uuid: string) => {
    setCurrentClipUuid(uuid);
    fetchPlaylists();
    setIsModalVisible(true);
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

  const handleEdit = (uuid: string) => {
    navigate(`/clip/edit/${uuid}`);
    message.info('正在编辑切片');
  };

  const handleReview = async (uuid: string) => {
    try {
      await api.reviewedClip(uuid);
      message.success('标记为已审核');
      fetchClips();
    } catch (error) {
      console.error('标记审核失败:', error);
      message.error('标记审核失败');
    }
  };

  const getStatusTag = (status: string) => {
    switch (status) {
      case 'processing':
        return <Tag color="processing">处理中</Tag>;
      case 'failed':
        return <Tag color="error">失败</Tag>;
      case 'reviewing':
        return <Tag color="success">审核中</Tag>;
      case 'reviewed':
        return <Tag color="blue">已审核</Tag>;
      default:
        return <Tag>{status}</Tag>;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const columns = [
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
    },
    {
      title: 'VUP',
      dataIndex: 'vup',
      key: 'vup',
    },
    {
      title: '歌曲',
      dataIndex: 'song',
      key: 'song',
    },
    {
      title: '上传时间',
      dataIndex: 'upload_time',
      key: 'upload_time',
      render: (text: number) => formatDate(text),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (text: string) => getStatusTag(text),
    },
    {
      title: '操作',
      key: 'action',
      render: (_: any, record: Clip) => (
        <Space size="middle">
          <Button
            icon={<EditOutlined />}
            size="small"
            onClick={() => handleEdit(record.uuid)}
          >
            编辑
          </Button>
          {(record.status === 'reviewing') && (
            <Button
              icon={<CheckCircleOutlined />}
              size="small"
              type="primary"
              onClick={() => handleReview(record.uuid)}
            >
              通过审核
            </Button>
          )}
          <Button
            icon={<PlaySquareOutlined />}
            size="small"
            onClick={() => showAddToPlaylistModal(record.uuid)}
          >
            添加到播放列表
          </Button>
        </Space>
      ),
    },
  ];

  return (
    <div>
      <Title level={2}>我的切片</Title>
      <Table
        columns={columns}
        dataSource={clips.map(clip => ({ ...clip, key: clip.uuid }))}
        loading={loading}
        pagination={{ pageSize: 10 }}
      />
      <Modal
        title="添加到播放列表"
        visible={isModalVisible}
        onCancel={handleModalCancel}
        footer={null}
      >
        <Select
          placeholder="选择播放列表"
          onChange={value => setSelectedPlaylistId(value)}
          style={{ width: '100%' }}
          loading={loadingPlaylists}
        >
          {playlists.map(playlist => (
            <Option key={playlist.id} value={playlist.id}>
              {playlist.name}
            </Option>
          ))}
        </Select>
        <Button
          type="primary"
          onClick={handleAddToPlaylist}
          style={{ marginTop: 16, width: '100%' }}
        >
          添加
        </Button>
      </Modal>
    </div>
  );
};

export default ClipsPage;
