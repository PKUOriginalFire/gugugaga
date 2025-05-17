FROM rust:bullseye AS builder

COPY ./gugugaga /app
WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release && cp target/release/gugugaga .

FROM ubuntu:24.04

RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && \
    apt-get install -y zlib1g-dev fuse libfuse2 \
    libnss3 libgtk-3-0 libgbm1 libasound2t64 python3-uinput \
    fonts-wqy-microhei fonts-wqy-zenhei ttf-wqy-zenhei \
    xpra dunst dbus-x11 xauth ffmpeg curl gosu \
    libx264-dev libvpx-dev libmp3lame-dev libx265-dev \
    libcurl4 libcurl3-gnutls \
    && apt-get clean

# 创建必要的目录并设置权限
RUN mkdir -p /home/xuser/ && \
    curl -o /home/xuser/QQ.AppImage https://dldir1.qq.com/qqfile/qq/QQNT/Linux/QQ_3.2.17_250429_x86_64_01.AppImage

RUN chmod +x /home/xuser/QQ.AppImage

RUN useradd -m xuser
RUN /home/xuser/QQ.AppImage --appimage-extract && \
    mv squashfs-root /home/xuser/QQ

COPY ./start.sh /home/xuser/start.sh
COPY --from=builder /app/gugugaga /home/xuser/gugugaga
RUN chmod +x /home/xuser/start.sh && \
    chown xuser:xuser /home/xuser -R && \
    chown -R root:root /home/xuser/QQ/chrome-sandbox && \
    chmod 4755 /home/xuser/QQ/chrome-sandbox

RUN mkdir -p /run/user/$(id -u xuser) && \
    chown -R xuser:xuser /run/user/$(id -u xuser)

WORKDIR /home/xuser

EXPOSE 14500

CMD ["/home/xuser/start.sh"]