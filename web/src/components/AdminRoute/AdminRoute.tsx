import React, { useState, useEffect } from 'react';
import { useApi } from '../../context/AppContext';
interface AdminRouteProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

const AdminRoute: React.FC<AdminRouteProps> = ({
  children,
  fallback = <div className="access-denied">您没有管理员权限</div>
}) => {
  const [isAdmin, setIsAdmin] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const api = useApi();

  useEffect(() => {
    checkAdminPermission();
  }, []);

  const checkAdminPermission = async () => {
    try {
      const adminStatus = await api.getCurrentUser().then(user => user.is_admin);
      setIsAdmin(adminStatus);
    } catch {
      setIsAdmin(false);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="loading">检查权限中...</div>;
  }

  return isAdmin ? <>{children}</> : <>{fallback}</>;
};

export default AdminRoute;
