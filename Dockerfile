FROM rust:latest

WORKDIR /app

# Copy Cargo.toml and create a dummy main.rs to build dependencies
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy main.rs
RUN rm src/main.rs

# Copy the actual source code
COPY src/ ./src/

# Rebuild the application
RUN cargo build --release

CMD ["cargo", "run", "--release"]
