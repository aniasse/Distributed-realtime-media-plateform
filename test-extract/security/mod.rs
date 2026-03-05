use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    SuperAdmin,
    TenantAdmin,
    Host,
    Viewer,
    Moderator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<Role>,
    pub tenant_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    pub tenant_id: Uuid,
    pub permissions: HashMap<String, Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub tenant_id: Uuid,
    pub max_rooms: u32,
    pub max_peers_per_room: u32,
    pub max_bandwidth_mbps: u32,
    pub max_storage_gb: u32,
}

pub trait AuthProvider {
    fn authenticate(&self, credentials: &Credentials) -> Result<User, AuthError>;
    fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool, AuthError>;
    fn validate_token(&self, token: &str) -> Result<User, AuthError>;
    fn create_token(&self, user: &User, permissions: Vec<Permission>) -> Result<String, AuthError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
    UserNotFound,
    InternalError,
}

pub trait AccessControl {
    fn check_permission(
        &self,
        user: &User,
        resource: &str,
        action: &str,
    ) -> Result<bool, AccessControlError>;
    fn enforce_permission(
        &self,
        user: &User,
        resource: &str,
        action: &str,
    ) -> Result<(), AccessControlError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessControlError {
    PermissionDenied,
    ResourceNotFound,
    InternalError,
}

pub struct RBAC {
    pub roles: HashMap<Uuid, RoleDefinition>,
    pub permissions: HashMap<String, Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub name: String,
    pub permissions: Vec<Permission>,
    pub inherits: Vec<Uuid>,
}

impl RBAC {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            permissions: HashMap::new(),
        }
    }

    pub fn add_role(&mut self, role_id: Uuid, role: RoleDefinition) {
        self.roles.insert(role_id, role);
    }

    pub fn get_permissions(&self, role_id: &Uuid) -> Vec<Permission> {
        let mut permissions = Vec::new();
        if let Some(role) = self.roles.get(role_id) {
            permissions.extend(role.permissions.clone());
            for parent_id in &role.inherits {
                if let Some(parent) = self.roles.get(parent_id) {
                    permissions.extend(parent.permissions.clone());
                }
            }
        }
        permissions
    }
}
