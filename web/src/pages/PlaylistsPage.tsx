import React, { useState, useEffect } from 'react';
import { Form, message, Modal, Table, Tabs } from 'antd';
import { useApi } from '../context/AppContext';
import type { Playlist, PlaylistItem, LiveArea, RoomInfo } from '../types';
import { PlusOutlined, EditOutlined, DeleteOutlined, PlayCircleOutlined, VideoCameraOutlined, StopOutlined } from '@ant-design/icons';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './PlaylistsPage.css';

const PlaylistsPage: React.FC = () => {
  const [form] = Form.useForm();
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [playlistItems, setPlaylistItems] = useState<PlaylistItem[]>([]);
  const [liveAreas, setLiveAreas] = useState<LiveArea[]>([]);
  const [roomInfo, setRoomInfo] = useState<RoomInfo | null>(null);
  const [selectedPlaylist, setSelectedPlaylist] = useState<Playlist | null>(null);
  const [selectedArea, setSelectedArea] = useState<number | null>(null);
  const [selectedParentArea, setSelectedParentArea] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [liveLoading, setLiveLoading] = useState(false);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [isLiveModalVisible, setIsLiveModalVisible] = useState(false);
  const [activeTab, setActiveTab] = useState("1");
  const [formMode, setFormMode] = useState<'add' | 'edit'>('add');
  const [editingPlaylist, setEditingPlaylist] = useState<Playlist | null>(null);
  const api = useApi();

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [playlistsData, liveAreasData] = await Promise.all([
        api.listPlaylists(),
        api.getLiveAreas(),
      ]);

      setPlaylists(playlistsData);
      setLiveAreas(liveAreasData);

      // 获取直播状态
      try {
        const roomInfoData = await api.getLiveStatus();
        setRoomInfo(roomInfoData);
      } catch (error) {
        // 可能没有直播权限或其他错误，不影响主要功能
        console.log('获取直播状态失败:', error);
      }
    } catch (error) {
      console.error('加载数据失败:', error);
      message.error('加载数据失败');
    } finally {
      setLoading(false);
    }
  };

  const handleShowModal = (mode: 'add' | 'edit', playlist?: Playlist) => {
    setFormMode(mode);
    setEditingPlaylist(playlist || null);
    if (mode === 'edit' && playlist) {
      form.setFieldsValue({
        name: playlist.name,
        description: playlist.description,
      });
    } else {
      form.resetFields();
    }
    setIsModalVisible(true);
  };

  const handleModalCancel = () => {
    setIsModalVisible(false);
    setEditingPlaylist(null);
    form.resetFields();
  };

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();

      if (formMode === 'add') {
        await api.createPlaylist(values);
        message.success('创建成功');
      } else if (editingPlaylist) {
        await api.updatePlaylist(editingPlaylist.id, values);
        message.success('更新成功');
      }

      setIsModalVisible(false);
      form.resetFields();
      loadData();
    } catch (error) {
      console.error('操作失败:', error);
      message.error('操作失败');
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await api.deletePlaylist(id);
      message.success('删除成功');
      loadData();
    } catch (error) {
      console.error('删除失败:', error);
      message.error('删除失败');
    }
  };

  const handleSetActive = async (id: number) => {
    try {
      await api.setActivePlaylist(id);
      message.success('设置为活动播放列表');
      loadData();
    } catch (error) {
      console.error('设置失败:', error);
      message.error('设置失败');
    }
  };

  const handleUnsetActive = async (id: number) => {
    try {
      await api.unsetActivePlaylist(id);
      message.success('取消活动状态');
      loadData();
    } catch (error) {
      console.error('操作失败:', error);
      message.error('操作失败');
    }
  };

  const handleViewItems = async (playlist: Playlist) => {
    try {
      const items = await api.getPlaylistItems(playlist.id);
      setPlaylistItems(items);
      setSelectedPlaylist(playlist);
      setActiveTab("2");
    } catch (error) {
      console.error('获取播放列表项目失败:', error);
      message.error('获取播放列表项目失败');
    }
  };

  const handleStartLive = async () => {
    if (!selectedArea) {
      message.warning('请选择直播分区');
      return;
    }

    try {
      setLiveLoading(true);
      await api.startLive({ area_id: selectedArea });
      message.success('开始直播');
      setIsLiveModalVisible(false);
      loadData();
    } catch (error) {
      console.error('开始直播失败:', error);
      message.error('开始直播失败');
    } finally {
      setLiveLoading(false);
    }
  };

  const handleStopLive = async () => {
    try {
      setLiveLoading(true);
      await api.stopLive();
      message.success('停止直播');
      loadData();
    } catch (error) {
      console.error('停止直播失败:', error);
      message.error('停止直播失败');
    } finally {
      setLiveLoading(false);
    }
  };

  const handleParentAreaChange = (parentAreaId: number) => {
    setSelectedParentArea(parentAreaId);
    setSelectedArea(null);
  };

  const handleAreaChange = (areaId: number) => {
    setSelectedArea(areaId);
  };

  const handleTabChange = (key: string) => {
    setActiveTab(key);
  };

  const handleShowLiveModal = () => {
    setIsLiveModalVisible(true);
  };

  const handleHideLiveModal = () => {
    setIsLiveModalVisible(false);
    setSelectedArea(null);
    setSelectedParentArea(null);
  };

  // 播放列表表格列
  const playlistColumns = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: '25%',
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
      width: '35%',
    },
    {
      title: '切片数量',
      dataIndex: 'item_count',
      key: 'item_count',
      width: '10%',
      align: 'center' as const,
    },
    {
      title: '状态',
      key: 'is_active',
      width: '10%',
      align: 'center' as const,
      render: (_: unknown, record: Playlist) => (
        record.is_active ? <span className="playlist-status-tag status-active">活动中</span> : null
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: '20%',
      render: (_: unknown, record: Playlist) => (
        <div className="action-buttons">
          <button
            className="action-btn"
            onClick={() => handleShowModal('edit', record)}
          >
            <EditOutlined />
            编辑
          </button>
          <button
            className="action-btn danger"
            onClick={() => handleDelete(record.id)}
          >
            <DeleteOutlined />
            删除
          </button>
          <button
            className="action-btn"
            onClick={() => handleViewItems(record)}
          >
            <PlayCircleOutlined />
            查看
          </button>
          {record.is_active ? (
            <button
              className="action-btn secondary"
              onClick={() => handleUnsetActive(record.id)}
            >
              取消活动
            </button>
          ) : (
            <button
              className="action-btn success"
              onClick={() => handleSetActive(record.id)}
            >
              设为活动
            </button>
          )}
        </div>
      ),
    },
  ];

  const getTabItems = () => {
    const items = [
      {
        key: "1",
        label: "播放列表",
        children: (
          <Table
            columns={playlistColumns}
            dataSource={playlists.map(p => ({ ...p, key: p.id }))}
            loading={loading}
            pagination={{ pageSize: 10 }}
          />
        )
      }
    ];

    if (selectedPlaylist) {
      items.push({
        key: "2",
        label: "播放列表内容",
        children: (
          <div>
            <div className="playlist-header">
              <h3>{selectedPlaylist.name}</h3>
              <p>{selectedPlaylist.description}</p>
            </div>
            <div className="playlist-items">
              {playlistItems.map(item => (
                <div key={item.id} className="playlist-item">
                  <span className="item-position">{item.position}</span>
                  <span className="item-title">{item.clip_title}</span>
                  <span className="item-vup">{item.clip_vup}</span>
                </div>
              ))}
            </div>
          </div>
        )
      });
    }

    return items;
  };

  const createButton = (
    <button
      className="action-btn primary"
      onClick={() => handleShowModal('add')}
    >
      <PlusOutlined />
      新建播放列表
    </button>
  );

  const liveButton = roomInfo?.live_status === 1 ? (
    <button
      className="action-btn danger"
      onClick={handleStopLive}
      disabled={liveLoading}
    >
      <StopOutlined />
      停止直播
    </button>
  ) : (
    <button
      className="action-btn success"
      onClick={handleShowLiveModal}
      disabled={liveLoading}
    >
      <VideoCameraOutlined />
      开始直播
    </button>
  );

  return (
    <PageContainer title="播放列表管理" extra={<div style={{ display: 'flex', gap: '8px' }}>{createButton}{liveButton}</div>}>
      <ContentCard>
        <Tabs activeKey={activeTab} onChange={handleTabChange} items={getTabItems()} />
      </ContentCard>

      {/* 编辑/新建播放列表模态框 */}
      <Modal
        title={formMode === 'add' ? '新建播放列表' : '编辑播放列表'}
        open={isModalVisible}
        onCancel={handleModalCancel}
        footer={null}
      >
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item
            name="name"
            label="名称"
            rules={[{ required: true, message: '请输入播放列表名称' }]}
          >
            <input className="form-input" placeholder="请输入播放列表名称" />
          </Form.Item>
          <Form.Item
            name="description"
            label="描述"
          >
            <textarea className="form-textarea" placeholder="请输入描述（可选）" />
          </Form.Item>
          <div className="form-actions">
            <button type="submit" className="form-btn primary">
              {formMode === 'add' ? '创建' : '更新'}
            </button>
            <button type="button" className="form-btn secondary" onClick={handleModalCancel}>
              取消
            </button>
          </div>
        </Form>
      </Modal>

      {/* 开始直播模态框 */}
      <Modal
        title="开始直播"
        open={isLiveModalVisible}
        onCancel={handleHideLiveModal}
        footer={null}
      >
        <div className="live-modal-content">
          <div className="form-group">
            <label>选择父分区</label>
            <select
              className="form-select"
              value={selectedParentArea || ''}
              onChange={(e) => handleParentAreaChange(Number(e.target.value))}
            >
              <option value="">请选择父分区</option>
              {liveAreas.map(area => (
                <option key={area.id} value={area.id}>
                  {area.name}
                </option>
              ))}
            </select>
          </div>

          {selectedParentArea && (
            <div className="form-group">
              <label>选择子分区</label>
              <select
                className="form-select"
                value={selectedArea || ''}
                onChange={(e) => handleAreaChange(Number(e.target.value))}
              >
                <option value="">请选择子分区</option>
                {liveAreas
                  .find(area => area.id === selectedParentArea)?.list
                  .map(subArea => (
                    <option key={subArea.id} value={subArea.id}>
                      {subArea.name}
                    </option>
                  ))}
              </select>
            </div>
          )}

          <div className="form-actions">
            <button
              className="form-btn primary"
              onClick={handleStartLive}
              disabled={!selectedArea || liveLoading}
            >
              {liveLoading ? '开始中...' : '开始直播'}
            </button>
            <button
              className="form-btn secondary"
              onClick={handleHideLiveModal}
            >
              取消
            </button>
          </div>
        </div>
      </Modal>
    </PageContainer>
  );
};

export default PlaylistsPage;
