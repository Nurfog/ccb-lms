-- Add migration script here
-- Crea la tabla de inscripciones (enrollments)
CREATE TABLE enrollments (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    enrollment_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, course_id)
);
-- Fin del script de migración
