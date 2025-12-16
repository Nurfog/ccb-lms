use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ccb_common::{AuthenticatedUser, Claims, UserRole};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::env;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration, DateTime};
use uuid::Uuid;

// --- Modelos de Datos ---

/// Estructura para recibir los datos de registro.
#[derive(Deserialize)]
struct RegisterUser {
    username: String,
    password: String,
    email: String,
    first_name: String,
    last_name: String,
}

/// Estructura para recibir los datos de login.
#[derive(Deserialize)]
struct LoginUser {
    username: String,
    password: String,
}

/// Estructura para representar un usuario en la base de datos y en las respuestas API.
#[derive(Serialize, FromRow)]
struct User {
    id: Uuid,
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    #[serde(skip_serializing)] // Nunca enviar el hash de la contraseña al cliente
    password_hash: String,
    role: UserRole,
    created_at: DateTime<Utc>,
}

/// Estructura para la respuesta del login, que contiene el token.
#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

// --- Estado de la Aplicación ---

/// Contiene los datos compartidos entre los hilos del servidor, como el pool de conexiones a la BD.
struct AppState {
    db_pool: PgPool,
}

// --- Manejadores de Endpoints (Handlers) ---

/// Maneja las peticiones POST a /register
async fn register(
    state: web::Data<AppState>,
    user_data: web::Json<RegisterUser>,
) -> impl Responder {
    // Extraemos los datos antes de mover la contraseña a un hilo bloqueante.
    let username = user_data.username.clone();
    let password = user_data.password.clone();
    let email = user_data.email.clone();
    let first_name = user_data.first_name.clone();
    let last_name = user_data.last_name.clone();

    // Hashear la contraseña del usuario. Es un proceso que consume CPU,
    // por lo que lo ejecutamos en un hilo bloqueante para no detener el event loop.
    let password_hash = match web::block(move || hash(&password, DEFAULT_COST)).await {
        Ok(Ok(hash)) => hash,
        _ => return HttpResponse::InternalServerError().body("Error hashing password"),
    };

    // Insertar el nuevo usuario en la base de datos.
    // Usamos `query_as` para que sqlx mapee automáticamente el resultado a nuestra struct `User`.
    let new_user: Result<User, sqlx::Error> = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, password_hash, email, first_name, last_name) 
        VALUES ($1, $2, $3, $4, $5) 
        RETURNING id, username, password_hash, email, first_name, last_name, role, created_at
        "#,
        username,
        password_hash,
        email,
        first_name,
        last_name
    )
    .fetch_one(&state.db_pool)
    .await;

    match new_user {
        Ok(user) => HttpResponse::Created().json(user),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            HttpResponse::Conflict().body("Username already exists")
        }
        Err(e) => {
            error!("Failed to create user: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create user")
        }
    }
}

/// Maneja las peticiones POST a /login
async fn login(
    state: web::Data<AppState>,
    user_data: web::Json<LoginUser>,
) -> impl Responder {
    // 1. Buscar al usuario por su nombre de usuario.
    // Usamos `fetch_optional` porque el usuario puede no existir.
    let user = match sqlx::query_as!(
        User,
        "SELECT id, username, password_hash, email, first_name, last_name, role, created_at FROM users WHERE username = $1",
        user_data.username
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(user)) => user, // Si se encuentra, `user` es de tipo `User`
        Ok(None) => return HttpResponse::Unauthorized().body("Invalid username or password"),
        Err(_) => return HttpResponse::InternalServerError().body("Something went wrong"),
    };

    // 2. Verificar que la contraseña proporcionada coincide con el hash almacenado.
    let is_password_valid = match verify(&user_data.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => return HttpResponse::InternalServerError().body("Error verifying password"),
    };

    if !is_password_valid {
        return HttpResponse::Unauthorized().body("Invalid username or password");
    }

    // 3. Generar el JWT.
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // El token expira en 24 horas
        .expect("Failed to calculate expiration")
        .timestamp();

    let claims = Claims {
        sub: user.id.to_string(),
        role: user.role,
        exp: expiration as usize,
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to create token"),
    };

    // 4. Devolver el token al cliente.
    HttpResponse::Ok().json(TokenResponse { token })
}

/// Endpoint protegido que devuelve los datos del usuario autenticado.
async fn get_me(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser, // El middleware se ejecuta aquí. Si falla, este handler nunca se llama.
) -> impl Responder {
    // El ID del usuario viene del token validado por el middleware.
    let user_id = auth_user.id;

    match sqlx::query_as!(
        User,
        "SELECT id, username, password_hash, email, first_name, last_name, role, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}

// --- Función Principal ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Carga las variables de entorno desde un archivo .env si existe.
    // Y configura el logger.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenvy::dotenv().ok();

    // Lee la URL de la base de datos desde las variables de entorno.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Crea el pool de conexiones a la base de datos.
    // Este pool se compartirá de forma segura entre todos los hilos del servidor.
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool.");

    info!("🚀 Servidor de autenticación iniciado en http://127.0.0.1:8081");

    HttpServer::new(move || {
        App::new()
            // Middleware de CORS: permite peticiones desde cualquier origen.
            // ¡IMPORTANTE! En producción, esto debería restringirse a dominios específicos.
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(actix_web::middleware::Logger::default())
            // Comparte el estado (el pool de BD) con todos los handlers.
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
            }))
            // Define la ruta y el método para el endpoint de registro.
            .route("/register", web::post().to(register))
            // Define la ruta para el endpoint de login.
            .route("/login", web::post().to(login))
            // Define una ruta protegida.
            .route("/me", web::get().to(get_me))
    })
    .bind(("0.0.0.0", 8080))? // Escucha en todas las interfaces dentro del contenedor.
    .run()
    .await
}
