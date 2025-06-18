use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod master {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MasterInfo {
        pub exp: Exp,
        pub follower_num: i64,
        pub glory_count: i64,
        pub info: Info,
        pub link_group_num: i64,
        pub medal_name: String,
        pub pendant: String,
        pub room_id: u64,
        pub room_news: RoomNews,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Exp {
        pub master_level: MasterLevel,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MasterLevel {
        pub color: i64,
        pub current: Vec<i64>,
        pub level: i64,
        pub next: Vec<i64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Info {
        pub face: String,
        pub gender: i64,
        pub official_verify: OfficialVerify,
        pub uid: i64,
        pub uname: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OfficialVerify {
        pub desc: String,
        #[serde(rename = "type")]
        pub r#type: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoomNews {
        pub content: String,
        pub ctime: String,
        pub ctime_text: String,
    }
}

pub mod room {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoomInfo {
        pub allow_change_area_time: i64,
        pub allow_upload_cover_time: i64,
        pub area_id: i64,
        pub area_name: String,
        pub area_pendants: String,
        pub attention: i64,
        pub background: String,
        pub battle_id: i64,
        pub description: String,
        pub hot_words: Vec<String>,
        pub hot_words_status: i64,
        pub is_anchor: i64,
        pub is_portrait: bool,
        pub is_strict_room: bool,
        pub keyframe: String,
        pub live_status: i64,
        pub live_time: String,
        pub new_pendants: NewPendants,
        pub old_area_id: i64,
        pub online: i64,
        pub parent_area_id: i64,
        pub parent_area_name: String,
        pub pendants: String,
        pub pk_id: i64,
        pub pk_status: i64,
        pub room_id: u64,
        pub room_silent_level: i64,
        pub room_silent_second: i64,
        pub room_silent_type: String,
        pub short_id: i64,
        pub studio_info: StudioInfo,
        pub tags: String,
        pub title: String,
        pub uid: i64,
        pub up_session: String,
        pub user_cover: String,
        pub verify: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NewPendants {
        pub badge: Option<Badge>,
        pub frame: Frame,
        pub mobile_badge: Option<Badge>,
        pub mobile_frame: Frame,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Badge {
        pub name: String,
        pub position: i64,
        pub value: String,
        pub desc: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Frame {
        pub area: i64,
        pub area_old: i64,
        pub bg_color: String,
        pub bg_pic: String,
        pub desc: String,
        pub name: String,
        pub position: i64,
        pub use_old_area: bool,
        pub value: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudioInfo {
        pub master_list: Vec<Value>, // TODO: Define a proper struct for master_list
        pub status: i64,
    }
}

pub mod area {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LiveArea {
        pub id: i32,                // 父分区id
        pub name: String,           // 父分区名
        pub list: Vec<SubLiveArea>, // 子分区列表
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubLiveArea {
        pub id: String,          // 子分区id
        pub parent_id: String,   // 父分区id
        pub old_area_id: String, // 旧分区id
        pub name: String,        // 子分区名
        pub act_id: String,      // 0. **作用尚不明确**
        pub pk_status: String,   // ？？？ **作用尚不明确**
        pub hot_status: i32,     // 是否为热门分区。0：否。1：是
        pub lock_status: String, // 0. **作用尚不明确**
        pub pic: String,         // 子分区标志图片url
        pub parent_name: String, // 父分区名
        pub area_type: i32,
    }
}

///
/// **开播返回：**
///
/// 根对象：
///
/// | 字段    | 类型 | 内容     | 备注                                                         |
/// | ------- | ---- | -------- | ------------------------------------------------------------ |
/// | code    | num  | 返回值   | 0：成功<br />65530：token错误（登录错误）<br />1：错误<br />60009：分区不存在<br />60024: 目标分区需要人脸认证<br />60013：非常抱歉，您所在的地区受实名认证限制无法开播<br />**（其他错误码有待补充）** |
/// | msg     | str  | 错误信息 | 默认为空                                                     |
/// | message | str  | 错误信息 | 默认为空                                                     |
/// | data    | obj  | 信息本体 |                                                              |
///
/// `data`对象：
///
/// | 字段      | 类型  | 内容             | 备注                   |
/// | --------- | ----- | ---------------- | ---------------------- |
/// | change    | num   | 是否改变状态     | 0：未改变<br />1：改变 |
/// | status    | str   | 直播间状态       | `LIVE`                 |
/// | room_type | num   | 0                | 作用尚不明确           |
/// | rtmp      | obj   | RTMP推流地址信息 |                        |
/// | protocols | array | ？？？           | 作用尚不明确           |
/// | try_time  | str   | ？？？           | 作用尚不明确           |
/// | live_key  | str   | 标记直播场次的key |                        |
/// | sub_session_key | str   | 信息变动标识 |      |
/// | notice    | obj   | ？？？           | 作用尚不明确           |
/// | qr        | str   | `""`          | 作用尚不明确    |
/// | need_face_auth | bool  | 需要人脸识别? | 作用尚不明确  |
/// | service_source | str   | ？？？    | 作用尚不明确  |
/// | rtmp\_backup | null  | ？？？    | 作用尚不明确  |
/// | up_stream_extra | obj   | 主播推流额外信息? |    |
///
/// `data`中的`rtmp`对象：
///
/// | 字段     | 类型 | 内容                             | 备注         |
/// | -------- | ---- | -------------------------------- | ------------ |
/// | addr     | str  | RTMP推流（发送）地址             | **重要**     |
/// | code     | str  | RTMP推流参数（密钥）             | **重要**     |
/// | new_link | str  | 获取CDN推流ip地址重定向信息的url | 没啥用       |
/// | provider | str  | ？？？                           | 作用尚不明确 |
///
/// `data`中的`protocols`数组：
///
/// | 项   | 类型 | 内容   | 备注         |
/// | ---- | ---- | ------ | ------------ |
/// | 0    | obj  | ？？？ | 作用尚不明确 |
///
/// `data`中的`protocols`数组中的对象：
///
/// | 字段     | 类型 | 内容                             | 备注         |
/// | -------- | ---- | -------------------------------- | ------------ |
/// | protocol | str  | rtmp                             | 作用尚不明确 |
/// | addr     | str  | RTMP推流（发送）地址             |              |
/// | code     | str  | RTMP推流参数（密钥）             |              |
/// | new_link | str  | 获取CDN推流ip地址重定向信息的url |              |
/// | provider | str  | txy                              | 作用尚不明确 |
///
/// `data`中的`notice`对象：
///
/// | 字段        | 类型 | 内容 | 备注         |
/// | ----------- | ---- | ---- | ------------ |
/// | type        | num  | 1    | 作用尚不明确 |
/// | status      | num  | 0    | 作用尚不明确 |
/// | title       | str  | 空   | 作用尚不明确 |
/// | msg         | str  | 空   | 作用尚不明确 |
/// | button_text | str  | 空   | 作用尚不明确 |
/// | button_url  | str  | 空   | 作用尚不明确 |
///
/// `data`中的`up_stream_extra`对象：
///
/// | 字段 | 类型 | 内容 | 备注 |
/// | --- | --- | --- | --- |
/// | isp | str | 主播的互联网服务提供商 |  |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResponse {
    pub change: i64,
    pub live_key: String,
    pub need_face_auth: bool,
    pub notice: Notice,
    pub protocols: Vec<Protocols>,
    pub qr: String,
    pub room_type: i64,
    pub rtmp: Rtmp,
    pub rtmp_backup: Option<Value>,
    pub service_source: String,
    pub status: String,
    pub sub_session_key: String,
    pub try_time: String,
    pub up_stream_extra: UpStreamExtra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notice {
    pub button_text: String,
    pub button_url: String,
    pub msg: String,
    pub status: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub r#type: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocols {
    pub addr: String,
    pub code: String,
    pub new_link: String,
    pub protocol: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rtmp {
    pub addr: String,
    pub code: String,
    pub new_link: String,
    pub provider: String,
    #[serde(rename = "type")]
    pub r#type: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpStreamExtra {
    pub isp: String,
}

/// **关播返回：**
///
/// 根对象：
///
/// | 字段    | 类型 | 内容     | 备注                                                         |
/// | ------- | ---- | -------- | ------------------------------------------------------------ |
/// | code    | num  | 返回值   | 0：成功<br />65530：token错误（登录错误）<br />-400：没有权限<br />**（其他错误码有待补充）** |
/// | msg     | str  | 错误信息 | 默认为空                                                     |
/// | message | str  | 错误信息 | 默认为空                                                     |
/// | data    | obj  | 信息本体 |                                                              |
///
/// `data`对象：
///
/// | 字段   | 类型 | 内容         | 备注                   |
/// | ------ | ---- | ------------ | ---------------------- |
/// | change | num  | 是否改变状态 | 0：未改变<br />1：改变 |
/// | status | str  | 直播间状态   | `PREPARING`、`ROUND` |
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopResponse {
    pub change: i64,
    pub status: String,
}
