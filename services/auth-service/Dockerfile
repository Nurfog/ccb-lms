# --- Etapa 1: Compilación ---
# Usamos la imagen oficial de Rust. La nombramos "builder" para poder referenciarla después.
FROM rust:1-slim as builder

# Para compilar un binario estático que funcione en Alpine, necesitamos el target 'musl'.
RUN rustup target add x86_64-unknown-linux-musl

# Instalamos las herramientas de compilación para musl (como musl-gcc) que 'ring' necesita.
RUN apt-get update && apt-get install -y musl-tools

# Creamos un directorio de trabajo
WORKDIR /usr/src/app
 
# Copiamos todo el proyecto
COPY . .
 
# Le decimos a sqlx que use los metadatos pre-generados en lugar de conectarse a la BD.
ENV SQLX_OFFLINE=true

# Compilamos nuestro binario en modo release para el target musl.
RUN cargo build --locked --release --target x86_64-unknown-linux-musl --bin auth-service

# --- Etapa 2: Ejecución ---
# Empezamos desde una imagen base de Alpine, que es extremadamente ligera.
FROM alpine:latest

# Copiamos el binario compilado desde la etapa "builder".
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/auth-service /usr/local/bin/auth-service

# Exponemos el puerto que usará nuestra aplicación
EXPOSE 8080

# Comando para ejecutar el servicio cuando el contenedor inicie.
CMD ["auth-service"]