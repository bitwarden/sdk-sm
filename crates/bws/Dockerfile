###############################################
#                 Build stage                 #
###############################################
FROM --platform=$BUILDPLATFORM rust:1.82 AS build

# Docker buildx supplies the value for this arg
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Copy required project files
COPY . /app

# Build project
WORKDIR /app/crates/bws
RUN cargo build --release --bin bws

# Bundle bws dependencies
RUN mkdir /lib-bws
RUN mkdir /lib64-bws

RUN ldd /app/target/release/bws | tr -s '[:blank:]' '\n' | grep '^/lib' | xargs -I % cp % /lib-bws
RUN ldd /app/target/release/bws | tr -s '[:blank:]' '\n' | grep '^/lib64' | xargs -I % cp % /lib64-bws

# Make a user and HOME directory for the app stage
RUN useradd -m app

###############################################
#                  App stage                  #
###############################################
FROM scratch

ARG TARGETPLATFORM
LABEL com.bitwarden.product="bitwarden"

# Set a HOME directory and copy the user file
COPY --from=build /home/app /home/app
COPY --from=build /etc/passwd /etc/passwd
ENV HOME=/home/app
WORKDIR /home/app

# Switch to the app user
USER app

# Copy built project from the build stage
COPY --from=build /app/target/release/bws /bin/bws

# Copy certs
COPY --from=build /etc/ssl/certs /etc/ssl/certs

# Copy bws dependencies
COPY --from=build /lib-bws /lib
COPY --from=build /lib64-bws /lib64

ENTRYPOINT ["bws"]
