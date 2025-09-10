###############################################
#                 Build stage                 #
###############################################
FROM rust:1.85-alpine AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

# Install build dependencies
RUN apk add --no-cache ca-certificates gcc musl-dev

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

# Create app user
RUN adduser -D -u 1000 app

###############################################
#                  App stage                  #
###############################################
FROM scratch

LABEL com.bitwarden.product="bitwarden"

# Copy user data from the build stage
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build --chown=app:app /home/app /home/app
ENV HOME=/home/app
WORKDIR /home/app

# Copy bws from the build stage
COPY --from=build /source/crates/bws/target/release/bws /bin/bws

# Copy CA certificates
COPY --from=build /etc/ssl /etc/ssl

# Switch to the app user
USER app

ENTRYPOINT ["bws"]
