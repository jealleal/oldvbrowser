FROM rust:1.81 as builder

WORKDIR /app

# Копируем манифесты
COPY Cargo.toml Cargo.lock ./

# Копируем исходный код
COPY src ./src

# Собираем в релиз режиме
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

# Устанавливаем необходимые зависимости и Chromium
RUN apt-get update && apt-get install -y \
    chromium-browser \
    libnss3 \
    libxss1 \
    libappindicator1 \
    libindicator7 \
    fonts-liberation \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем собранный бинарь из builder
COPY --from=builder /app/target/release/roblox_browser /app/

# Переменные окружения для headless_chrome
ENV CHROME_PATH=/usr/bin/chromium-browser
ENV CHROMIUM_PATH=/usr/bin/chromium-browser

# Порт
EXPOSE 3000

# Запуск
CMD ["/app/roblox_browser"]
