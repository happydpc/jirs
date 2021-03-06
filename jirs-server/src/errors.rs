use actix_web::HttpResponse;

use jirs_data::ErrorResponse;

const TOKEN_NOT_FOUND: &str = "Token not found";
const DATABASE_CONNECTION_FAILED: &str = "Database connection failed";

#[derive(Debug)]
pub enum ServiceErrors {
    Unauthorized,
    DatabaseConnectionLost,
    DatabaseQueryFailed(String),
    RecordNotFound(String),
    RegisterCollision,
}

impl ServiceErrors {
    pub fn into_http_response(self) -> HttpResponse {
        self.into()
    }
}

impl Into<HttpResponse> for ServiceErrors {
    fn into(self) -> HttpResponse {
        match self {
            ServiceErrors::Unauthorized => HttpResponse::Unauthorized().json(ErrorResponse {
                errors: vec![TOKEN_NOT_FOUND.to_owned()],
            }),
            ServiceErrors::DatabaseConnectionLost => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    errors: vec![DATABASE_CONNECTION_FAILED.to_owned()],
                })
            }
            ServiceErrors::DatabaseQueryFailed(error) => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    errors: vec![error],
                })
            }
            ServiceErrors::RecordNotFound(resource_name) => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    errors: vec![format!("Resource not found {}", resource_name)],
                })
            }
            ServiceErrors::RegisterCollision => HttpResponse::Unauthorized().json(ErrorResponse {
                errors: vec!["Register collision".to_string()],
            }),
        }
    }
}
