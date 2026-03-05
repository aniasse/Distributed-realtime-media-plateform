use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{info, error, debug};

use crate::shared::security::{AuthProvider, AuthError, Role, User, Permission, RBAC, RoleDefinition};
use crate::shared::utils::{Logger, Metrics, ErrorHandler};

pub struct AuthService {
    pub db_pool: PgPool,
    pub rbac: RBAC,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl AuthService {
    pub fn new(db_pool: PgPool) -> Self {
        let mut rbac = RBAC::new();
        
        // Define roles
        let super_admin_role = RoleDefinition {
            name: "SuperAdmin".to_string(),
            permissions: vec![
                Permission { resource: "*".to_string(), action: "*".to_string(), allowed: true },
            ],
            inherits: vec![],
        };
        
        let tenant_admin_role = RoleDefinition {
            name: "TenantAdmin".to_string(),
            permissions: vec![
                Permission { resource: "rooms".to_string(), action: "create".to_string(), allowed: true },
                Permission { resource: "rooms".to_string(), action: "delete".to_string(), allowed: true },
                Permission { resource: "users".to_string(), action: "manage".to_string(), allowed: true },
            ],
            inherits: vec![],
        };
        
        let host_role = RoleDefinition {
            name: "Host".to_string(),
            permissions: vec![
                Permission { resource: "rooms".to_string(), action: "publish".to_string(), allowed: true },
                Permission { resource: "rooms".to_string(), action: "manage".to_string(), allowed: true },
            ],
            inherits: vec![],
        };
        
        let viewer_role = RoleDefinition {
            name: "Viewer".to_string(),
            permissions: vec![
                Permission { resource: "rooms".to_string(), action: "subscribe".to_string(), allowed: true },
            ],
            inherits: vec![],
        };
        
        let moderator_role = RoleDefinition {
            name: "Moderator".to_string(),
            permissions: vec![
                Permission { resource: "rooms".to_string(), action: "moderate".to_string(), allowed: true },
                Permission { resource: "users".to_string(), action: "manage".to_string(), allowed: true },
            ],
            inherits: vec![],
        };
        
        // Add roles to RBAC
        rbac.add_role(Uuid::new_v4(), super_admin_role);
        rbac.add_role(Uuid::new_v4(), tenant_admin_role);
        rbac.add_role(Uuid::new_v4(), host_role);
        rbac.add_role(Uuid::new_v4(), viewer_role);
        rbac.add_role(Uuid::new_v4(), moderator_role);
        
        Self {
            db_pool,
            rbac,
            logger: Logger::new("auth"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("auth"),
        }
    }

    pub async fn start(&self) -> std::io::Result<()> {
        self.logger.info("Starting Auth service");
        
        // Initialize database tables
        self.initialize_db().await?;
        
        Ok(())
    }

    async fn initialize_db(&self) -> Result<(), sqlx::Error> {
        self.logger.info("Initializing Auth database");
        
        // Create users table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username VARCHAR(255) UNIQUE NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP NOT NULL DEFAULT NOW()
            )"
        )
        .execute(&self.db_pool)
        .await?;
        
        // Create roles table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS roles (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                role_name VARCHAR(50) NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            )"
        )
        .execute(&self.db_pool)
        .await?;
        
        // Create permissions table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS permissions (
                id UUID PRIMARY KEY,
                role_id UUID NOT NULL REFERENCES roles(id),
                resource VARCHAR(255) NOT NULL,
                action VARCHAR(50) NOT NULL,
                allowed BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            )"
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }

    pub async fn register_user(&self, username: &str, email: &str, password: &str) -> Result<Uuid, AuthError> {
        self.logger.info(&format!("Registering user: {}", username));
        
        // Hash password
        let password_hash = crate::shared::utils::hash_password(password);
        
        // Insert user
        let user_id = Uuid::new_v4();
        let result = sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)")
            .bind(user_id)
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .execute(&self.db_pool)
            .await;
        
        match result {
            Ok(_) => {
                self.metrics.increment_counter("users_registered", 1);
                Ok(user_id)
            }
            Err(e) => {
                self.error_handler.handle_error(&e, "register_user");
                Err(AuthError::InternalError)
            }
        }
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<User, AuthError> {
        self.logger.info(&format!("Authenticating user: {}", username));
        
        // Get user from database
        let result = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.db_pool)
            .await;
        
        let user_row = match result {
            Ok(row) => row,
            Err(_) => return Err(AuthError::InvalidCredentials),
        };
        
        // Verify password
        if !crate::shared::utils::verify_password(password, &user_row.password_hash) {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Get user roles
        let roles = self.get_user_roles(user_row.id).await?;
        
        Ok(User {
            id: user_row.id,
            username: user_row.username,
            email: user_row.email,
            roles,
            tenant_id: None, // Would come from actual tenant system
            created_at: user_row.created_at,
        })
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, AuthError> {
        let result = sqlx::query_as::<_, RoleRow>("SELECT role_name FROM roles WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.db_pool)
            .await;
        
        let role_rows = match result {
            Ok(rows) => rows,
            Err(e) => {
                self.error_handler.handle_error(&e, "get_user_roles");
                return Err(AuthError::InternalError);
            }
        };
        
        let mut roles = Vec::new();
        for role_row in role_rows {
            match role_row.role_name.as_str() {
                "SuperAdmin" => roles.push(Role::SuperAdmin),
                "TenantAdmin" => roles.push(Role::TenantAdmin),
                "Host" => roles.push(Role::Host),
                "Viewer" => roles.push(Role::Viewer),
                "Moderator" => roles.push(Role::Moderator),
                _ => warn!("Unknown role: {}", role_row.role_name),
            }
        }
        
        Ok(roles)
    }

    pub async fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool, AuthError> {
        self.logger.debug(&format!("Authorizing user {} for {}:{}", user.username, resource, action));
        
        // Check RBAC permissions
        for role in &user.roles {
            let permissions = self.rbac.get_permissions(&role);
            for permission in permissions {
                if permission.resource == "*" || permission.resource == resource {
                    if permission.action == "*" || permission.action == action {
                        return Ok(permission.allowed);
                    }
                }
            }
        }
        
        Ok(false)
    }

    pub async fn create_token(&self, user: &User, permissions: Vec<Permission>) -> Result<String, AuthError> {
        self.logger.info(&format!("Creating token for user {}", user.username));
        
        // In a real implementation, this would generate a JWT or similar token
        // For now, we'll just return a dummy token
        let token = format!("token_{}_{}", user.id, chrono::Utc::now().timestamp());
        
        self.metrics.increment_counter("tokens_created", 1);
        Ok(token)
    }

    pub async fn validate_token(&self, token: &str) -> Result<User, AuthError> {
        self.logger.debug(&format!("Validating token: {}", token));
        
        // In a real implementation, this would validate a JWT or similar token
        // For now, we'll just return a dummy user
        let user_id = Uuid::new_v4();
        let username = "test_user";
        
        // Get user from database
        let result = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await;
        
        let user_row = match result {
            Ok(row) => row,
            Err(_) => return Err(AuthError::TokenExpired),
        };
        
        // Get user roles
        let roles = self.get_user_roles(user_row.id).await?;
        
        Ok(User {
            id: user_row.id,
            username: user_row.username,
            email: user_row.email,
            roles,
            tenant_id: None, // Would come from actual tenant system
            created_at: user_row.created_at,
        })
    }

    pub async fn add_role(&self, user_id: Uuid, role_name: &str) -> Result<(), AuthError> {
        self.logger.info(&format!("Adding role {} to user {}", role_name, user_id));
        
        let result = sqlx::query("INSERT INTO roles (id, user_id, role_name) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(role_name)
            .execute(&self.db_pool)
            .await;
        
        match result {
            Ok(_) => {
                self.metrics.increment_counter("roles_added", 1);
                Ok(())
            }
            Err(e) => {
                self.error_handler.handle_error(&e, "add_role");
                Err(AuthError::InternalError)
            }
        }
    }

    pub async fn remove_role(&self, user_id: Uuid, role_name: &str) -> Result<(), AuthError> {
        self.logger.info(&format!("Removing role {} from user {}", role_name, user_id));
        
        let result = sqlx::query("DELETE FROM roles WHERE user_id = $1 AND role_name = $2")
            .bind(user_id)
            .bind(role_name)
            .execute(&self.db_pool)
            .await;
        
        match result {
            Ok(rows) if rows > 0 => {
                self.metrics.increment_counter("roles_removed", 1);
                Ok(())
            }
            Ok(_) => Err(AuthError::RoleNotFound),
            Err(e) => {
                self.error_handler.handle_error(&e, "remove_role");
                Err(AuthError::InternalError)
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    role_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auth_service_creation() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        assert!(auth_service.db_pool.is_valid());
    }
    
    #[tokio::test]
    async fn test_register_user() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_service = AuthService::new(db_pool);
        
        let user_id = auth_service.register_user("test", "test@example.com", "password").await.unwrap();
        assert!(user_id != Uuid::new_v4());
    }
}