import React, { useEffect, useState, useCallback } from 'react';
import {
  Typography,
  Table,
  Button,
  Modal,
  Form,
  Input,
  message,
  Space,
  Tabs,
  Card,
  Select,
  Divider,
  Tag,
  Spin
} from 'antd';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  PlayCircleOutlined,
  VideoCameraOutlined,
  StopOutlined
} from '@ant-design/icons';
import { useApi } from '../context/AppContext';
import type {Playlist, PlaylistRequest, PlaylistItem, Clip, LiveArea, RoomInfo} from '../types';

const { Title, Text } = Typography;
const { TextArea } = Input;
const { Option } = Select;

const PlaylistsPage: React.FC = () => {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [clips, setClips] = useState<Clip[]>([]);
  const [playlistItems, setPlaylistItems] = useState<PlaylistItem[]>([]);
  const [liveAreas, setLiveAreas] = useState<LiveArea[]>([]);
  const [roomInfo, setRoomInfo] = useState<RoomInfo | null>(null);
  const [selectedPlaylist, setSelectedPlaylist] = useState<Playlist | null>(null);
  const [selectedArea, setSelectedArea] = useState<number | null>(null);
  const [selectedParentArea, setSelectedParentArea] = useState<number | null>(null);

  const [loading, setLoading] = useState(true);
  const [liveLoading, setLiveLoading] = useState(true);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [isLiveModalVisible, setIsLiveModalVisible] = useState(false);
  const [activeTab, setActiveTab] = useState('1');
  const [formMode, setFormMode] = useState<'add' | 'edit'>('add');
  const [form] = Form.useForm();

  const api = useApi();

  // 获取播放列表
  const fetchPlaylists = async () => {
    try {
      setLoading(true);
      const data = await api.listPlaylists();
      setPlaylists(data);
    } catch (error) {
      console.error('获取播放列表失败:', error);
      message.error('获取播放列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 获取切片列表
  const fetchClips = async () => {
    try {
      const data = await api.listClips();
      setClips(data.filter(clip => clip.status === 'reviewed')); // 只显示已审核的切片
    } catch (error) {
      console.error('获取切片失败:', error);
      message.error('获取切片失败');
    }
  };

  // 获取直播分区
  const fetchLiveAreas = async () => {
    try {
      setLiveLoading(true);
      const data = await api.getLiveAreas();
      setLiveAreas(data);
      if (data.length > 0) {
        setSelectedParentArea(data[0].id);
      }
    } catch (error) {
      console.error('获取直播分区失败:', error);
      message.error('获取直播分区失败');
    } finally {
      setLiveLoading(false);
    }
  };

  // 获取直播状态
  const fetchLiveStatus = async () => {
    try {
      setLiveLoading(true);
      const data = await api.getLiveStatus();
      setRoomInfo(data);
    } catch (error) {
      console.error('获取直播状态失败:', error);
      // 可能未开播，不显示错误
      setRoomInfo(null);
    } finally {
      setLiveLoading(false);
    }
  };

  // 获取播放列表项目
  const fetchPlaylistItems = async (playlistId: number) => {
    try {
      const data = await api.getPlaylistItems(playlistId);
      setPlaylistItems(data);
    } catch (error) {
      console.error('获取播放列表项目失败:', error);
      message.error('获取播放列表项目失败');
    }
  };

  useEffect(() => {
    fetchPlaylists();
    fetchClips();
    fetchLiveAreas();
    fetchLiveStatus();
  }, []);

  // 当选择的父分区改变时，重置子分区
  useEffect(() => {
    if (selectedParentArea !== null && liveAreas.length > 0) {
      const parentArea = liveAreas.find(area => area.id === selectedParentArea);
      if (parentArea && parentArea.list.length > 0) {
        setSelectedArea(parseInt(parentArea.list[0].id));
      } else {
        setSelectedArea(null);
      }
    }
  }, [selectedParentArea, liveAreas]);

  // 打开新增/编辑播放列表模态框
  const showModal = (mode: 'add' | 'edit', playlist?: Playlist) => {
    setFormMode(mode);
    if (mode === 'edit' && playlist) {
      form.setFieldsValue({
        name: playlist.name,
        description: playlist.description,
      });
      setSelectedPlaylist(playlist);
    } else {
      form.resetFields();
      setSelectedPlaylist(null);
    }
    setIsModalVisible(true);

    // 添加回车键监听
    setTimeout(() => {
      document.addEventListener('keydown', handlePlaylistModalKeyDown);
    }, 100);
  };

  // 关闭模态框
  const handleCancel = () => {
    setIsModalVisible(false);
    form.resetFields();
    document.removeEventListener('keydown', handlePlaylistModalKeyDown);
  };

  // 提交表单
  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();
      const data: PlaylistRequest = {
        name: values.name,
        description: values.description,
      };

      if (formMode === 'add') {
        await api.createPlaylist(data);
        message.success('创建播放列表成功');
      } else if (formMode === 'edit' && selectedPlaylist) {
        await api.updatePlaylist(selectedPlaylist.id, data);
        message.success('更新播放列表成功');
      }

      fetchPlaylists();
      setIsModalVisible(false);
    } catch (error) {
      console.error('操作失败:', error);
      message.error('操作失败');
    }
  };

  // 删除播放列表
  const handleDelete = async (id: number) => {
    Modal.confirm({
      title: '确认删除',
      content: '确定要删除这个播放列表吗？',
      onOk: async () => {
        try {
          await api.deletePlaylist(id);
          message.success('删除成功');
          fetchPlaylists();
        } catch (error) {
          console.error('删除失败:', error);
          message.error('删除失败');
        }
      },
    });
  };

  // 设置激活播放列表
  const handleSetActive = async (id: number) => {
    try {
      await api.setActivePlaylist(id);
      message.success('设置活动播放列表成功');
      fetchPlaylists();
    } catch (error) {
      console.error('设置失败:', error);
      message.error('设置失败');
    }
  };

  // 取消激活播放列表
  const handleUnsetActive = async (id: number) => {
    try {
      await api.unsetActivePlaylist(id);
      message.success('取消活动播放列表成功');
      fetchPlaylists();
    } catch (error) {
      console.error('操作失败:', error);
      message.error('操作失败');
    }
  };

  // 查看播放列表项目
  const handleViewItems = (playlist: Playlist) => {
    setSelectedPlaylist(playlist);
    fetchPlaylistItems(playlist.id);
    setActiveTab('2');
  };

  // 将切片添加到播放列表
  const handleAddToPlaylist = async (clipUuid: string) => {
    if (!selectedPlaylist) {
      message.error('请先选择一个播放列表');
      return;
    }

    try {
      await api.addClipToPlaylist({
        playlist_id: selectedPlaylist.id,
        clip_uuid: clipUuid
      });
      message.success('添加成功');
      fetchPlaylistItems(selectedPlaylist.id);
    } catch (error) {
      console.error('添加失败:', error);
      message.error('添加失败');
    }
  };

  // 从播放列表中移除切片
  const handleRemoveFromPlaylist = async (clipUuid: string) => {
    if (!selectedPlaylist) return;

    try {
      await api.removeClipFromPlaylist({
        playlist_id: selectedPlaylist.id,
        clip_uuid: clipUuid
      });
      message.success('移除成功');
      fetchPlaylistItems(selectedPlaylist.id);
    } catch (error) {
      console.error('���除失败:', error);
      message.error('移除失败');
    }
  };

  // 重新排序播放列表项目
  const handleReorderItem = async (itemId: number, newPosition: number) => {
    if (!selectedPlaylist) return;

    try {
      await api.reorderPlaylistItem({
        playlist_id: selectedPlaylist.id,
        item_id: itemId,
        new_position: newPosition
      });
      message.success('排序更新成功');
      fetchPlaylistItems(selectedPlaylist.id);
    } catch (error) {
      console.error('排序更新失败:', error);
      message.error('排序更新失败');
    }
  };

  // 显示开播模态框
  const showLiveModal = () => {
    setIsLiveModalVisible(true);

    // 添加回车键监听
    setTimeout(() => {
      document.addEventListener('keydown', handleLiveModalKeyDown);
    }, 100);
  };

  // 关闭开播模态框
  const handleLiveCancel = () => {
    setIsLiveModalVisible(false);
    document.removeEventListener('keydown', handleLiveModalKeyDown);
  };

  // 开始直播
  const handleStartLive = async () => {
    if (!selectedArea) {
      message.error('请选择直播分区');
      return;
    }

    try {
      await api.startLive({ area_id: selectedArea });
      message.success('开播成功');
      setIsLiveModalVisible(false);
      fetchLiveStatus();
    } catch (error) {
      console.error('开播失败:', error);
      message.error('开播失败');
    }
  };

  // 停止直播
  const handleStopLive = async () => {
    try {
      await api.stopLive();
      message.success('停播成功');
      setRoomInfo(null);
    } catch (error) {
      console.error('停播失败:', error);
      message.error('停播失败');
    }
  };

  // 处理回车键确认的函数
  const handleEnterKeyPress = useCallback((e: KeyboardEvent, submitFn: () => void) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitFn();
    }
  }, []);

  // 播放列表模态框的键盘事件处理
  const handlePlaylistModalKeyDown = useCallback((e: KeyboardEvent) => {
    handleEnterKeyPress(e, handleSubmit);
  }, [handleEnterKeyPress, handleSubmit]);

  // 直播模态框的键盘事件处理
  const handleLiveModalKeyDown = useCallback((e: KeyboardEvent) => {
    handleEnterKeyPress(e, handleStartLive);
  }, [handleEnterKeyPress, handleStartLive]);

  // 播放列表表格列
  const playlistColumns = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: '30%',
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
      width: '40%',
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
      render: (_: any, record: Playlist) => (
        record.is_active ? <Tag color="green">活动中</Tag> : null
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: '40%',
      render: (_: any, record: Playlist) => (
        <Space size="small">
          <Button
            icon={<EditOutlined />}
            size="small"
            onClick={() => showModal('edit', record)}
          >
            编辑
          </Button>
          <Button
            icon={<DeleteOutlined />}
            size="small"
            danger
            onClick={() => handleDelete(record.id)}
          >
            删除
          </Button>
          <Button
            icon={<PlayCircleOutlined />}
            size="small"
            type="primary"
            onClick={() => handleViewItems(record)}
          >
            查看
          </Button>
          {record.is_active ? (
            <Button
              size="small"
              onClick={() => handleUnsetActive(record.id)}
            >
              取消活动
            </Button>
          ) : (
            <Button
              size="small"
              type="primary"
              ghost
              onClick={() => handleSetActive(record.id)}
            >
              设为活动
            </Button>
          )}
        </Space>
      ),
    },
  ];

  // 播放列表项目表格列
  const playlistItemColumns = [
    {
      title: '位置',
      dataIndex: 'position',
      key: 'position',
    },
    {
      title: '标题',
      dataIndex: 'clip_title',
      key: 'clip_title',
    },
    {
      title: 'VUP',
      dataIndex: 'clip_vup',
      key: 'clip_vup',
    },
    {
      title: '操��',
      key: 'action',
      render: (_: any, record: PlaylistItem) => (
        <Space size="small">
          <Button
            size="small"
            danger
            onClick={() => handleRemoveFromPlaylist(record.clip_uuid)}
          >
            移除
          </Button>
          <Button
            size="small"
            disabled={record.position === 1}
            onClick={() => handleReorderItem(record.id, record.position - 1)}
          >
            上移
          </Button>
          <Button
            size="small"
            disabled={record.position === playlistItems.length}
            onClick={() => handleReorderItem(record.id, record.position + 1)}
          >
            下移
          </Button>
        </Space>
      ),
    },
  ];

  // 可用切片表格列
  const availableClipsColumns = [
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
      title: '操作',
      key: 'action',
      render: (_: any, record: Clip) => (
        <Button
          size="small"
          type="primary"
          onClick={() => handleAddToPlaylist(record.uuid)}
        >
          添加到播放列表
        </Button>
      ),
    },
  ];

  // 构建Tabs项目
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

    // 仅当选择了播放列表时才添加第二个标签页
    if (selectedPlaylist) {
      items.push({
        key: "2",
        label: "播放列表内容",
        children: (
          <>
            <div style={{ marginBottom: '16px' }}>
              <Title level={4}>{selectedPlaylist.name}</Title>
              <Text type="secondary">{selectedPlaylist.description}</Text>
            </div>

            <Divider orientation="left">当前内容</Divider>
            <Table
              columns={playlistItemColumns}
              dataSource={playlistItems.map(item => ({ ...item, key: item.id }))}
              pagination={{ pageSize: 10 }}
            />

            <Divider orientation="left">可添加的切片</Divider>
            <Table
              columns={availableClipsColumns}
              dataSource={clips
                .filter(clip => !playlistItems.some(item => item.clip_uuid === clip.uuid))
                .map(clip => ({ ...clip, key: clip.uuid }))}
              pagination={{ pageSize: 5 }}
            />
          </>
        )
      });
    }

    return items;
  };

  return (
    <div>
      <Title level={2}>播放列表</Title>

      <div style={{ marginBottom: '16px', display: 'flex', justifyContent: 'space-between' }}>
        <Space>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => showModal('add')}
          >
            新建播放列表
          </Button>
        </Space>

        <Space>
          {roomInfo && roomInfo.live_status === 1 ? (
            <Button
              danger
              icon={<StopOutlined />}
              onClick={handleStopLive}
              loading={liveLoading}
            >
              停止直播
            </Button>
          ) : (
            <Button
              type="primary"
              icon={<VideoCameraOutlined />}
              onClick={showLiveModal}
              loading={liveLoading}
            >
              开始直播
            </Button>
          )}
        </Space>
      </div>

      {roomInfo && roomInfo.live_status === 1 && (
        <Card style={{ marginBottom: '16px', backgroundColor: '#f6ffed', borderColor: '#b7eb8f' }}>
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <VideoCameraOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
            <div>
              <Text strong style={{ fontSize: '16px' }}>直播中</Text>
              <div>直播间: {roomInfo.title}</div>
              <div>分区: {roomInfo.area_name}</div>
              <div>在线人数: {roomInfo.online}</div>
            </div>
          </div>
        </Card>
      )}

      <Tabs
        activeKey={activeTab}
        onChange={(key) => {
          setActiveTab(key);
          // 当切换标签页时，如果没有选中播放列表，则不允许切换到播放列表内容标签
          if (key === '2' && !selectedPlaylist) {
            return;
          }
        }}
        items={getTabItems()}
      />

      {/* 创建/编辑播放列表模态框 */}
      <Modal
        title={formMode === 'add' ? '创建播放列表' : '编辑播放列表'}
        open={isModalVisible}
        onOk={handleSubmit}
        onCancel={handleCancel}
      >
        <Form form={form} layout="vertical">
          <Form.Item
            name="name"
            label="播放列表名称"
            rules={[{ required: true, message: '请输入播放列表名称' }]}
          >
            <Input placeholder="请输入播放列表名称" />
          </Form.Item>

          <Form.Item name="description" label="描述">
            <TextArea rows={4} placeholder="请输入播放列表描述（可选）" />
          </Form.Item>
        </Form>
      </Modal>

      {/* 开始直播模态框 */}
      <Modal
        title="开始直播"
        open={isLiveModalVisible}
        onOk={handleStartLive}
        onCancel={handleLiveCancel}
      >
        {liveLoading ? (
          <div style={{ textAlign: 'center', padding: '20px' }}>
            <Spin />
            <div style={{ marginTop: '10px' }}>加载中...</div>
          </div>
        ) : (
          <Form layout="vertical">
            <Form.Item
              label="选择分区"
              required
            >
              <div style={{ display: 'flex', gap: '10px' }}>
                <Select
                  style={{ width: '50%' }}
                  placeholder="选择父分区"
                  value={selectedParentArea}
                  onChange={(value) => setSelectedParentArea(value)}
                >
                  {liveAreas.map(area => (
                    <Option key={area.id} value={area.id}>{area.name}</Option>
                  ))}
                </Select>

                <Select
                  style={{ width: '50%' }}
                  placeholder="选择子分区"
                  value={selectedArea}
                  onChange={(value) => setSelectedArea(value)}
                  disabled={!selectedParentArea}
                >
                  {selectedParentArea && liveAreas.find(area => area.id === selectedParentArea)?.list.map(subArea => (
                    <Option key={subArea.id} value={parseInt(subArea.id)}>{subArea.name}</Option>
                  ))}
                </Select>
              </div>
            </Form.Item>

            <div style={{ marginTop: '10px' }}>
              <Text type="secondary">
                注意: 开始直播前，请确保已在"播放列表"标签中设置了活动播放列表。
              </Text>
            </div>
          </Form>
        )}
      </Modal>
    </div>
  );
};

export default PlaylistsPage;
