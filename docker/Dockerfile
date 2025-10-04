# Stage 1
FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y \
  && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-deb \
  && cargo install cargo-aur --version 1.7.1 \
  && cargo install cargo-generate-rpm

RUN cargo build --release

RUN cargo deb

RUN cargo aur

RUN cargo generate-rpm

# Stage 2
FROM archlinux:latest AS aur-builder

WORKDIR /app

RUN pacman -Syu --noconfirm base-devel

# Create a non-root user for makepkg
RUN useradd -m -s /bin/bash builder

COPY --from=builder /app/target/cargo-aur /app/target/cargo-aur

# Change ownership to the builder user
RUN chown -R builder:builder /app/target/cargo-aur

# Switch to builder user and run makepkg
USER builder
RUN cd target/cargo-aur && makepkg --noconfirm

# Final stage
FROM rust:latest

WORKDIR /app

COPY --from=builder /app/target/debian /app/target/debian
COPY --from=builder /app/target/generate-rpm /app/target/generate-rpm
COPY --from=aur-builder /app/target/cargo-aur /app/target/cargo-aur

CMD ["tail", "-f", "/dev/null"]
