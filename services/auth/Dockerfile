FROM rust:1.75-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev cargo

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory to build dependencies
RUN mkdir -p src/bin
RUN echo 'fn main() {}' > src/bin/dummy.rs

# Build dependencies only
RUN cargo build --release --bin dummy

# Remove dummy file
RUN rm -f src/bin/dummy.rs

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Create final image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache libssl3 libcrypto3

# Copy the binary
COPY --from=builder /app/target/release/auth /usr/local/bin/auth

# Create non-root user
RUN addgroup -g 1001 -S drmp && \
    adduser -S drmp -u 1001 -G drmp

USER drmp

EXPOSE 8080

CMD ["/usr/local/bin/auth"]