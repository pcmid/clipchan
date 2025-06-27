import React, { useState, useEffect } from 'react';
import type {User} from '../../types';
import { useApi } from '../../context/AppContext';
import './UserStatus.css';

interface UserStatusProps {
  showFullInfo?: boolean;
}

const UserStatus: React.FC<UserStatusProps> = ({ showFullInfo = false }) => {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const api = useApi();

  useEffect(() => {
    loadUserInfo();
  }, []);

  const loadUserInfo = async () => {
    try {
      setLoading(true);
      setError(null);
      const userData = await api.getCurrentUser();
      setUser(userData);
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取用户信息失败');
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="user-status loading">加载中...</div>;
  }

  if (error || !user) {
    return (
      <div className="user-status error">
        <span>用户信息获取失败</span>
        <button onClick={loadUserInfo} className="retry-btn">重试</button>
      </div>
    );
  }

  const getStatusIcon = () => {
    if (user.is_disabled) return '🚫';
    if (user.is_admin) return '👑';
    if (!user.can_stream) return '📹';
    return '✅';
  };

  const getStatusText = () => {
    if (user.is_disabled) return '账户已禁用';
    if (user.is_admin) return '管理员';
    if (!user.can_stream) return '禁止开播';
    return '正常';
  };

  return (
    <div className={`user-status ${user.is_disabled ? 'disabled' : ''}`}>
      <div className="user-info">
        <span className="status-icon">{getStatusIcon()}</span>
        <span className="username">{user.uname}</span>
        <span className="status-text">{getStatusText()}</span>
      </div>

      {showFullInfo && (
        <div className="user-details">
          <div className="detail-item">
            <span className="label">用户ID:</span>
            <span className="value">{user.id}</span>
          </div>
          <div className="detail-item">
            <span className="label">B站UID:</span>
            <span className="value">{user.mid}</span>
          </div>
          <div className="detail-item">
            <span className="label">管理员权限:</span>
            <span className={`value ${user.is_admin ? 'enabled' : 'disabled'}`}>
              {user.is_admin ? '是' : '否'}
            </span>
          </div>
          <div className="detail-item">
            <span className="label">开播权限:</span>
            <span className={`value ${user.can_stream ? 'enabled' : 'disabled'}`}>
              {user.can_stream ? '允许' : '禁止'}
            </span>
          </div>
          <div className="detail-item">
            <span className="label">账户状态:</span>
            <span className={`value ${user.is_disabled ? 'disabled' : 'enabled'}`}>
              {user.is_disabled ? '已禁用' : '正常'}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default UserStatus;
