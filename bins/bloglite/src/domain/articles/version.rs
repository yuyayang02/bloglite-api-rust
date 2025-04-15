use std::{collections::HashMap, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("哈希值不能为空")]
    EmptyHashValue, // 输入格式无效

    #[error("版本 '{0}' 已存在")]
    DuplicateVersion(String), // 版本哈希值重复

    #[error("版本 '{0}' 不存在")]
    VersionNotFound(String), // 版本哈希值不存在
}

pub struct Version {
    pub hash: Arc<str>,
    pub parent: Option<Arc<str>>,
}

impl Version {
    pub fn new<T: AsRef<str>>(hash: T) -> Result<Self, Error> {
        let s = hash.as_ref();
        if s.is_empty() {
            if s.is_empty() {
                return Err(Error::EmptyHashValue);
            }
        }

        Ok(Self {
            hash: Arc::from(s), // 在内部创建 Arc
            parent: None,
        })
    }

    pub fn set_parent(&mut self, parent: Arc<str>) {
        self.parent = Some(parent);
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.hash
    }
}

impl From<Version> for Arc<str> {
    fn from(value: Version) -> Self {
        value.hash
    }
}

pub struct VersionHistory {
    pub current_version_hash: Arc<str>,
    pub version_history: HashMap<Arc<str>, Version>,
}

impl VersionHistory {
    /// 从根hash创建一个version history
    pub fn new<T: AsRef<str>>(version_hash: T) -> Result<Self, Error> {
        let version_hash = version_hash.as_ref();

        let version = Version::new(version_hash)?;

        Ok(Self {
            current_version_hash: version.hash.clone(),
            version_history: HashMap::from([(version.hash.clone(), version)]),
        })
    }

    /// 添加新版本
    pub fn add_version<T: AsRef<str>>(&mut self, version_hash: T) -> Result<(), Error> {
        let version_hash = version_hash.as_ref();

        if self.is_exist(version_hash) {
            return Err(Error::DuplicateVersion(version_hash.to_string()));
        }

        let mut version = Version::new(&version_hash)?;

        // 设置版本父级
        version.set_parent(self.current_version_hash.clone());

        // 更新当前版本并插入新版本
        self.current_version_hash = version.hash.clone();
        self.version_history.insert(version.hash.clone(), version);

        Ok(())
    }

    /// 回退到某版本
    pub fn rollback_to_version<T: AsRef<str>>(&mut self, version_hash: T) -> Result<(), Error> {
        let version_hash = version_hash.as_ref();

        if self.version_history.contains_key(version_hash) {
            self.current_version_hash = Arc::from(version_hash);
            Ok(())
        } else {
            Err(Error::VersionNotFound(version_hash.to_string()))
        }
    }

    /// 检查历史版本是否存在某版本
    pub fn is_exist<T: AsRef<str>>(&self, hash: T) -> bool {
        self.version_history.contains_key(hash.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_version() {
        let version = Version::new("version").unwrap();
        assert_eq!(version.as_ref(), "version");
        assert!(version.is_root());
        assert!(Version::new("").is_err())
    }

    #[test]
    fn test_new_version_history() {
        let version = Version::new("version").unwrap();
        let _ = VersionHistory::new(version).unwrap();
    }

    #[test]
    fn test_version_history_add_version() {
        let mut history = VersionHistory::new(Version::new("version").unwrap()).unwrap();

        assert!(history.add_version("version").is_err());

        assert!(history.add_version("new_version").is_ok());

        // assert_eq!(history.current_version().unwrap().as_ref(), "new_version")
    }
}
