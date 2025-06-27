import React from 'react';
import './PageContainer.css';

interface PageContainerProps {
  children: React.ReactNode;
  title?: string;
  extra?: React.ReactNode;
  className?: string;
}

const PageContainer: React.FC<PageContainerProps> = ({
  children,
  title,
  extra,
  className = ''
}) => {
  return (
    <div className={`page-container ${className}`}>
      {title && (
        <div className="page-header">
          <h1 className="page-title">{title}</h1>
          {extra && <div className="page-extra">{extra}</div>}
        </div>
      )}
      <div className="page-content">
        {children}
      </div>
    </div>
  );
};

export default PageContainer;
