import React from 'react';
import './ContentCard.css';

interface ContentCardProps {
  children: React.ReactNode;
  title?: string;
  extra?: React.ReactNode;
  className?: string;
  loading?: boolean;
}

const ContentCard: React.FC<ContentCardProps> = ({
  children,
  title,
  extra,
  className = '',
  loading = false
}) => {
  return (
    <div className={`content-card ${className}`}>
      {title && (
        <div className="card-header">
          <h3 className="card-title">{title}</h3>
          {extra && <div className="card-extra">{extra}</div>}
        </div>
      )}
      <div className="card-content">
        {loading ? (
          <div className="card-loading">
            <div className="loading-spinner"></div>
            <span>加载中...</span>
          </div>
        ) : (
          children
        )}
      </div>
    </div>
  );
};

export default ContentCard;
