-- Add migration script here
-- Habilita la extensión para generar UUIDs
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Esta extensión permite la generación de UUIDs en PostgreSQL
-- para ser utilizada en tablas y otros objetos de la base de datos.
-- Asegúrate de ejecutar este script con los permisos adecuados
-- para crear extensiones en la base de datos.
-- Fin del script de migración