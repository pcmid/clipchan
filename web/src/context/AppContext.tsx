import React, { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import ApiService from '../services/api';
import type { Account, AppConfig, LoginInfo } from '../types';

// 默认配置
const defaultConfig: AppConfig = {
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL,
};

interface AuthContextValue {
  isAuthenticated: boolean;
  account: Account | null;
  login: (loginInfo: LoginInfo) => void;
  logout: () => void;
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
    return storedConfig ? JSON.parse(storedConfig) : defaultConfig;
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
  const [api] = useState<ApiService>(new ApiService(config.apiBaseUrl));

  // 初始化时设置令牌
  useEffect(() => {
    const { token } = loadUserInfo();
    if (token) {
      api.setToken(token);
    }
  }, [api]);

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
  };

  // 登出
  const logout = () => {
    setAccount(null);
    api.clearToken();
    localStorage.removeItem('account');
    localStorage.removeItem('token');
  };

  const authContextValue: AuthContextValue = {
    isAuthenticated: !!account,
    account,
    login,
    logout,
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
