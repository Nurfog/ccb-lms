use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ccb_common::{AuthenticatedUser, UserRole};
use actix_cors::Cors;
use serde::{Deserialize, Serialize}; 
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::env;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};
// --- Modelos de Datos ---

/// Estructura para recibir los datos para crear un curso.
#[derive(Deserialize)]
struct CreateCourse {
    title: String,
    description: Option<String>,
}

/// Estructura para recibir los datos para actualizar un curso. Los campos son opcionales.
#[derive(Deserialize)]
struct UpdateCourse {
    title: Option<String>,
    description: Option<String>,
}

/// Estructura para representar un curso en la base de datos.
#[derive(Serialize, FromRow)]
struct Course {
    id: Uuid,
    title: String,
    description: Option<String>,
    instructor_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// --- Estado de la Aplicación ---

struct AppState {
    db_pool: PgPool,
}

// --- Manejadores de Endpoints ---

async fn create_course(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    course_data: web::Json<CreateCourse>,
) -> impl Responder {
    // Solo los instructores o administradores pueden crear cursos.
    if auth_user.role != UserRole::Instructor && auth_user.role != UserRole::Admin {
        return HttpResponse::Forbidden().body("Only instructors or admins can create courses");
    }

    let new_course = sqlx::query_as!(
        Course,
        r#"
        INSERT INTO courses (title, description, instructor_id)
        VALUES ($1, $2, $3)
        RETURNING id, title, description, instructor_id, created_at, updated_at
        "#,
        course_data.title,
        course_data.description,
        auth_user.id, // Usamos el ID del token validado
    )
    .fetch_one(&state.db_pool)
    .await;

    match new_course {
        Ok(course) => HttpResponse::Created().json(course),
        Err(e) => {
            tracing::error!("Failed to create course: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create course")
        }
    }
}

async fn get_courses(state: web::Data<AppState>) -> impl Responder {
    let courses = sqlx::query_as!(
        Course,
        r#"
        SELECT id, title, description, instructor_id, created_at, updated_at 
        FROM courses
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await;

    match courses {
        Ok(courses) => HttpResponse::Ok().json(courses),
        Err(e) => {
            tracing::error!("Failed to fetch courses: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch courses")
        }
    }
}

async fn get_course_by_id(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let course_id = path.into_inner();

    let course = sqlx::query_as!(
        Course,
        r#"
        SELECT id, title, description, instructor_id, created_at, updated_at 
        FROM courses
        WHERE id = $1
        "#,
        course_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match course {
        Ok(course) => HttpResponse::Ok().json(course),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Course not found"),
        Err(e) => {
            tracing::error!("Failed to fetch course: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch course")
        }
    }
}

async fn update_course_by_id(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<Uuid>,
    update_data: web::Json<UpdateCourse>,
) -> impl Responder {
    let course_id = path.into_inner();
    let instructor_id = auth_user.id;

    // 1. Verificar que el curso existe.
    let course = match sqlx::query_as!(Course, "SELECT * FROM courses WHERE id = $1", course_id)
        .fetch_optional(&state.db_pool)
        .await
    {
        Ok(Some(course)) => course,
        Ok(None) => return HttpResponse::NotFound().body("Course not found"),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // 2. Verificar permisos: solo el instructor que creó el curso o un admin pueden modificarlo.
    if course.instructor_id != instructor_id && auth_user.role != UserRole::Admin {
        return HttpResponse::Forbidden().body("You are not authorized to update this course");
    }

    // 2. Preparar los nuevos datos. Si un campo es None en la petición, se mantiene el valor antiguo.
    let title = update_data.title.clone().unwrap_or(course.title);
    let description = update_data.description.clone();

    // 4. Ejecutar la actualización.
    let updated_course = sqlx::query_as!(
        Course,
        r#"
        UPDATE courses SET title = $1, description = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, title, description, instructor_id, created_at, updated_at
        "#,
        title,
        description,
        course_id
    )
    .fetch_one(&state.db_pool)
    .await;

    match updated_course {
        Ok(course) => HttpResponse::Ok().json(course),
        Err(e) => {
            tracing::error!("Failed to update course: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to update course")
        }
    }
}

async fn delete_course_by_id(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let course_id = path.into_inner();
    let instructor_id = auth_user.id;

    // Para eliminar, requerimos que sea el instructor propietario o un admin.
    // La consulta SQL se simplifica si lo manejamos en el código.
    let query = if auth_user.role == UserRole::Admin {
        sqlx::query!("DELETE FROM courses WHERE id = $1", course_id)
    } else {
        sqlx::query!(
            "DELETE FROM courses WHERE id = $1 AND instructor_id = $2",
            course_id,
            instructor_id
        )
    };

    // Usamos `execute` para borrar, que devuelve el número de filas afectadas.
    let result = query.execute(&state.db_pool).await;

    match result {
        Ok(res) if res.rows_affected() == 1 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().body("Course not found or you are not the owner"),
        Err(e) => {
            tracing::error!("Failed to delete course: {:?}", e);
            HttpResponse::InternalServerError().finish()
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

    info!("🚀 Servidor de cursos iniciado en http://localhost:8082");

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
            // Agrupamos las rutas bajo el scope "/courses"
            .service(
                web::scope("/courses")
                    .route("", web::get().to(get_courses)) // GET /courses
                    .route("", web::post().to(create_course)) // POST /courses
                    .route("/{id}", web::get().to(get_course_by_id)) // GET /courses/{id}
                    .route("/{id}", web::put().to(update_course_by_id)) // PUT /courses/{id}
                    .route("/{id}", web::delete().to(delete_course_by_id)), // DELETE /courses/{id}
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
