use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tokio::sync::Mutex;

use crate::bapi::area::LiveArea;
use crate::bapi::master::MasterInfo;
use crate::bapi::room::RoomInfo;
use crate::bapi::*;
use crate::session::Session;
use crate::wbi::WBI;

pub struct Live {
    pub session: Session,
    pub wbi: Arc<Mutex<WBI>>,
}

impl Live {
    pub fn new(session: Session, wbi: Arc<Mutex<WBI>>) -> Self {
        Self { session, wbi }
    }
}

impl Live {
    pub async fn master_info(&self, mid: u64) -> Result<MasterInfo, Box<dyn Error + Sync + Send>> {
        let mut params = BTreeMap::new();
        params.insert("uid", mid.to_string());
        let res: ResponseData<MasterInfo> = self
            .session
            .client
            .get("https://api.live.bilibili.com/live_user/v1/Master/info")
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Response from master_info: {:?}", res);
        if res.code != 0 {
            return Err(format!("Error fetching master info: {}", res.message).into());
        }
        let data = res.data.ok_or("No master info available")?;
        Ok(data)
    }

    pub async fn room_info_by_mid(
        &self,
        mid: u64,
    ) -> Result<RoomInfo, Box<dyn Error + Sync + Send>> {
        let master_info = self.master_info(mid).await?;
        if master_info.room_id == 0 {
            return Err("No room found for this user".into());
        }
        self.room_info(master_info.room_id).await
    }

    pub async fn room_info(&self, room: u64) -> Result<RoomInfo, Box<dyn Error + Sync + Send>> {
        let wts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let mut params = BTreeMap::new();
        params.insert("room_id", room.to_string());
        params.insert("wts", wts.clone());

        let w_rid = self.wbi.lock().await.sign_with_latest(&params).await?;
        let query_string = format!("room_id={}&wts={}&w_rid={}", room, wts, w_rid);

        let res: ResponseData<Value> = self
            .session
            .client
            .get(&format!(
                "https://api.live.bilibili.com/room/v1/Room/get_info?{}",
                query_string
            ))
            .send()
            .await?
            .json()
            .await?;

        if res.code != 0 {
            return Err(format!(
                "Error fetching room info: Code: {}, {}",
                res.code, res.message
            )
            .into());
        }
        let data = res.data.ok_or("No room info available")?;
        tracing::trace!("Response from room_info: {:?}", data);
        let data: RoomInfo = serde_json::from_value(data)
            .map_err(|e| format!("Failed to parse room info: {}", e))?;
        Ok(data)
    }

    pub async fn get_all_areas(&self) -> Result<Vec<LiveArea>, Box<dyn Error + Sync + Send>> {
        let res: ResponseData<Vec<LiveArea>> = self
            .session
            .client
            .get("https://api.live.bilibili.com/room/v1/Area/getList")
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Response from get_all_area: {:?}", res);
        if res.code != 0 {
            return Err(format!("Error fetching all areas: {}", res.message).into());
        }
        Ok(res.data.ok_or("No area data available")?)
    }

    /// ## 开始直播
    ///
    /// > https://api.live.bilibili.com/room/v1/Room/startLive
    ///
    /// *请求方式：POST*
    ///
    /// 认证方式：Cookie（SESSDATA）
    ///
    /// 鉴权方式：Cookie中`bili_jct`的值正确并与`csrf`相同
    ///
    /// 开播时必须有分区选择，开播后返回推流地址
    ///
    /// **正文参数（ application/x-www-form-urlencoded ）：**
    ///
    /// | 参数名   | 类型 | 内容                     | 必要性 | 备注                                |
    /// | -------- | ---- | ------------------------ | ------ | ----------------------------------- |
    /// | room_id  | num  | 直播间id                 | 必要   | 必须为自己的直播间id                |
    /// | area_v2  | num  | 直播分区id（子分区id）   | 必要   | 详见[直播分区](live_area.md)        |
    /// | platform | str  | 直播平台                 | 必要   | 直播姬（pc）：pc_link<br />web在线直播：web_link<br />bililink：android_link |
    /// | csrf     | str  | CSRF Token（位于cookie） | 必要   |                                     |
    ///
    /// **示例：**
    ///
    /// 以`27`作为分区id开播直播间`10352053`
    ///
    /// 其中`"data"."rtmp"."addr"`为推流地址
    ///
    /// `"data"."rtmp"."code"`为推流参数
    ///
    /// ```shell
    /// curl 'https://api.live.bilibili.com/room/v1/Room/startLive' \
    /// --data-urlencode 'room_id=10352053' \
    /// --data-urlencode 'area_v2=27' \
    /// --data-urlencode 'platform=pc' \
    /// --data-urlencode 'csrf=xxx' \
    /// -b 'SESSDATA=xxx;bili_jct=xx'
    /// ```
    pub async fn start_live(
        &self,
        room_id: u64,
        area_id: i32,
    ) -> Result<StartResponse, Box<dyn Error + Sync + Send>> {
        let csrf = self.session.credentials()?.bili_jct;
        let mut params = BTreeMap::new();
        params.insert("room_id", room_id.to_string());
        params.insert("area_v2", area_id.to_string()); // 示例分区ID
        params.insert("platform", "web".to_string()); // 这里要用web
        params.insert("csrf", csrf);

        let res: ResponseData<StartResponse> = self
            .session
            .client
            .post("https://api.live.bilibili.com/room/v1/Room/startLive")
            .form(&params)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Response from start_live: {:?}", res);
        if res.code != 0 {
            return Err(format!(
                "Error starting live: Code: {}, Message: {}",
                res.code, res.message
            )
            .into());
        }
        let data = res.data.ok_or("No data available in response")?;
        tracing::debug!("Live started successfully: {}", data.change);
        Ok(data)
    }

    /// ## 关闭直播
    ///
    /// > https://api.live.bilibili.com/room/v1/Room/stopLive
    ///
    /// *请求方式：POST*
    ///
    /// 认证方式：Cookie（SESSDATA）
    ///
    /// 鉴权方式：Cookie中`bili_jct`的值正确并与`csrf`相同
    ///
    /// **正文参数（ application/x-www-form-urlencoded ）：**
    ///
    /// | 参数名  | 类型 | 内容                     | 必要性 | 备注                 |
    /// | ------- | ---- | ------------------------ | ------ | -------------------- |
    /// | room_id | num  | 直播间id                 | 必要   | 必须为自己的直播间id |
    /// | csrf    | str  | CSRF Token（位于cookie） | 必要   |                      |
    ///
    /// **示例：**
    ///
    /// 关闭直播间`10352053`的直播
    ///
    /// ```shell
    /// curl 'https://api.live.bilibili.com/room/v1/Room/stopLive' \
    ///   --data-urlencode 'room_id=10352053' \
    ///   --data-urlencode 'csrf=xxx' \
    ///   -b 'SESSDATA=xxx;bili_jct=xxx'
    /// ```
    pub async fn stop_live(&self, room_id: u64) -> Result<(), Box<dyn Error + Sync + Send>> {
        let csrf = self.session.credentials()?.bili_jct;
        let mut params = BTreeMap::new();
        params.insert("room_id", room_id.to_string());
        params.insert("csrf", csrf);

        let res: ResponseData<Value> = self
            .session
            .client
            .post("https://api.live.bilibili.com/room/v1/Room/stopLive")
            .form(&params)
            .send()
            .await?
            .json()
            .await?;
        tracing::trace!("Response from stop_live: {:?}", res);

        if res.code != 0 {
            return Err(format!(
                "Error stopping live: Code: {}, Message: {}",
                res.code, res.message
            )
            .into());
        }

        let data = res.data.ok_or("No data available in response")?;
        serde_json::to_writer_pretty(&mut std::io::stdout(), &data)?;
        tracing::info!("Live stopped successfully: {:?}", data);
        Ok(())
    }

    pub async fn get_face_auth_qrcode(
        &self,
        mid: u64,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        // https://www.bilibili.com/blackboard/live/face-auth-middle.html?source_event=400&mid=$mid
        let _params = BTreeMap::from([
            ("source_event", "400".to_string()),
            ("mid", mid.to_string()),
        ]);
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;
    use std::time::Duration;

    use qrcode::QrCode;
    use qrcode::render::svg;
    use tokio::sync::Mutex;

    use super::*;
    use crate::session::Session;
    use crate::wbi::WBI;

    #[tokio::test]
    async fn test_master_info() {
        let wbi = Arc::new(Mutex::new(WBI::new().await.unwrap()));
        let session = Session::new().await.unwrap();
        let live = Live::new(session, wbi);
        let result = live.master_info(3493138238802376).await;
        assert!(
            result.is_ok(),
            "Failed to fetch master info: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_room_info() {
        let wbi = Arc::new(Mutex::new(WBI::new().await.unwrap()));
        let session = Session::new().await.unwrap();
        let live = Live::new(session, wbi);
        let result = live.room_info(1816494416).await;
        assert!(
            result.is_ok(),
            "Failed to fetch room info: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_start_live() {
        let wbi = Arc::new(Mutex::new(WBI::new().await.unwrap()));
        let session = Session::new().await.unwrap();
        let qrcode_info = session
            .get_qrcode()
            .await
            .map_err(|e| format!("Failed to get QR code: {}", e))
            .unwrap();
        assert!(
            !qrcode_info.url.is_empty(),
            "QR code URL should not be empty"
        );
        tracing::debug!("QR Code: {:?}", qrcode_info);

        let qr = QrCode::new(qrcode_info.url).unwrap();
        let qr_string = qr
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        tracing::debug!("QR code:\n{}", qr_string);
        let svg = qr
            .render()
            .min_dimensions(200, 200)
            .quiet_zone(true)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();

        let mut file = std::fs::File::create("test.qr.svg").unwrap();
        file.write_all(svg.as_bytes()).unwrap();
        let info = session
            .wait_for_login(&qrcode_info.qrcode_key, Duration::from_secs(60))
            .await
            .map_err(|e| format!("Failed to wait for login: {}", e))
            .unwrap();

        let mid = session
            .get_account()
            .await
            .map_err(|e| format!("Failed to get account info: {}", e))
            .unwrap()
            .mid;
        let wbi = Arc::new(Mutex::new(WBI::new().await.unwrap()));
        let live = Live::new(session, wbi);
        let result = live
            .master_info(mid)
            .await
            .map_err(|e| format!("Failed to fetch master info: {}", e))
            .unwrap();
        let room_id = result.room_id;
        if room_id == 0 {
            tracing::warn!("No room ID found for user: {}", result.info.uname);
            return;
        }

        let result = live.start_live(room_id, 744).await;
        assert!(result.is_ok(), "Failed to start live: {:?}", result.err());

        tracing::info!("Live started successfully for room ID: {}", room_id);
        // let stop_result = live
        //     .stop_live(room_id)
        //     .await
        //     .map_err(|e| format!("Failed to stop live: {:?}", e))
        //     .unwrap();
    }

    #[tokio::test]
    async fn test_get_all_areas() {
        let wbi = Arc::new(Mutex::new(WBI::new().await.unwrap()));
        let session = Session::new().await.unwrap();
        let live = Live::new(session, wbi);
        let result = live
            .get_all_areas()
            .await
            .map_err(|e| format!("Failed to get all areas: {}", e))
            .unwrap();
        tracing::info!("Fetched all areas successfully: {:?}", result);
    }
}
