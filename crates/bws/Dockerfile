###############################################
#                 Build stage                 #
###############################################
FROM --platform=$BUILDPLATFORM rust:1.86.0-alpine3.21 AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

# Add dev libraries. Make a user and HOME directory for the app stage
RUN apk add --no-cache musl-dev && adduser -D app

WORKDIR /app/crates/bws
COPY . /app

# Build project

RUN cargo build --release --bin bws

###############################################
#                  App stage                  #
###############################################
FROM scratch

ARG TARGETPLATFORM
LABEL com.bitwarden.product="bitwarden"

# Set a HOME directory and copy the user file
# Must chown home folder, otherwise bws config file writing will cause permission errors in output
COPY --from=build --chown=app:app /home/app /home/app
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
ENV HOME=/home/app
WORKDIR /home/app

# Switch to the app user
USER app

# Copy built project from the build stage
COPY --from=build /app/target/release/bws /bin/bws

# Copy certs
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENTRYPOINT ["bws"]
