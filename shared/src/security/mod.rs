//! Security utilities and types for the media platform.

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<Role>,
    pub tenant_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Role {
    SuperAdmin,
    TenantAdmin,
    Host,
    Viewer,
    Moderator,
    Admin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RBAC {
    pub roles: Vec<RoleDefinition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoleDefinition {
    pub name: String,
    pub permissions: Vec<Permission>,
    pub inherits: Vec<Uuid>,
}

impl RBAC {
    pub fn new() -> Self {
        Self { roles: Vec::new() }
    }

    pub fn add_role(&mut self, _id: Uuid, role: RoleDefinition) {
        self.roles.push(role);
    }

    pub fn get_permissions(&self, role: &Role) -> Vec<Permission> {
        self.roles
            .iter()
            .find(|r| r.name == format!("{:?}", role))
            .map(|r| r.permissions.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    PermissionDenied,
    InternalError,
    RoleNotFound,
}

pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> Result<User, AuthError>;
    fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool, AuthError>;
    fn create_token(&self, user: &User) -> Result<String, AuthError>;
    fn validate_token(&self, token: &str) -> Result<User, AuthError>;
    fn validate_stream_key(&self, stream_key: &str) -> bool;
}
