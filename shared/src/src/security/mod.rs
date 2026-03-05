use crate::security::middleware::SecurityMiddleware;
use crate::security::audit::{SecurityAuditLogger, SecurityEventMiddleware, InputValidator, CorsMiddleware, SecurityHeadersMiddleware, CorsConfig};
use crate::security::middleware::{JwtConfig, RateLimiter, RateLimitStore, SqlxRateLimitStore};
use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use actix_web::middleware::NormalizePath;
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use log::{info, error, warn, debug};
use crate::utils::{Logger as UtilsLogger, Metrics, ErrorHandler};
use crate::security::{AuthProvider, AuthError, Role, User, Permission, RBAC, RoleDefinition};

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimiter,
    pub audit_db_pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct SecurityModule {
    pub middleware: SecurityMiddleware,
    pub audit_logger: SecurityAuditLogger,
    pub event_middleware: SecurityEventMiddleware,
    pub input_validator: InputValidator,
    pub cors_middleware: CorsMiddleware,
    pub security_headers_middleware: SecurityHeadersMiddleware,
    pub rate_limiter: RateLimiter,
}

impl SecurityModule {
    pub fn new(
        config: SecurityConfig,
        logger: UtilsLogger,
        metrics: Metrics,
        error_handler: ErrorHandler,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> Self {
        let audit_logger = SecurityAuditLogger::new(
            logger.clone(),
            metrics.clone(),
            error_handler.clone(),
            config.audit_db_pool.clone(),
        );

        let event_middleware = SecurityEventMiddleware::new(audit_logger.clone());
        let input_validator = InputValidator::new(audit_logger.clone());
        let cors_middleware = CorsMiddleware::new(config.cors.clone(), audit_logger.clone());
        let security_headers_middleware = SecurityHeadersMiddleware::new(audit_logger.clone());

        let middleware = SecurityMiddleware::new(
            config.jwt.clone(),
            logger,
            metrics,
            error_handler,
            auth_provider,
        );

        Self {
            middleware,
            audit_logger,
            event_middleware,
            input_validator,
            cors_middleware,
            security_headers_middleware,
            rate_limiter: config.rate_limit,
        }
    }

    pub fn init_db(&self) -> Result<(), sqlx::Error> {
        // Create audit log table
        sqlx::query("
            CREATE TABLE IF NOT EXISTS security_audit_log (
                id VARCHAR(36) PRIMARY KEY,
                event_type VARCHAR(50) NOT NULL,
                user_id VARCHAR(36),
                ip_address VARCHAR(45) NOT NULL,
                resource VARCHAR(255) NOT NULL,
                action VARCHAR(50) NOT NULL,
                details TEXT,
                timestamp TIMESTAMP NOT NULL DEFAULT NOW()
            )
        ").execute(&self.audit_logger.db_pool).await?;

        // Create rate limit table
        sqlx::query("
            CREATE TABLE IF NOT EXISTS rate_limits (
                key VARCHAR(255) PRIMARY KEY,
                count INTEGER NOT NULL DEFAULT 1,
                reset_at TIMESTAMP NOT NULL
            )
        ").execute(&self.audit_logger.db_pool).await?;

        Ok(())
    }

    pub fn apply_to_app(&self, app: App) -> App {
        let cors = Cors::new()
            .allowed_origin(self.cors_middleware.config.allowed_origins.clone())
            .allowed_methods(self.cors_middleware.config.allowed_methods.clone())
            .allowed_headers(self.cors_middleware.config.allowed_headers.clone())
            .supports_credentials()
            .max_age(self.cors_middleware.config.max_age as usize);

        app
            .wrap(self.middleware.clone())
            .wrap(self.event_middleware.clone())
            .wrap(self.security_headers_middleware.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(NormalizePath)
    }

    pub async fn authenticate_user(&self, credentials: &crate::security::Credentials) -> Result<User, AuthError> {
        self.middleware.auth_provider.authenticate(credentials).await
    }

    pub async fn create_jwt(&self, user: &User) -> Result<String, AuthError> {
        self.middleware.create_jwt(user).await
    }

    pub async fn validate_jwt(&self, token: &str) -> Result<User, AuthError> {
        self.middleware.validate_jwt(token).await
    }

    pub async fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<(), AuthError> {
        self.middleware.authorize_request(user, resource, action).await
    }

    pub async fn rate_limit_check(&self, key: &str) -> Result<(), AuthError> {
        let current_count = self.rate_limiter.store.increment(key).await?;
        if current_count > self.rate_limiter.max_requests {
            self.audit_logger.log_rate_limit_exceeded("unknown", "rate_limit").await.ok();
            return Err(AuthError::InternalError);
        }
        Ok(())
    }

    pub fn validate_input(&self) -> InputValidator {
        self.input_validator.clone()
    }

    pub async fn log_authentication_success(&self, user_id: &str, ip_address: &str, resource: &str) -> Result<(), sqlx::Error> {
        self.audit_logger.log_authentication_success(user_id, ip_address, resource).await
    }

    pub async fn log_authentication_failure(&self, ip_address: &str, resource: &str, reason: &str) -> Result<(), sqlx::Error> {
        self.audit_logger.log_authentication_failure(ip_address, resource, reason).await
    }

    pub async fn log_authorization_success(&self, user_id: &str, ip_address: &str, resource: &str, action: &str) -> Result<(), sqlx::Error> {
        self.audit_logger.log_authorization_success(user_id, ip_address, resource, action).await
    }

    pub async fn log_authorization_failure(&self, user_id: &str, ip_address: &str, resource: &str, action: &str, reason: &str) -> Result<(), sqlx::Error> {
        self.audit_logger.log_authorization_failure(user_id, ip_address, resource, action, reason).await
    }

    pub async fn log_security_event(&self, event: crate::security::SecurityEvent) -> Result<(), sqlx::Error> {
        self.audit_logger.log_event(event).await
    }
}

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LdapConfig {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_filter: String,
    pub group_filter: String,
}

#[derive(Debug, Clone)]
pub struct AuthProviderConfig {
    pub provider_type: AuthProviderType,
    pub oauth2: Option<OAuth2Config>,
    pub ldap: Option<LdapConfig>,
    pub custom: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AuthProviderType {
    Jwt,
    OAuth2,
    Ldap,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ConfigurableAuthProvider {
    pub config: AuthProviderConfig,
    pub logger: Arc<Logger>,
    pub metrics: Arc<Metrics>,
    pub error_handler: Arc<ErrorHandler>,
}

#[async_trait::async_trait]
impl AuthProvider for ConfigurableAuthProvider {
    async fn authenticate(&self, credentials: &crate::security::Credentials) -> Result<User, AuthError> {
        match &self.config.provider_type {
            AuthProviderType::Jwt => {
                // JWT authentication logic
                unimplemented!()
            }
            AuthProviderType::OAuth2 => {
                // OAuth2 authentication logic
                unimplemented!()
            }
            AuthProviderType::Ldap => {
                // LDAP authentication logic
                unimplemented!()
            }
            AuthProviderType::Custom => {
                // Custom authentication logic
                unimplemented!()
            }
        }
    }

    async fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool, AuthError> {
        // Authorization logic
        unimplemented!()
    }

    async fn validate_token(&self, token: &str) -> Result<User, AuthError> {
        // Token validation logic
        unimplemented!()
    }

    async fn create_token(&self, user: &User, permissions: Vec<crate::security::Permission>) -> Result<String, AuthError> {
        // Token creation logic
        unimplemented!()
    }
}

pub struct SecurityFactory {
    pub config: SecurityConfig,
    pub logger: UtilsLogger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl SecurityFactory {
    pub fn new(config: SecurityConfig, logger: UtilsLogger, metrics: Metrics, error_handler: ErrorHandler) -> Self {
        Self {
            config,
            logger,
            metrics,
            error_handler,
        }
    }

    pub fn create_auth_provider(&self, config: AuthProviderConfig) -> ConfigurableAuthProvider {
        ConfigurableAuthProvider {
            config,
            logger: Arc::new(self.logger.clone()),
            metrics: Arc::new(self.metrics.clone()),
            error_handler: Arc::new(self.error_handler.clone()),
        }
    }

    pub fn create_security_module(&self, auth_provider: Arc<dyn AuthProvider>) -> SecurityModule {
        SecurityModule::new(
            self.config.clone(),
            self.logger.clone(),
            self.metrics.clone(),
            self.error_handler.clone(),
            auth_provider,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SecurityHealthCheck {
    pub audit_logger: Arc<SecurityAuditLogger>,
}

impl SecurityHealthCheck {
    pub fn new(audit_logger: SecurityAuditLogger) -> Self {
        Self {
            audit_logger: Arc::new(audit_logger),
        }
    }

    pub async fn check_database_connection(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.audit_logger.db_pool)
            .await?;
        Ok(())
    }

    pub async fn check_rate_limit_store(&self) -> Result<(), sqlx::Error> {
        let store = SqlxRateLimitStore {
            db_pool: self.audit_logger.db_pool.clone(),
        };
        store.increment("health_check").await?;
        Ok(())
    }

    pub async fn get_security_metrics(&self) -> SecurityMetrics {
        SecurityMetrics {
            total_events: self.audit_logger.metrics.get_counter("security_events_logged").unwrap_or(0),
            rate_limit_hits: self.audit_logger.metrics.get_counter("rate_limit_exceeded").unwrap_or(0),
            authentication_failures: self.audit_logger.metrics.get_counter("auth_failures").unwrap_or(0),
            authorization_failures: self.audit_logger.metrics.get_counter("authz_failures").unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_events: u64,
    pub rate_limit_hits: u64,
    pub authentication_failures: u64,
    pub authorization_failures: u64,
}