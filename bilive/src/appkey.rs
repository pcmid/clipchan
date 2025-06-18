use md5::{Digest, Md5};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum AppKeyStore {
    BiliTV,
    Android,
}

#[allow(dead_code)]
impl AppKeyStore {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "BiliTV" => Some(Self::BiliTV),
            "Android" => Some(Self::Android),
            _ => None,
        }
    }

    pub(crate) fn app_key(&self) -> &'static str {
        match self {
            Self::BiliTV => "4409e2ce8ffd12b8",
            Self::Android => "1d8b6e7d45233436",
        }
    }

    pub(crate) fn app_sec(&self) -> &'static str {
        match self {
            Self::BiliTV => "59b43e04ad6965f34319062b478f83dd",
            Self::Android => "560c52ccd288fed045859ed18bffd973",
        }
    }
    pub(crate) fn sign(&self, param: &str) -> String {
        sign(param, self.app_sec())
    }
}

impl Default for AppKeyStore {
    fn default() -> Self {
        Self::BiliTV
    }
}

pub(crate) fn sign(param: &str, app_sec: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(format!("{}{}", param, app_sec));
    format!("{:x}", hasher.finalize())
}
