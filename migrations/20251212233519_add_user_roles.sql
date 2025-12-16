-- Add migration script here
-- 1. Crear un nuevo tipo ENUM para los roles de usuario.
CREATE TYPE user_role AS ENUM ('student', 'instructor', 'admin');

-- 2. Añadir la columna 'role' a la tabla 'users'.
-- Asignaremos el rol 'student' por defecto a todos los usuarios existentes y nuevos.
ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'student';
