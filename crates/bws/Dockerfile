###############################################
#                 Build stage                 #
###############################################
FROM rust:1.91-alpine AS build

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
RUN cargo build --release --bin bws

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
