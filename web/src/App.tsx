import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, message } from 'antd';
import zhCN from 'antd/lib/locale/zh_CN';
import { AppProvider, useAuth } from './context/AppContext';

// 配置全局消息提示
message.config({
  top: 24,
  duration: 2,
  maxCount: 3,
});

// 页面组件
import LoginPage from './pages/LoginPage';
import ClipsPage from './pages/ClipsPage';
import UploadPage from './pages/UploadPage';
import EditClipPage from './pages/EditClipPage';
import PlaylistsPage from './pages/PlaylistsPage';
import ProfilePage from './pages/ProfilePage';
import SettingsPage from './pages/SettingsPage';
import AdminPage from "./pages/AdminPage.tsx";

// 布局组件
import MainLayout from './components/MainLayout';
import AdminRoute from './components/AdminRoute/AdminRoute';
import SteamRoute from './components/StreamRoute/StreamRoute';

// 受保护的路由组件
const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <MainLayout>{children}</MainLayout>;
};

const App: React.FC = () => {
  return (
    <ConfigProvider locale={zhCN}>
      <AppProvider>
        <Router>
          <Routes>
            <Route path="/login" element={<LoginPage />} />

            <Route
              path="/clips"
              element={
                <ProtectedRoute>
                  <ClipsPage />
                </ProtectedRoute>
              }
            />

            <Route
              path="/upload"
              element={
                <ProtectedRoute>
                  <UploadPage />
                </ProtectedRoute>
              }
            />

            <Route
              path="/clip/edit/:uuid"
              element={
                <ProtectedRoute>
                  <EditClipPage />
                </ProtectedRoute>
              }
            />

            <Route
              path="/playlists"
              element={
                <ProtectedRoute>
                  <SteamRoute>
                    <PlaylistsPage />
                  </SteamRoute>
                </ProtectedRoute>
              }
            />

            <Route
              path="/profile"
              element={
                <ProtectedRoute>
                  <ProfilePage />
                </ProtectedRoute>
              }
            />

            <Route
              path="/settings"
              element={
                <ProtectedRoute>
                  <SettingsPage />
                </ProtectedRoute>
              }
            />

            <Route
              path="/admin"
              element={
                <ProtectedRoute>
                  <AdminRoute>
                    <AdminPage />
                  </AdminRoute>
                </ProtectedRoute>
              }
            />
            <Route path="/" element={<Navigate to="/clips" replace />} />
            <Route path="*" element={<Navigate to="/clips" replace />} />
          </Routes>
        </Router>
      </AppProvider>
    </ConfigProvider>
  );
};

export default App;
