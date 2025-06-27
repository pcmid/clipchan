import React, { useState, useEffect } from 'react';
import { useAuth, useApi } from '../context/AppContext';
import { message } from 'antd';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './ProfilePage.css';

const ProfilePage: React.FC = () => {
  const { currentUser, logout } = useAuth();
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState({
    totalClips: 0,
    reviewedClips: 0,
    totalPlaylists: 0
  });
  const api = useApi();

  useEffect(() => {
    fetchUserStats();
  }, []);

  const fetchUserStats = async () => {
    try {
      setLoading(true);
      const [clips, playlists] = await Promise.all([
        api.listClips(),
        api.listPlaylists()
      ]);

      setStats({
        totalClips: clips.length,
        reviewedClips: clips.filter(clip => clip.status === 'reviewed').length,
        totalPlaylists: playlists.length
      });
    } catch (error) {
      console.error('获取用户统计失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleLogout = () => {
    logout();
    message.success('已退出登录');
  };

  // const formatDate = (dateString: string) => {
  //   return new Date(dateString).toLocaleString('zh-CN');
  // };

  const getStatusBadge = () => {
    if (!currentUser) return null;

    if (currentUser.is_disabled) {
      return <span className="status-badge disabled">已禁用</span>;
    }
    if (currentUser.is_admin) {
      return <span className="status-badge admin">管理员</span>;
    }
    if (!currentUser.can_stream) {
      return <span className="status-badge no-stream">禁止开播</span>;
    }
    return <span className="status-badge normal">正常</span>;
  };

  return (
    <PageContainer title="个人资料">
      <ContentCard loading={loading}>
        {currentUser && (
          <div className="profile-content">
            {/* 用户基本信息 */}
            <div className="profile-section">
              <h3 className="section-title">👤 基本信息</h3>
              <div className="profile-grid">
                <div className="profile-item">
                  <span className="profile-label">用户ID:</span>
                  <span className="profile-value">{currentUser.id}</span>
                </div>
                <div className="profile-item">
                  <span className="profile-label">B站UID:</span>
                  <span className="profile-value">{currentUser.mid}</span>
                </div>
                <div className="profile-item">
                  <span className="profile-label">用户名:</span>
                  <span className="profile-value">{currentUser.uname}</span>
                </div>
                <div className="profile-item">
                  <span className="profile-label">账户状态:</span>
                  <span className="profile-value">{getStatusBadge()}</span>
                </div>
                {/*<div className="profile-item">*/}
                {/*  <span className="profile-label">注册时间:</span>*/}
                {/*  <span className="profile-value">{formatDate(currentUser.created_at)}</span>*/}
                {/*</div>*/}
              </div>
            </div>

            {/* 权限信息 */}
            <div className="profile-section">
              <h3 className="section-title">🔐 权限信息</h3>
              <div className="permissions-grid">
                {currentUser.is_admin && (
                <div className="permission-item">
                  <div className="permission-icon">
                    {currentUser.is_admin ? '✅' : '❌'}
                  </div>
                  <div className="permission-content">
                    <div className="permission-title">管理员权限</div>
                    <div className="permission-desc">
                      {currentUser.is_admin ? '拥有系统管理权限' : '普通用户权限'}
                    </div>
                  </div>
                </div>)}
                <div className="permission-item">
                  <div className="permission-icon">
                    {currentUser.can_stream ? '✅' : '❌'}
                  </div>
                  <div className="permission-content">
                    <div className="permission-title">直播权限</div>
                    <div className="permission-desc">
                      {currentUser.can_stream ? '可以开启直播' : '无法开启直播'}
                    </div>
                  </div>
                </div>
                <div className="permission-item">
                  <div className="permission-icon">
                    {!currentUser.is_disabled ? '✅' : '❌'}
                  </div>
                  <div className="permission-content">
                    <div className="permission-title">账户状态</div>
                    <div className="permission-desc">
                      {!currentUser.is_disabled ? '账户正常' : '账户已被禁用'}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* 统计信息 */}
            <div className="profile-section">
              <h3 className="section-title">📊 使用统计</h3>
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-icon">🎬</div>
                  <div className="stat-number">{stats.totalClips}</div>
                  <div className="stat-label">总切片数</div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon">✅</div>
                  <div className="stat-number">{stats.reviewedClips}</div>
                  <div className="stat-label">已审核切片</div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon">📋</div>
                  <div className="stat-number">{stats.totalPlaylists}</div>
                  <div className="stat-label">播放列表数</div>
                </div>
              </div>
            </div>

            {/* 操作按钮 */}
            <div className="profile-actions">
              <button
                className="logout-btn"
                onClick={handleLogout}
              >
                🚪 退出登录
              </button>
            </div>
          </div>
        )}
      </ContentCard>
    </PageContainer>
  );
};

export default ProfilePage;
