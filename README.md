# Collaborative Course Builder (CCB) - Un LMS basado en Microservicios

¡Bienvenido al Collaborative Course Builder (CCB)! Este proyecto es un moderno y escalable Sistema de Gestión de Aprendizaje (LMS) construido con una arquitectura de microservicios utilizando Rust.

## Objetivo del Proyecto

El objetivo principal es crear un backend robusto y de alto rendimiento para una plataforma educativa, aprovechando la seguridad y velocidad de Rust, la eficiencia de Actix Web y la fiabilidad de PostgreSQL, todo containerizado con Docker para un fácil desarrollo y despliegue.

## Stack Tecnológico

*   **Lenguaje**: Rust (Edición 2021)
*   **Framework Web**: Actix Web
*   **Base de Datos**: PostgreSQL
*   **Toolkit de BD**: SQLx
*   **Containerización**: Docker & Docker Compose
*   **Runtime Asíncrono**: Tokio

---

## Servicios

El proyecto se divide en varios microservicios independientes que se comunican a través de una red.

### 1. Servicio de Autenticación (`auth-service`)

*   **Descripción**: Responsable de todas las tareas de autenticación y gestión de usuarios. Maneja el registro, el inicio de sesión y la emisión de JSON Web Tokens (JWT) para asegurar la API.
*   **Puerto Local**: `8081`
*   **Endpoints**:
    *   `POST /register`: Registra un nuevo usuario.
    *   `POST /login`: Inicia sesión y devuelve un JWT.
    *   `GET /me`: (Ruta protegida) Devuelve la información del usuario autenticado.

*   **Ejemplos de uso con `curl`**:

    *   **Registrar un usuario:**
        ```bash
        curl -X POST http://localhost:8081/register \
        -H "Content-Type: application/json" \
        -d '{
          "username": "test_instructor",
          "password": "secure_password_123",
          "email": "instructor@example.com",
          "first_name": "Test",
          "last_name": "Instructor"
        }'
        ```

    *   **Iniciar sesión:**
        ```bash
        curl -X POST http://localhost:8081/login \
        -H "Content-Type: application/json" \
        -d '{
          "username": "test_instructor",
          "password": "secure_password_123"
        }'
        ```

    *   **Obtener datos del usuario autenticado:**
        ```bash
        # Reemplaza <TU_TOKEN_JWT> con el token obtenido en el login
        curl -X GET http://localhost:8081/me \
        -H "Authorization: Bearer <TU_TOKEN_JWT>"
        ```

### 2. Servicio de Cursos (`course-service`)

*   **Descripción**: Gestionará la creación, el contenido y los metadatos de los cursos. Actualmente es un placeholder.
*   **Puerto Local**: (Aún no asignado)
*   **Descripción**: Gestiona la creación, el contenido y los metadatos de los cursos.
*   **Puerto Local**: `8082`
*   **Endpoints**:
    *   `POST /courses`: (Ruta protegida) Crea un nuevo curso.
    *   `GET /courses`: Devuelve una lista de todos los cursos.
    *   `GET /courses/{id}`: Devuelve los detalles de un curso específico.
    *   `PUT /courses/{id}`: (Ruta protegida) Actualiza un curso.
    *   `DELETE /courses/{id}`: (Ruta protegida) Elimina un curso.

*   **Ejemplos de uso con `curl`**:

    *   **Crear un curso (requiere token):**
        ```bash
        # Reemplaza <TU_TOKEN_JWT> con el token obtenido en el login
        curl -X POST http://localhost:8082/courses \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer <TU_TOKEN_JWT>" \
        -d '{
          "title": "Introducción a Rust",
          "description": "Un curso completo sobre los fundamentos de Rust."
        }'
        ```

    *   **Listar todos los cursos:**
        ```bash
        curl -X GET http://localhost:8082/courses
        ```

    *   **Obtener un curso específico:**
        ```bash
        # Reemplaza <ID_DEL_CURSO> con el ID de un curso existente
        curl -X GET http://localhost:8082/courses/<ID_DEL_CURSO>
        ```

    *   **Actualizar un curso (requiere token del propietario):**
        ```bash
        # Reemplaza <ID_DEL_CURSO> y <TU_TOKEN_JWT>
        curl -X PUT http://localhost:8082/courses/<ID_DEL_CURSO> \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer <TU_TOKEN_JWT>" \
        -d '{
          "title": "Rust Avanzado: Genéricos y Tiempos de Vida"
        }'
        ```

    *   **Eliminar un curso (requiere token del propietario):**
        ```bash
        # Reemplaza <ID_DEL_CURSO> y <TU_TOKEN_JWT>
        curl -X DELETE http://localhost:8082/courses/<ID_DEL_CURSO> \
        -H "Authorization: Bearer <TU_TOKEN_JWT>"
        ```

### 3. Servicio de Inscripciones (`enrollment-service`)

*   **Descripción**: Maneja la lógica de inscripción de los usuarios en los cursos.
*   **Puerto Local**: `8083`
*   **Endpoints**:
    *   `POST /enrollments`: (Ruta protegida) Inscribe al usuario autenticado en un curso.
    *   `GET /enrollments/my-courses`: (Ruta protegida) Devuelve una lista de los cursos en los que el usuario está inscrito.

*   **Ejemplos de uso con `curl`**:

    *   **Inscribir un usuario en un curso (requiere token):**
        ```bash
        # Reemplaza <ID_DEL_CURSO> y <TU_TOKEN_JWT>
        curl -X POST http://localhost:8083/enrollments \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer <TU_TOKEN_JWT>" \
        -d '{
          "course_id": "<ID_DEL_CURSO>"
        }'
        ```

    *   **Listar los cursos en los que estoy inscrito (requiere token):**
        ```bash
        # Reemplaza <TU_TOKEN_JWT>
        curl -X GET http://localhost:8083/enrollments/my-courses \
        -H "Authorization: Bearer <TU_TOKEN_JWT>"
        ```

### 4. Base de Datos (`db`)

*   **Descripción**: Una instancia de PostgreSQL 16 que sirve como la capa de persistencia de datos para todos los servicios.
*   **Puerto Local**: `5432`

---
## Panel de Pruebas Frontend

Para facilitar las pruebas de los microservicios, se ha incluido un panel de pruebas frontend muy simple. Una vez que el entorno esté levantado con `docker-compose up`, puedes acceder a él en tu navegador:

*   **URL**: http://localhost:8000

Este panel te permite registrar usuarios, iniciar sesión para obtener un token JWT, crear y listar cursos, e inscribirte en ellos, mostrando la respuesta del API para cada acción.

---

## Cómo Empezar

Sigue estos pasos para levantar el entorno de desarrollo local.

### Prerrequisitos

*   Docker
*   Docker Compose
*   Rust y Cargo
*   `sqlx-cli` (`cargo install sqlx-cli`)

### Pasos de Instalación

1.  **Clonar el repositorio.**

2.  **Crear el archivo de entorno**:
    Crea un archivo `.env` en la raíz del proyecto con el siguiente contenido:
    ```
    DATABASE_URL=postgres://lms_user:lms_password@localhost:5432/lms_db
    ```

3.  **Iniciar la base de datos**:
    ```bash
    docker-compose up -d db
    ```

4.  **Aplicar las migraciones de la base de datos**:
    ```bash
    cargo sqlx migrate run
    ```

5.  **Levantar todos los servicios**:
    ```bash
    docker-compose up --build
    ```
    El servicio de autenticación estará disponible en `http://localhost:8081`.

---

## Roadmap del Proyecto

- [x] **Fase 1: Fundación y Autenticación**
  - [x] Configuración del workspace de Rust y Docker.
  - [x] Implementación del endpoint de registro de usuarios (`/register`).
  - [x] Implementación del endpoint de login (`/login`) con generación de JWT.
  - [x] Middleware para proteger rutas usando JWT.
- [x] **Fase 2: Servicio de Cursos (Básico)**
  - [x] Implementar endpoint `POST /courses` para crear cursos (protegido por JWT).
  - [x] Implementar endpoint `GET /courses` para listar todos los cursos.
  - [x] Implementar endpoint `GET /courses/{id}` para obtener un curso específico.
  - [x] Implementar endpoint `PUT /courses/{id}` para actualizar un curso.
  - [x] Implementar endpoint `DELETE /courses/{id}` para eliminar un curso.
- [x] **Fase 3: Servicio de Inscripciones (Básico)**
  - [x] Crear servicio y endpoint `POST /enrollments` para inscribir usuarios.
  - [x] Implementar endpoint `GET /enrollments/my-courses` para listar los cursos de un usuario.
- [x] **Fase 4: Funcionalidades Avanzadas**
  - [x] Roles y permisos de usuario (Admin, Instructor, Estudiante).
  - [ ] Creación de un servicio de Quizzes/Exámenes.