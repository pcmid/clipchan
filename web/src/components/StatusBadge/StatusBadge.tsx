import React from 'react';
import './StatusBadge.css';

interface StatusBadgeProps {
  status: 'success' | 'warning' | 'error' | 'info' | 'processing';
  children: React.ReactNode;
  className?: string;
}

const StatusBadge: React.FC<StatusBadgeProps> = ({ status, children, className = '' }) => {
  return (
    <span className={`status-badge status-${status} ${className}`}>
      {children}
    </span>
  );
};

export default StatusBadge;
