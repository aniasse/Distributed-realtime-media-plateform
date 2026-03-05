use actix_web::{dev::ServiceRequest, dev::ServiceResponse, http::header, HttpMessage, HttpResponse, ResponseError};
use actix_web::middleware::MiddlewareFactory;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use log::{info, error, warn, debug};
use crate::utils::{Logger, Metrics, ErrorHandler};
use crate::security::{AuthProvider, AuthError, Role, User, Permission, RBAC, RoleDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub tenant_id: Option<Uuid>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub token_expiration: Duration,
    pub refresh_token_expiration: Duration,
}

#[derive(Debug, Clone)]
pub struct SecurityMiddleware {
    pub jwt_config: Arc<JwtConfig>,
    pub logger: Arc<Logger>,
    pub metrics: Arc<Metrics>,
    pub error_handler: Arc<ErrorHandler>,
    pub auth_provider: Arc<dyn AuthProvider>,
}

impl SecurityMiddleware {
    pub fn new(
        jwt_config: JwtConfig,
        logger: Logger,
        metrics: Metrics,
        error_handler: ErrorHandler,
        auth_provider: Arc<dyn AuthProvider>,
    ) -> Self {
        Self {
            jwt_config: Arc::new(jwt_config),
            logger: Arc::new(logger),
            metrics: Arc::new(metrics),
            error_handler: Arc::new(error_handler),
            auth_provider,
        }
    }

    async fn extract_token(&self, req: &ServiceRequest) -> Option<String> {
        if let Some(auth) = req.headers().get(header::AUTHORIZATION) {
            let auth = auth.to_string();
            if auth.starts_with("Bearer ") {
                return Some(auth["Bearer ".len()..].to_string());
            }
        }
        None
    }

    async fn validate_jwt(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let validation = Validation {
            iss: Some(self.jwt_config.issuer.clone()),
            aud: Some(self.jwt_config.audience.clone()),
            ..Validation::default()
        };

        let token_data = match decode::<JwtClaims>(token, 
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &validation) {
            Ok(data) => data,
            Err(e) => {
                self.logger.warn(&format!("JWT validation failed: {}", e));
                return Err(AuthError::TokenExpired);
            }
        };

        Ok(token_data.claims)
    }

    async fn create_jwt(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.jwt_config.token_expiration).unwrap();
        
        let claims = JwtClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.iter().map(|r| format!("{:?}", r)).collect(),
            tenant_id: user.tenant_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.jwt_config.issuer.clone(),
            aud: self.jwt_config.audience.clone(),
        };

        let token = encode(&Header::default(), &claims, 
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()))
            .map_err(|e| {
                self.logger.error(&format!("JWT creation failed: {}", e));
                AuthError::InternalError
            })?;

        Ok(token)
    }

    async fn authenticate_request(&self, req: &ServiceRequest) -> Result<User, AuthError> {
        if let Some(token) = self.extract_token(req).await {
            let claims = self.validate_jwt(&token).await?;
            
            // Convert claims to User
            let mut roles = Vec::new();
            for role_str in claims.roles {
                match role_str.as_str() {
                    "SuperAdmin" => roles.push(Role::SuperAdmin),
                    "TenantAdmin" => roles.push(Role::TenantAdmin),
                    "Host" => roles.push(Role::Host),
                    "Viewer" => roles.push(Role::Viewer),
                    "Moderator" => roles.push(Role::Moderator),
                    _ => warn!("Unknown role in JWT: {}", role_str),
                }
            }

            Ok(User {
                id: Uuid::parse_str(&claims.sub).unwrap_or_default(),
                username: claims.username,
                email: claims.email,
                roles,
                tenant_id: claims.tenant_id,
                created_at: Utc::now(),
            })
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    async fn authorize_request(&self, user: &User, resource: &str, action: &str) -> Result<(), AuthError> {
        if let Err(e) = self.auth_provider.authorize(user, resource, action).await {
            self.logger.warn(&format!("Authorization failed for {}: {}: {}", user.username, resource, action));
            return Err(e);
        }
        Ok(())
    }

    async fn rate_limit_check(&self, req: &ServiceRequest) -> Result<(), AuthError> {
        // Implement rate limiting logic here
        // For now, just return success
        Ok(())
    }

    async fn security_headers(&self, req: &ServiceRequest, resp: &mut ServiceResponse) {
        resp.headers_mut().insert(header::CONTENT_SECURITY_POLICY, 
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';".parse().unwrap());
        resp.headers_mut().insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
        resp.headers_mut().insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
        resp.headers_mut().insert(header::X_XSS_PROTECTION, "1; mode=block".parse().unwrap());
        resp.headers_mut().insert(header::REFERRER_POLICY, "strict-origin-when-cross-origin".parse().unwrap());
    }
}

impl<S, B> MiddlewareFactory<SecurityMiddleware, S> for SecurityMiddleware
where
    S: actix_web::dev::Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = S::Error;
    type Transform = SecurityMiddlewareService<S>;

    fn create(&self, service: S) -> Self::Transform {
        SecurityMiddlewareService {
            service,
            middleware: self.clone(),
        }
    }
}

pub struct SecurityMiddlewareService<S> {
    service: S,
    middleware: SecurityMiddleware,
}

impl<S, B> actix_web::dev::Service for SecurityMiddlewareService<S>
where
    S: actix_web::dev::Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = actix_web::dev::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: Self::Request) -> Self::Future {
        let middleware = self.middleware.clone();
        let fut = self.service.call(req);
        
        Box::pin(async move {
            // Rate limiting check
            if let Err(e) = middleware.rate_limit_check(&req).await {
                return Ok(HttpResponse::TooManyRequests().finish().into_body());
            }

            // Authenticate request
            let user = match middleware.authenticate_request(&req).await {
                Ok(user) => user,
                Err(_) => {
                    return Ok(HttpResponse::Unauthorized().finish().into_body());
                }
            };

            // Authorize request based on resource and action
            let resource = req.uri().path();
            let action = req.method().as_str();
            
            if let Err(e) = middleware.authorize_request(&user, resource, action).await {
                return Ok(HttpResponse::Forbidden().finish().into_body());
            }

            // Call the actual service
            let mut res = fut.await?;

            // Add security headers
            middleware.security_headers(&req, &mut res);

            Ok(res)
        })
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub max_requests: u32,
    pub window: Duration,
    pub store: Arc<dyn RateLimitStore>,
}

#[async_trait::async_trait]
pub trait RateLimitStore {
    async fn increment(&self, key: &str) -> Result<u32, sqlx::Error>;
    async fn reset(&self, key: &str) -> Result<(), sqlx::Error>;
    async fn get(&self, key: &str) -> Result<u32, sqlx::Error>;
}

#[derive(Debug, Clone)]
pub struct SqlxRateLimitStore {
    pub db_pool: Arc<PgPool>,
}

#[async_trait::async_trait]
impl RateLimitStore for SqlxRateLimitStore {
    async fn increment(&self, key: &str) -> Result<u32, sqlx::Error> {
        let now = Utc::now();
        let result = sqlx::query_as::<_, RateLimitRow>("
            INSERT INTO rate_limits (key, count, reset_at) 
            VALUES ($1, 1, $2)
            ON CONFLICT (key) DO UPDATE 
            SET count = rate_limits.count + 1,
                reset_at = EXCLUDED.reset_at
            RETURNING count
        ")
        .bind(key)
        .bind(now + chrono::Duration::from_std(self.store.window).unwrap())
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.count)
    }

    async fn reset(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM rate_limits WHERE key = $1")
            .bind(key)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<u32, sqlx::Error> {
        let result = sqlx::query_as::<_, RateLimitRow>("
            SELECT count FROM rate_limits WHERE key = $1
        ")
        .bind(key)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.count)
    }
}

#[derive(sqlx::FromRow)]
struct RateLimitRow {
    count: u32,
}