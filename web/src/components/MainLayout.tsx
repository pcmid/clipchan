import React from 'react';
import { Layout, Menu, Button } from 'antd';
import { Link, useLocation } from 'react-router-dom';
import {
  VideoCameraOutlined,
  UploadOutlined,
  PlaySquareOutlined,
  UserOutlined,
  SettingOutlined,
  LogoutOutlined
} from '@ant-design/icons';
import { useAuth } from '../context/AppContext';
import type { MenuProps } from 'antd';

const { Header, Sider, Content } = Layout;

interface MainLayoutProps {
  children: React.ReactNode;
}

const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  const location = useLocation();
  const { isAuthenticated, logout } = useAuth();

  // 使用items属性定义菜单项
  const menuItems: MenuProps['items'] = [
    {
      key: '/clips',
      icon: <VideoCameraOutlined />,
      label: <Link to="/clips">我的切片</Link>,
    },
    {
      key: '/upload',
      icon: <UploadOutlined />,
      label: <Link to="/upload">上传切片</Link>,
    },
    {
      key: '/playlists',
      icon: <PlaySquareOutlined />,
      label: <Link to="/playlists">播放列表</Link>,
    },
    {
      key: '/profile',
      icon: <UserOutlined />,
      label: <Link to="/profile">个人信息</Link>,
    },
    {
      key: '/settings',
      icon: <SettingOutlined />,
      label: <Link to="/settings">系统设置</Link>,
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 16px' }}>
        <div style={{ color: 'white', fontSize: '1.5rem', fontWeight: 'bold' }}>ClipChan</div>
        {isAuthenticated && (
          <Button
            icon={<LogoutOutlined />}
            type="link"
            style={{ color: 'white' }}
            onClick={logout}
          >
            退出登录
          </Button>
        )}
      </Header>
      <Layout>
        {isAuthenticated && (
          <Sider width={200} theme="light">
            <Menu
              mode="inline"
              selectedKeys={[location.pathname]}
              style={{ height: '100%', borderRight: 0 }}
              items={menuItems}
            />
          </Sider>
        )}
        <Layout style={{ padding: '24px' }}>
          <Content
            style={{
              padding: 24,
              margin: 0,
              minHeight: 280,
              background: '#fff',
              borderRadius: '4px',
            }}
          >
            {children}
          </Content>
        </Layout>
      </Layout>
    </Layout>
  );
};

export default MainLayout;
