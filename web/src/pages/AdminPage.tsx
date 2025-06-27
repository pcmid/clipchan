import React, { useState, useEffect } from 'react';
import { message } from 'antd';
import { useAuth, useApi } from '../context/AppContext';
import type { User, UpdateUserPermissionsRequest } from '../types';
import PageContainer from '../components/PageContainer/PageContainer';
import ContentCard from '../components/ContentCard/ContentCard';
import './AdminPage.css';

const AdminPage: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [updating, setUpdating] = useState<Set<number>>(new Set());
  const { currentUser } = useAuth();
  const api = useApi();

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      setLoading(true);
      setError(null);
      const usersData = await api.getAllUsers();
      setUsers(usersData);
    } catch (error) {
      console.error('获取用户列表失败:', error);
      setError('获取用户列表失败');
      message.error('获取用户列表失败');
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = () => {
    loadUsers();
  };

  const handleClearError = () => {
    setError(null);
  };

  const handlePermissionToggle = async (user: User, permission: 'is_admin' | 'can_stream' | 'is_disabled') => {
    // 防止用户修改自己的管理员权限或禁用自己
    if (currentUser && user.id === currentUser.id && user.is_admin) {
      if (permission === 'is_admin' || permission === 'is_disabled') {
        message.warning('不能修改自己的管理员权限或禁用自己');
        return;
      }
    }

    try {
      setUpdating(prev => new Set(prev).add(user.id));

      const updateData: UpdateUserPermissionsRequest = {
        [permission]: !user[permission]
      };

      await api.updateUserPermissions(user.id, updateData);

      // 更新本地状态
      setUsers(prev => prev.map(u =>
        u.id === user.id
          ? { ...u, [permission]: !u[permission] }
          : u
      ));

      message.success(`${getPermissionName(permission)}更新成功`);
    } catch (error) {
      console.error(`更新${getPermissionName(permission)}失败:`, error);
      message.error(`更新${getPermissionName(permission)}失败`);
    } finally {
      setUpdating(prev => {
        const newSet = new Set(prev);
        newSet.delete(user.id);
        return newSet;
      });
    }
  };

  const getPermissionName = (permission: string) => {
    switch (permission) {
      case 'is_admin':
        return '管理员权限';
      case 'can_stream':
        return '开播权限';
      case 'is_disabled':
        return '禁用状态';
      default:
        return '权限';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString('zh-CN');
  };

  const getStatusBadge = (user: User) => {
    if (user.is_disabled) {
      return <span className="badge badge-disabled">已禁用</span>;
    }
    if (user.is_admin) {
      return <span className="badge badge-admin">管理员</span>;
    }
    if (!user.can_stream) {
      return <span className="badge badge-no-stream">禁止开播</span>;
    }
    return <span className="badge badge-normal">正常</span>;
  };

  const refreshButton = (
    <button
      className="action-btn"
      onClick={handleRefresh}
      disabled={loading}
    >
      刷新
    </button>
  );

  return (
    <PageContainer title="用户管理" extra={refreshButton}>
      <ContentCard loading={loading}>
        {!loading && (
          <div className="admin-panel">
            {error && (
              <div className="error-banner">
                {error}
                <button onClick={handleClearError}>×</button>
              </div>
            )}

            <div className="users-table-container">
              <table className="users-table">
                <thead>
                  <tr>
                    <th>用户ID</th>
                    <th>B站UID</th>
                    <th>用户名</th>
                    <th>状态</th>
                    <th>管理员</th>
                    <th>开播权限</th>
                    <th>禁用状态</th>
                    <th>注册时间</th>
                    <th>最后更新</th>
                  </tr>
                </thead>
                <tbody>
                  {users.map(user => (
                    <tr key={user.id} className={user.is_disabled ? 'user-disabled' : ''}>
                      <td>{user.id}</td>
                      <td>{user.mid}</td>
                      <td>
                        {user.uname}
                        {currentUser && user.id === currentUser.id && (
                          <span className="current-user-label">(当前用户)</span>
                        )}
                      </td>
                      <td>{getStatusBadge(user)}</td>
                      <td>
                        <label className="toggle-switch">
                          <input
                            type="checkbox"
                            checked={user.is_admin}
                            disabled={
                              updating.has(user.id) ||
                              (currentUser && user.id === currentUser.id && user.is_admin) || false
                            }
                            onChange={() => handlePermissionToggle(user, 'is_admin')}
                          />
                          <span className="toggle-slider"></span>
                        </label>
                      </td>
                      <td>
                        <label className="toggle-switch">
                          <input
                            type="checkbox"
                            checked={user.can_stream}
                            disabled={updating.has(user.id)}
                            onChange={() => handlePermissionToggle(user, 'can_stream')}
                          />
                          <span className="toggle-slider"></span>
                        </label>
                      </td>
                      <td>
                        <label className="toggle-switch">
                          <input
                            type="checkbox"
                            checked={user.is_disabled}
                            disabled={
                                updating.has(user.id) ||
                                (currentUser && user.id === currentUser.id && user.is_admin) || false
                            }
                            onChange={() => handlePermissionToggle(user, 'is_disabled')}
                          />
                          <span className="toggle-slider"></span>
                        </label>
                      </td>
                      <td>{formatDate(user.created_at)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {users.length === 0 && (
              <div className="empty-state">
                <p>暂无用户数据</p>
              </div>
            )}
          </div>
        )}
      </ContentCard>
    </PageContainer>
  );
};

export default AdminPage;
