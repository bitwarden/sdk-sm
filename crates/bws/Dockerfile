###############################################
#                 Build stage                 #
###############################################
FROM rust:1.81 AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Copy required project files
COPY . /app

# Set required args for build
ENV CARGO_TARGET_DIR='./target'
ENV RUSTFLAGS='-C target-feature=+crt-static'

# Build project
WORKDIR /app/crates/bws
RUN <<EOF
  # Explicit target args are required: https://github.com/rust-lang/rust/issues/78210#issuecomment-714776007
  case "$TARGETPLATFORM" in
    *"linux/amd64"*)
      cargo build --release --bin bws --target x86_64-unknown-linux-gnu
      mv ./target/x86_64-unknown-linux-gnu/release/bws ./target/release/bws;;
    *"linux/arm64"*)
      cargo build --release --bin bws --target aarch64-unknown-linux-gnu
      mv ./target/aarch64-unknown-linux-gnu/release/bws ./target/release/bws;;
    *)
      echo "Unsupported target platform: $TARGETPLATFORM";
      exit 1;;
  esac
EOF

# Make a user and HOME directory for the app stage
RUN useradd -m app

###############################################
#                  App stage                  #
###############################################
FROM scratch

LABEL com.bitwarden.product="bitwarden"

# Set a HOME directory and copy the user file
COPY --from=build /home/app /home/app
COPY --from=build /etc/passwd /etc/passwd
ENV HOME=/home/app
WORKDIR /home/app

# Switch to the app user
USER app

# Copy built project from the build stage
COPY --from=build /app/crates/bws/target/release/bws /bin/bws

# Copy certs
COPY --from=build /etc/ssl/certs /etc/ssl/certs

ENTRYPOINT ["bws"]
