import axios, {type AxiosInstance } from 'axios';
import type {
    QrCodeInfo,
    LoginInfo,
    Clip,
    ClipRequest,
    Playlist,
    PlaylistRequest,
    PlaylistItem,
    PlaylistItemRequest,
    ReorderItemRequest,
    LiveArea,
    StartLiveRequest,
    RoomInfo,
    ServerConfig
} from '../types';

class ApiService {
  private api: AxiosInstance;
  private token: string | null = null;

  constructor(baseURL: string) {
    this.api = axios.create({
      baseURL,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 10000, // 添加10秒超时设置
    });

    // 添加请求拦截器，自动添加token
    this.api.interceptors.request.use((config) => {
      if (this.token) {
        config.headers['Authorization'] = `Bearer ${this.token}`;
      }
      return config;
    });

    // 添加响应拦截器，处理401错误
    this.api.interceptors.response.use(
      (response) => {
        return response;
      },
      (error) => {
        if (error.response && error.response.status === 401) {
          this.clearToken();
          const currentPath = window.location.pathname;
          sessionStorage.setItem('redirectPath', currentPath);
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }


  setBaseURL(url: string): void {
    this.api.defaults.baseURL = url;
  }

  setToken(token: string): void {
    this.token = token;
  }

  clearToken(): void {
    this.token = null;
  }

  // 配置相关API
  async getServerConfig(): Promise<ServerConfig> {
    const response = await this.api.get('/configs');
    return response.data;
  }

  // 用户相关API
  async getLoginQrcode(): Promise<QrCodeInfo> {
    const response = await this.api.get('/user/login/qrcode');
    return response.data;
  }

  async checkBilibiliLogin(qrcodeKey: string): Promise<LoginInfo | null> {
    const response = await this.api.get(`/user/login/check?qrcode_key=${qrcodeKey}`);
    if (response.data === 'NotLoggedIn') {
      return null;
    }
    return response.data.LoggedIn;
  }

  // 切片��关API
  async listClips(): Promise<Clip[]> {
    const response = await this.api.get('/clips');
    return response.data;
  }

  async uploadClip(file: File, metadata: ClipRequest): Promise<Clip> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('metadata', JSON.stringify(metadata));

    const response = await this.api.post('/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    return response.data;
  }

  async updateClip(uuid: string, data: ClipRequest): Promise<Clip> {
    const response = await this.api.post(`/clip/${uuid}`, data);
    return response.data;
  }

  async reviewedClip(uuid: string): Promise<Clip> {
    const response = await this.api.post(`/clip/${uuid}/reviewed`);
    return response.data;
  }

  async deleteClip(uuid: string): Promise<void> {
    await this.api.delete(`/clip/${uuid}`);
  }

  // 播放列表相关API
  async listPlaylists(): Promise<Playlist[]> {
    const response = await this.api.get('/playlists');
    return response.data;
  }

  async createPlaylist(data: PlaylistRequest): Promise<Playlist> {
    const response = await this.api.post('/playlists', data);
    return response.data;
  }

  async getActivePlaylist(): Promise<Playlist> {
    const response = await this.api.get('/playlists/active');
    return response.data;
  }

  async getPlaylistById(id: number): Promise<Playlist> {
    const response = await this.api.get(`/playlists/${id}`);
    return response.data;
  }

  async updatePlaylist(id: number, data: PlaylistRequest): Promise<Playlist> {
    const response = await this.api.post(`/playlists/${id}`, data);
    return response.data;
  }

  async deletePlaylist(id: number): Promise<void> {
    await this.api.delete(`/playlists/${id}`);
  }

  async setActivePlaylist(id: number): Promise<void> {
    await this.api.post(`/playlists/${id}/active`);
  }

  async unsetActivePlaylist(id: number): Promise<void> {
    await this.api.delete(`/playlists/${id}/active`);
  }

  async getPlaylistItems(id: number): Promise<PlaylistItem[]> {
    const response = await this.api.get(`/playlists/${id}/items`);
    return response.data;
  }

  async addClipToPlaylist(data: PlaylistItemRequest): Promise<void> {
    await this.api.post('/playlists/items/add', data);
  }

  async removeClipFromPlaylist(data: PlaylistItemRequest): Promise<void> {
    await this.api.delete('/playlists/items/remove', { data });
  }

  async reorderPlaylistItem(data: ReorderItemRequest): Promise<void> {
    await this.api.post('/playlists/items/reorder', data);
  }

  // 直播相关API
  async getLiveAreas(): Promise<LiveArea[]> {
    const response = await this.api.get('/live/areas');
    return response.data;
  }

  async startLive(data: StartLiveRequest): Promise<void> {
    await this.api.post('/live/start', data);
  }

  async stopLive(): Promise<void> {
    await this.api.post('/live/stop');
  }

  async getLiveStatus(): Promise<RoomInfo> {
    const response = await this.api.get('/live/status');
    return response.data;
  }
}

export default ApiService;
