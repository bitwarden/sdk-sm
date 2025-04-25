###############################################
#                 Build stage                 #
###############################################
FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/cargo-zigbuild:latest@sha256:b3b422171a9e2eacc5e4d5c6eb0d95c2c7e65cad2a62ea6938ae9805ce78df3e AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

# Install ca-certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates && \
  rm -rf /var/lib/apt/lists/*

# Copy required project files
COPY . /source

# Set required args for build
ENV CARGO_TARGET_DIR='./target'

# Build project
WORKDIR /source/crates/bws
RUN <<EOF
  # Building for $TARGETPLATFORM
  case "$TARGETPLATFORM" in
    *"linux/amd64"*)
      cargo zigbuild --release --bin bws --target x86_64-unknown-linux-musl
      mv ./target/x86_64-unknown-linux-musl/release/bws ./target/release/bws
      ;;
    *"linux/arm64"*)
      cargo zigbuild --release --bin bws --target aarch64-unknown-linux-musl
      mv ./target/aarch64-unknown-linux-musl/release/bws ./target/release/bws
      ;;
    *)
      echo "Unsupported target platform: $TARGETPLATFORM"
      exit 1
      ;;
  esac
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

# Copy certs
COPY --from=build /etc/ssl/certs /etc/ssl/certs

ENTRYPOINT ["bws"]
