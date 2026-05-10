# Blog Project – Full-stack Rust блог с HTTP/gRPC, CLI и WASM фронтендом

## Описание

Проект представляет собой полноценную блог-платформу, реализованную на Rust с использованием современных технологий. Состоит из четырёх крейтов:

- **blog-server** – backend сервер с HTTP API (Actix-web) и gRPC API (Tonic).  
  Использует PostgreSQL, миграции SQLx, JWT-аутентификацию, хеширование паролей Argon2, CORS.
- **blog-client** – библиотека для взаимодействия с сервером, поддерживающая HTTP и gRPC транспорты.
- **blog-cli** – CLI клиент на базе `blog-client` и `clap`.
- **blog-wasm** – фронтенд на Rust, компилируемый в WebAssembly, работает в браузере.

### Архитектура

`domain` (модели),  
`application` (бизнес-логика),  
`data` (репозитории),  
`infrastructure` (БД, JWT, логирование),  
`presentation` (HTTP/gRPC).

---

## Требования

- Rust
    ```bash
    rustc --version
    rustc 1.94.0 (4a4ef493e 2026-03-02)
    ```
    ```bash
    cargo --version
    cargo 1.94.0 (85eff7c80 2026-01-15)
    ```
- Docker (для PostgreSQL) или локальный PostgreSQL
   ```bash
   postgresql@15
   ```
- `wasm-pack` (для сборки WASM)
- `python3` (для запуска простого HTTP-сервера)

---

## Установка и запуск

### 1. PostgreSQL

```bash
docker run -d --name blog-db \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=blog_db \
  -p 5432:5432 \
  postgres:15
```

---

### 2. Настройка окружения

Скопируйте `.env.example` в `.env` в папке `blog-server` и отредактируйте:

```bash
cd blog-server
cp .env.example .env
```

**Основные переменные:**

- `DATABASE_URL` – строка подключения к PostgreSQL  
- `JWT_SECRET` – секретный ключ для JWT (не менее 32 символов)

**Опциональные:**

- `HTTP_PORT`
- `GRPC_PORT`
- `JWT_EXPIRY_HOURS`
- `DB_MAX_CONNECTIONS`
- `DEFAULT_PAGINATION_LIMIT`

---

### 3. Запуск сервера

```bash
cargo run --bin blog-server
```

Сервер запустит:

- HTTP API на порту **3000**
- gRPC API на порту **50051**

Миграции применятся автоматически.

---

## CLI клиент

Все команды используют HTTP по умолчанию (порт 3000).  
Для gRPC добавьте флаг `--grpc` (порт 50051).

### Примеры:

```bash
# Регистрация
cargo run --bin blog-cli -- register \
  --username alice \
  --email alice@example.com \
  --password secret

# Вход
cargo run --bin blog-cli -- login \
  --username alice \
  --password secret

# Создание поста
cargo run --bin blog-cli -- create \
  --title "Hello" \
  --content "World"

# Получение поста
cargo run --bin blog-cli -- get --id 1

# Список постов (пагинация)
cargo run --bin blog-cli -- list --limit 10 --offset 0

# Обновление поста
cargo run --bin blog-cli -- update \
  --id 1 \
  --title "New title" \
  --content "New content"

# Удаление поста
cargo run --bin blog-cli -- delete --id 1

# Использование gRPC
cargo run --bin blog-cli -- --grpc create \
  --title "gRPC post" \
  --content "via tonic"
```

---

## WASM фронтенд
> Важно: перед запуском фронтенда убедитесь, что сервер бэкенда (blog-server) запущен и работает на порту 3000.
### Сборка WASM-модуля

```bash
cd blog-wasm
wasm-pack build --target web
```

### Запуск HTTP-сервера

```bash
python3 -m http.server 8000
```

Откройте браузер по адресу:

```
http://localhost:8000
```

Вы сможете:

- зарегистрироваться
- войти
- создавать / редактировать / удалять посты

Кнопки редактирования и удаления отображаются только для постов текущего пользователя.

---

## Тестирование

### Модульные тесты (не требуют БД)

```bash
cargo test -p blog-server --lib
```

---

### Интеграционные тесты репозиториев

Для запуска интеграционных тестов необходима отдельная тестовая база данных.

```bash
# Создание тестовой базы (если используется Docker)
docker exec -it blog-db psql -U postgres \
  -c "CREATE DATABASE blog_test;"

# Запуск интеграционных тестов
TEST_DATABASE_URL=postgres://postgres:password@localhost/blog_test \
  cargo test -p blog-server --test integration_test -- --nocapture
```

---

### Полный прогон всех тестов workspace

```bash
TEST_DATABASE_URL=postgres://postgres:password@localhost/blog_test \
  cargo test --workspace
```

---

## Примеры сценариев

### HTTP API (curl)

```bash
# Регистрация
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"bob","email":"bob@ex.com","password":"123"}'

# Логин (сохраните token)
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"bob","password":"123"}'

# Создание поста (с токеном)
curl -X POST http://localhost:3000/api/posts \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Curl post","content":"Hello"}'

# Список постов
curl "http://localhost:3000/api/posts?limit=5&offset=0"
```

---

### gRPC (grpcurl)

Установите `grpcurl`, затем:

```bash
# Регистрация
grpcurl -plaintext \
  -d '{"username":"charlie","email":"c@ex.com","password":"pass"}' \
  localhost:50051 blog.BlogService/Register

# Логин
grpcurl -plaintext \
  -d '{"username":"charlie","password":"pass"}' \
  localhost:50051 blog.BlogService/Login

# Создание поста (с токеном в метаданных)
grpcurl -plaintext \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{"title":"gRPC demo","content":"hello"}' \
  localhost:50051 blog.BlogService/CreatePost
```

---

## Структура проекта

```text
blog-project/
├── Cargo.toml                 # workspace
├── blog-server/               # backend
│   ├── migrations/            # SQL миграции
│   ├── proto/                 # protobuf схема
│   └── src/                   # исходники
├── blog-client/               # клиентская библиотека
├── blog-cli/                  # CLI клиент
├── blog-wasm/                 # WASM фронтенд
└── .github/workflows/ci.yaml  # CI
```

---

## CI/CD

Проект настроен на GitHub Actions:

- проверка форматирования (`rustfmt`)
- линтинг (`clippy`)
- модульные тесты
- интеграционные тесты (с PostgreSQL)
- сборка релизной версии

---

## Лицензия

MIT