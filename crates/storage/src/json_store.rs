use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub models: Vec<ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub capabilities: Vec<String>,
}

/// 策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub strategy_type: String,
    pub config: Option<serde_json::Value>,
}

/// 配置变更事件
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    ProvidersChanged,
    StrategiesChanged,
    ApiKeysChanged,
}

/// JSON 配置管理
pub struct JsonStore {
    config_path: PathBuf,
    last_modified: std::time::SystemTime,
}

impl JsonStore {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let config_path = PathBuf::from(path);
        std::fs::create_dir_all(&config_path)?;

        let last_modified = std::time::UNIX_EPOCH;
        let mut store = Self {
            config_path,
            last_modified,
        };
        store.update_last_modified();
        Ok(store)
    }

    fn update_last_modified(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.config_path) {
            if let Ok(modified) = meta.modified() {
                self.last_modified = modified;
            }
        }
    }

    /// 检查配置是否已变更
    pub fn check_changed(&mut self) -> bool {
        if let Ok(meta) = std::fs::metadata(&self.config_path) {
            if let Ok(modified) = meta.modified() {
                if modified > self.last_modified {
                    self.update_last_modified();
                    return true;
                }
            }
        }
        false
    }

    /// 加载提供商配置
    pub fn load_providers(&self) -> anyhow::Result<Vec<ProviderConfig>> {
        let path = self.config_path.join("providers.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        #[derive(Deserialize)]
        struct Wrapper {
            providers: Vec<ProviderConfig>,
        }
        let wrapper: Wrapper = serde_json::from_str(&content)?;
        Ok(wrapper.providers)
    }

    /// 保存提供商配置
    pub fn save_providers(&self, providers: &[ProviderConfig]) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            providers: &'a [ProviderConfig],
        }
        let content = serde_json::to_string_pretty(&Wrapper { providers })?;
        std::fs::write(self.config_path.join("providers.json"), content)?;
        Ok(())
    }

    /// 加载策略配置
    pub fn load_strategies(&self) -> anyhow::Result<Vec<StrategyConfig>> {
        let path = self.config_path.join("strategies.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        #[derive(Deserialize)]
        struct Wrapper {
            strategies: Vec<StrategyConfig>,
        }
        let wrapper: Wrapper = serde_json::from_str(&content)?;
        Ok(wrapper.strategies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_json_store() {
        let dir = std::env::temp_dir().join("test_json_store");
        let _ = std::fs::create_dir_all(&dir);
        let store = JsonStore::new(dir.to_str().unwrap()).unwrap();

        let providers = vec![ProviderConfig {
            name: "test".to_string(),
            api_base_url: "https://test.com".to_string(),
            api_key: "sk-test".to_string(),
            models: vec![ModelConfig {
                id: "test-model".to_string(),
                capabilities: vec!["chat".to_string()],
            }],
        }];
        store.save_providers(&providers).unwrap();

        let loaded = store.load_providers().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "test");

        let _ = std::fs::remove_dir_all(&dir);
    }
}