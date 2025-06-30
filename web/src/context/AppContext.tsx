import React, { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import ApiService from '../services/api';
import type { Account, AppConfig, LoginInfo, User } from '../types';

// 默认配置
const defaultConfig: AppConfig = {
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL,
  defaultApiBaseUrl: import.meta.env.VITE_API_BASE_URL,
};

interface AuthContextValue {
  isAuthenticated: boolean;
  isAdmin: boolean;
  canStream: boolean;
  isDisabled: boolean;
  account: Account | null;
  currentUser: User | null;
  login: (loginInfo: LoginInfo) => void;
  logout: () => void;
  refreshUserInfo: () => Promise<void>;
}

interface ConfigContextValue {
  config: AppConfig;
  updateConfig: (newConfig: Partial<AppConfig>) => void;
}

interface AppContextValue {
  auth: AuthContextValue;
  config: ConfigContextValue;
  api: ApiService;
}

const AppContext = createContext<AppContextValue | undefined>(undefined);

export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  // 从本地存储加载配置
  const loadConfig = (): AppConfig => {
    const storedConfig = localStorage.getItem('appConfig');
    const config = storedConfig ? JSON.parse(storedConfig) : defaultConfig;
    config.defaultApiBaseUrl = import.meta.env.VITE_API_BASE_URL; // 确保默认API基础URL正确
    return config
  };

  // 从本地存储加载用户信息
  const loadUserInfo = (): { account: Account | null; token: string | null } => {
    const storedAccount = localStorage.getItem('account');
    const storedToken = localStorage.getItem('token');
    return {
      account: storedAccount ? JSON.parse(storedAccount) : null,
      token: storedToken,
    };
  };

  const [config, setConfig] = useState<AppConfig>(loadConfig());
  const [account, setAccount] = useState<Account | null>(loadUserInfo().account);
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [api] = useState<ApiService>(new ApiService(config.apiBaseUrl));

  // 获取当前用户详细信息
  const refreshUserInfo = async () => {
    if (!account) {
      setCurrentUser(null);
      return;
    }

    try {
      const userInfo = await api.getCurrentUser();
      setCurrentUser(userInfo);
    } catch (error) {
      console.error('获取用户信息失败:', error);
      // 如果获取用户信息失败，可能是token过期，执行登出
      logout();
    }
  };

  // 初始化时设置令牌并获取用户信息
  useEffect(() => {
    const { token } = loadUserInfo();
    if (token) {
      api.setToken(token);
      // 如果有token且有account，获取详细用户信息
      if (account) {
        refreshUserInfo();
      }
    }
  }, [api, account]);

  // 更新配置
  const updateConfig = (newConfig: Partial<AppConfig>) => {
    const updatedConfig = { ...config, ...newConfig };
    setConfig(updatedConfig);
    localStorage.setItem('appConfig', JSON.stringify(updatedConfig));

    // 如果API基础URL更改，则更新API服务
    if (newConfig.apiBaseUrl) {
      api.setBaseURL(newConfig.apiBaseUrl);
    }
  };

  // 登录
  const login = (loginInfo: LoginInfo) => {
    setAccount(loginInfo.account);
    api.setToken(loginInfo.token);
    localStorage.setItem('account', JSON.stringify(loginInfo.account));
    localStorage.setItem('token', loginInfo.token);
    // 登录后立即获取用户详细信息
    setTimeout(() => {
      refreshUserInfo();
    }, 100);
  };

  // 登出
  const logout = () => {
    setAccount(null);
    setCurrentUser(null);
    api.clearToken();
    localStorage.removeItem('account');
    localStorage.removeItem('token');
  };

  const authContextValue: AuthContextValue = {
    isAuthenticated: !!account,
    isAdmin: currentUser?.is_admin || false,
    canStream: currentUser?.can_stream || false,
    isDisabled: currentUser?.is_disabled || false,
    account,
    currentUser,
    login,
    logout,
    refreshUserInfo,
  };

  const configContextValue: ConfigContextValue = {
    config,
    updateConfig,
  };

  const appContextValue: AppContextValue = {
    auth: authContextValue,
    config: configContextValue,
    api,
  };

  return <AppContext.Provider value={appContextValue}>{children}</AppContext.Provider>;
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
};

export const useAuth = () => {
  return useApp().auth;
};

export const useConfig = () => {
  return useApp().config;
};

export const useApi = () => {
  return useApp().api;
};
