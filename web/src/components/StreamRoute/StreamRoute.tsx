import React, { useState, useEffect } from 'react';
import { useApi } from '../../context/AppContext';
import './StreamRoute.css';

interface StreamRouteProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

const StreamRoute: React.FC<StreamRouteProps> = ({
  children,
  fallback = <div className="access-denied">您没有开播权限</div>
}) => {
  const [canStream, setCanStream] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const api = useApi();

  useEffect(() => {
    checkStreamPermission();
  }, []);

  const checkStreamPermission = async () => {
    try {
      const user = await api.getCurrentUser();
      setCanStream(user.can_stream && !user.is_disabled);
    } catch {
      setCanStream(false);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="loading">检查权限中...</div>;
  }

  return canStream ? <>{children}</> : <>{fallback}</>;
};

export default StreamRoute;
