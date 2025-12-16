use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ccb_common::AuthenticatedUser;
use actix_cors::Cors;
use serde::{Deserialize, Serialize}; 
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::env;
use tracing::{error, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// --- Modelos de Datos ---

#[derive(Deserialize)]
struct EnrollmentRequest {
    course_id: Uuid,
}

#[derive(Serialize, FromRow)]
struct Enrollment {
    user_id: Uuid,
    course_id: Uuid,
    enrollment_date: DateTime<Utc>,
}

/// Estructura para devolver los detalles de un curso en el que el usuario está inscrito.
#[derive(Serialize, FromRow)]
struct EnrolledCourseDetails {
    course_id: Uuid,
    title: String,
    description: Option<String>,
    enrollment_date: DateTime<Utc>,
}

struct AppState {
    db_pool: PgPool,
}

async fn enroll_in_course(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    enrollment_data: web::Json<EnrollmentRequest>,
) -> impl Responder {
    let user_id = auth_user.id;
    let course_id = enrollment_data.course_id;

    let new_enrollment = sqlx::query_as!(
        Enrollment,
        "INSERT INTO enrollments (user_id, course_id) VALUES ($1, $2) RETURNING user_id, course_id, enrollment_date",
        user_id,
        course_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match new_enrollment {
        Ok(enrollment) => HttpResponse::Created().json(enrollment),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            HttpResponse::Conflict().body("User is already enrolled in this course")
        }
        Err(e) => {
            error!("Failed to enroll user in course: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to enroll in course")
        }
    }
}

/// Maneja las peticiones GET a /enrollments/my-courses
async fn get_my_enrollments(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    let user_id = auth_user.id;

    // Hacemos un JOIN entre las tablas `enrollments` y `courses` para obtener los detalles.
    let enrolled_courses = sqlx::query_as!(
        EnrolledCourseDetails,
        r#"
        SELECT 
            c.id as "course_id!",
            c.title as "title!",
            c.description,
            e.enrollment_date as "enrollment_date!"
        FROM enrollments e
        JOIN courses c ON e.course_id = c.id
        WHERE e.user_id = $1
        ORDER BY e.enrollment_date DESC
        "#,
        user_id
    )
    .fetch_all(&state.db_pool)
    .await;

    match enrolled_courses {
        Ok(courses) => HttpResponse::Ok().json(courses),
        Err(e) => {
            error!("Failed to fetch user enrollments: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to retrieve your enrollments")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool.");

    info!("🚀 Servidor de inscripciones iniciado en http://localhost:8083");

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
            .app_data(web::Data::new(AppState { db_pool: db_pool.clone() }))
            .service(
                web::scope("/enrollments")
                    .route("", web::post().to(enroll_in_course))
                    .route("/my-courses", web::get().to(get_my_enrollments)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
