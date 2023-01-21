#Build
from rust:1.66.1-slim AS builder
WORKDIR /app/build/
COPY src ./src
COPY Cargo.lock Cargo.toml ./
RUN cargo build --release

#RUN
from rust:1.58.1-slim
COPY --from=builder /app/build/target/release/spacechains /app/spacechains
ENTRYPOINT ["/app/spacechains"]
