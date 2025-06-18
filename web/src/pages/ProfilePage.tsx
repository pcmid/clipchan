import React from 'react';
import { Card, Typography, Descriptions, Avatar, Tag } from 'antd';
import { UserOutlined } from '@ant-design/icons';
import { useAuth } from '../context/AppContext';

const { Title } = Typography;

const ProfilePage: React.FC = () => {
  const { account } = useAuth();

  if (!account) {
    return <div>未找到用户信息</div>;
  }

  return (
    <div>
      <Title level={2}>个人信息</Title>
      <Card style={{ maxWidth: '800px' }}>
        <div style={{ display: 'flex', alignItems: 'center', marginBottom: '24px' }}>
          <Avatar size={64} icon={<UserOutlined />} />
          <div style={{ marginLeft: '16px' }}>
            <Title level={3} style={{ margin: 0 }}>{account.uname}</Title>
            <Tag color="blue">UID: {account.mid}</Tag>
          </div>
        </div>

        <Descriptions bordered column={1}>
          <Descriptions.Item label="用户名">{account.uname}</Descriptions.Item>
          <Descriptions.Item label="UID">{account.mid}</Descriptions.Item>
          <Descriptions.Item label="签名">{account.sign || '暂无签名'}</Descriptions.Item>
          <Descriptions.Item label="性别">{account.sex || '未设置'}</Descriptions.Item>
          <Descriptions.Item label="生日">{account.birthday || '未设置'}</Descriptions.Item>
        </Descriptions>
      </Card>
    </div>
  );
};

export default ProfilePage;
