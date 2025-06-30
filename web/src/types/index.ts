// 用户相关类型
export interface Account {
  mid: number;
  uname: string;
  userid: string;
  sign: string;
  birthday: string;
  sex: string;
  nick_free: boolean;
  rank: string;
}

export interface LoginInfo {
  account: Account;
  token: string;
}

export interface QrCodeInfo {
  qrcode_key: string;
  svg: string;
}

// 用户权限相关类型
export interface User {
  id: number;
  mid: number;
  uname: string;
  is_admin: boolean;
  can_stream: boolean;
  is_disabled: boolean;
  created_at: string;
}

export interface UpdateUserPermissionsRequest {
  is_admin?: boolean;
  can_stream?: boolean;
  is_disabled?: boolean;
}

// 切片相关类型
export interface Clip {
  uuid: string;
  title: string;
  vup: string;
  song: string;
  upload_time: number;
  status: string;
}

export interface ClipRequest {
  title: string;
  vup: string;
  song: string;
}

// 播放列表相关类型
export interface Playlist {
  id: number;
  name: string;
  description: string;
  is_active: boolean;
  created_at: number;
  updated_at: number;
  item_count: number;
}

export interface PlaylistRequest {
  name: string;
  description?: string;
  is_active?: boolean;
}

export interface PlaylistItem {
  id: number;
  playlist_id: number;
  clip_uuid: string;
  position: number;
  clip_title: string;
  clip_vup: string;
}

export interface PlaylistItemRequest {
  playlist_id: number;
  clip_uuid: string;
}

export interface ReorderItemRequest {
  playlist_id: number;
  item_id: number;
  new_position: number;
}

// 直播相关类型
export interface LiveArea {
  id: number;
  name: string;
  list: SubLiveArea[];
}

export interface SubLiveArea {
  id: string;
  parent_id: string;
  old_area_id: string;
  name: string;
  act_id: string;
  pk_status: string;
  hot_status: number;
  lock_status: string;
  pic: string;
  parent_name: string;
  area_type: number;
}

export interface StartLiveRequest {
  area_id: number;
}

export interface RoomInfo {
  area_id: number;
  area_name: string;
  attention: number;
  description: string;
  live_status: number;
  online: number;
  room_id: number;
  title: string;
  user_cover: string;
  [key: string]: any;
}

// 应用配置类型
export interface AppConfig {
  apiBaseUrl: string;
  defaultApiBaseUrl: string;
}

// 服务器配置类型
export interface ServerConfig {
  max_file_size: number;
}
