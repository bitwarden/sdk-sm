###############################################
#                 Build stage                 #
###############################################
FROM rust:1.85-alpine AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

# Install build dependencies
RUN apk add --no-cache musl-dev gcc

# Copy required project files
COPY . /source

# Set required args for build
ENV CARGO_TARGET_DIR='./target'

# Build project
WORKDIR /source/crates/bws
RUN <<EOF
  # Add Rust toolchains based on $TARGETPLATFORM
  case "$TARGETPLATFORM" in
    *"linux/amd64"*)
      rustup target add x86_64-unknown-linux-musl
      TARGET="x86_64-unknown-linux-musl"
      ;;
    *"linux/arm64"*)
      rustup target add aarch64-unknown-linux-musl
      TARGET="aarch64-unknown-linux-musl"
      ;;
    *)
      echo "Unsupported target platform: $TARGETPLATFORM"
      exit 1
      ;;
  esac

  # Build
  cargo build --release --bin bws --target $TARGET
  mv "./target/$TARGET/release/bws" "./target/release/bws"
EOF

# Create the config directory to avoid permission issues
RUN mkdir -p /home/app/.config/bws

###############################################
#                  App stage                  #
###############################################
FROM scratch

LABEL com.bitwarden.product="bitwarden"

# Create minimal /etc/passwd file and user
COPY --chmod=644 --chown=0:0 <<EOF /etc/passwd
app:x:1000:1000::/home/app:/bin/false
EOF
COPY --chmod=700 --chown=1000:1000 --from=build /home/app /home/app
ENV HOME=/home/app
WORKDIR /home/app

# Switch to app user
USER app

# Copy bws from the build stage
COPY --from=build /source/crates/bws/target/release/bws /bin/bws

# Copy certs from Alpine
COPY --from=build /etc/ssl/certs /etc/ssl/certs

ENTRYPOINT ["bws"]
