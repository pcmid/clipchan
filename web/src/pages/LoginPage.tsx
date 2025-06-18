import React, { useState, useEffect, useCallback } from 'react';
import { Card, Typography, Spin, Button, Modal, Form, Input, message } from 'antd';
import { SettingOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useApi, useAuth, useConfig } from '../context/AppContext';
import type {QrCodeInfo} from '../types';

const { Title, Text } = Typography;

const LoginPage: React.FC = () => {
  const [qrCode, setQrCode] = useState<QrCodeInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [checking, setChecking] = useState(false);
  const [isSettingsModalVisible, setIsSettingsModalVisible] = useState(false);
  const [qrCodeError, setQrCodeError] = useState(false); // 添加二维码错误状态
  const [form] = Form.useForm();

  const api = useApi();
  const { login, isAuthenticated } = useAuth();
  const { config, updateConfig } = useConfig();
  const navigate = useNavigate();

  // 如果已经登录，则重定向到切片页面
  useEffect(() => {
    if (isAuthenticated) {
      navigate('/clips');
    }
  }, [isAuthenticated, navigate]);

  // 初始化表单值
  useEffect(() => {
    form.setFieldsValue({
      apiBaseUrl: config.apiBaseUrl,
    });
  }, [config, form]);

  // 获取登录二维码
  useEffect(() => {
    const fetchQrCode = async () => {
      try {
        setLoading(true);
        const data = await api.getLoginQrcode();
        setQrCode(data);
        setQrCodeError(false); // 重置二维码错误状态
      } catch (error) {
        console.error('获取二维码失败:', error);
        setQrCodeError(true); // 设置二维码错误状态
      } finally {
        setLoading(false);
      }
    };

    fetchQrCode();
  }, [api]);

  // 定期检查登录状态
  useEffect(() => {
    if (!qrCode || checking || qrCodeError) return; // 如果二维码有错误，不再进行检查

    const checkLoginStatus = async () => {
      try {
        setChecking(true);
        const loginInfo = await api.checkBilibiliLogin(qrCode.qrcode_key);
        if (loginInfo) {
          login(loginInfo);
          navigate('/clips');
        }
      } catch (error) {
        console.error('检查登录状态失败:', error);
        setQrCodeError(true); // 设置二维码错误状态
      } finally {
        setChecking(false);
      }
    };

    // 每3秒检查一次登录状态
    const intervalId = setInterval(checkLoginStatus, 3000);
    return () => clearInterval(intervalId);
  }, [qrCode, checking, qrCodeError, api, login, navigate]);

  // 显示设置模态框
  const showSettingsModal = () => {
    setIsSettingsModalVisible(true);

    // 添加回车键监听
    setTimeout(() => {
      document.addEventListener('keydown', handleSettingsModalKeyDown);
    }, 100);
  };

  // 关闭设置模态框
  const handleSettingsCancel = () => {
    setIsSettingsModalVisible(false);
    document.removeEventListener('keydown', handleSettingsModalKeyDown);
  };

  // 保存设置
  const handleSettingsSave = async () => {
    try {
      const values = await form.validateFields();

      updateConfig({
        apiBaseUrl: values.apiBaseUrl,
      });

      message.success('后端地址设置已保存');
      setIsSettingsModalVisible(false);
      document.removeEventListener('keydown', handleSettingsModalKeyDown);

      // 重新加载二维码
      setLoading(true);
      setQrCode(null);
      try {
        const data = await api.getLoginQrcode();
        setQrCode(data);
        setQrCodeError(false); // 重置二维码错误状态
      } catch (error) {
        console.error('获取二维码失败:', error);
        setQrCodeError(true); // 设置二维码错误状态
      } finally {
        setLoading(false);
      }
    } catch (error) {
      console.error('保存设置失败:', error);
    }
  };

  // 刷新二维码
  const refreshQrCode = async () => {
    try {
      setLoading(true);
      const data = await api.getLoginQrcode();
      setQrCode(data);
      setQrCodeError(false); // 重置二维码错误状态
    } catch (error) {
      console.error('获取二维码失败:', error);
      setQrCodeError(true); // 设置二维码错误状态
    } finally {
      setLoading(false);
    }
  };

  // 处理回车键确认的函数
  const handleEnterKeyPress = useCallback((e: KeyboardEvent, submitFn: () => void) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitFn();
    }
  }, []);

  // 设置模态框的键盘事件处理
  const handleSettingsModalKeyDown = useCallback((e: KeyboardEvent) => {
    handleEnterKeyPress(e, handleSettingsSave);
  }, [handleEnterKeyPress, handleSettingsSave]);

  return (
    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh', background: '#f0f2f5' }}>
      <Card style={{ width: 400, textAlign: 'center', padding: '20px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
          <Title level={2} style={{ margin: 0 }}>ClipChan</Title>
          <Button
            icon={<SettingOutlined />}
            type="text"
            onClick={showSettingsModal}
            title="设置后端地址"
          />
        </div>

        <Title level={4}>哔哩哔哩账号登录</Title>

        {loading ? (
          <div style={{ padding: '40px 0' }}>
            <Spin size="large" />
            <Text style={{ display: 'block', marginTop: 16 }}>正在加载二维码...</Text>
          </div>
        ) : qrCode ? (
          <div style={{ position: 'relative' }}>
            <div
              dangerouslySetInnerHTML={{ __html: qrCode.svg }}
            />
            {qrCodeError && (
              <div style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'center',
                alignItems: 'center',
                borderRadius: '4px'
              }}>
                <Text style={{ color: '#fff', marginBottom: '10px' }}>二维码已失效</Text>
                <Button
                  type="primary"
                  icon={<ReloadOutlined />}
                  onClick={refreshQrCode}
                >
                  刷新二维码
                </Button>
              </div>
            )}
            <Text type="secondary" style={{ display: 'block', marginTop: '10px' }}>
              请使用哔哩哔哩APP扫描二维码登录
            </Text>
          </div>
        ) : (
          <div style={{ padding: '20px 0' }}>
            <Text type="danger" style={{ display: 'block', marginBottom: '16px' }}>获取二维码失败</Text>
            <Button
              type="primary"
              icon={<ReloadOutlined />}
              onClick={refreshQrCode}
            >
              重新获取二维码
            </Button>
          </div>
        )}
      </Card>

      {/* 设置模态框 */}
      <Modal
        title="设置后端服务地址"
        open={isSettingsModalVisible}
        onOk={handleSettingsSave}
        onCancel={handleSettingsCancel}
      >
        <Form
          form={form}
          layout="vertical"
        >
          <Form.Item
            name="apiBaseUrl"
            label="后端API地址"
            rules={[
              { required: true, message: '请输入后端API地址' },
              { type: 'string', message: '请输入有效的URL' }
            ]}
            extra="设置后端服务器的API基础URL，例如: http://localhost:3000"
          >
            <Input placeholder="请输入后端API地址" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default LoginPage;
