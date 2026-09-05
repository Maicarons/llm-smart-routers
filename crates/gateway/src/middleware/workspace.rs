use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 工作空间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub api_keys: Vec<String>,
    pub provider_ids: Vec<String>,
    pub rate_limit_rpm: u64,
    pub monthly_budget: Option<f64>,
    pub is_active: bool,
}

/// 工作空间管理器 - 与路由引擎隔离
pub struct WorkspaceManager {
    workspaces: dashmap::DashMap<String, Workspace>,
    key_to_workspace: dashmap::DashMap<String, String>,  // api_key → workspace_id
}

impl WorkspaceManager {
    pub fn new() -> Self {
        // 默认工作空间
        let default = Workspace {
            id: "default".to_string(),
            name: "Default".to_string(),
            api_keys: vec!["sk-local-dev".to_string()],
            provider_ids: vec![],
            rate_limit_rpm: 1000,
            monthly_budget: None,
            is_active: true,
        };
        let workspaces = dashmap::DashMap::new();
        let key_to_workspace = dashmap::DashMap::new();
        for key in &default.api_keys {
            key_to_workspace.insert(key.clone(), "default".to_string());
        }
        workspaces.insert("default".to_string(), default);
        Self { workspaces, key_to_workspace }
    }

    /// 根据 API Key 获取工作空间
    pub fn get_workspace_by_key(&self, api_key: &str) -> Option<Workspace> {
        let ws_id = self.key_to_workspace.get(api_key)?;
        self.workspaces.get(ws_id.value()).map(|w| w.value().clone())
    }

    /// 创建新工作空间
    pub fn create_workspace(&self, name: &str, keys: Vec<String>, rpm: u64, budget: Option<f64>) -> String {
        let id = format!("ws_{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        let ws = Workspace {
            id: id.clone(),
            name: name.to_string(),
            api_keys: keys.clone(),
            provider_ids: vec![],
            rate_limit_rpm: rpm,
            monthly_budget: budget,
            is_active: true,
        };
        for key in &keys {
            self.key_to_workspace.insert(key.clone(), id.clone());
        }
        self.workspaces.insert(id.clone(), ws);
        id
    }

    /// 检查速率限制
    pub fn check_rate_limit(&self, _workspace_id: &str) -> bool {
        // 简化实现：实际应使用 Token Bucket 算法
        true
    }

    /// 获取工作空间
    pub fn get_workspace(&self, id: &str) -> Option<Workspace> {
        self.workspaces.get(id).map(|w| w.value().clone())
    }

    /// 列出所有工作空间
    pub fn list_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.iter().map(|w| w.value().clone()).collect()
    }

    /// 校验 API Key 是否有效
    pub fn validate_key(&self, key: &str) -> bool {
        self.key_to_workspace.contains_key(key)
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_workspace() {
        let mgr = WorkspaceManager::new();
        assert!(mgr.validate_key("sk-local-dev"));
        let ws = mgr.get_workspace_by_key("sk-local-dev");
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "Default");
    }

    #[test]
    fn test_create_workspace() {
        let mgr = WorkspaceManager::new();
        let id = mgr.create_workspace("Test", vec!["sk-test-1".to_string()], 100, Some(10.0));
        assert!(mgr.validate_key("sk-test-1"));
        let ws = mgr.get_workspace_by_key("sk-test-1").unwrap();
        assert_eq!(ws.name, "Test");
        assert_eq!(ws.monthly_budget, Some(10.0));
    }
}