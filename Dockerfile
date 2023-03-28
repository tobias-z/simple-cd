FROM rust:1.68.0-slim as builder
WORKDIR /build
COPY . .
RUN rustup default nightly
# RUN apt update && apt install pkg-config libssl-dev -y
RUN cargo install --path .

FROM debian:11-slim

# Install git and docker
RUN apt-get update && apt-get install -y \
    git \
    ca-certificates \
    curl \
    gnupg
RUN mkdir -m 0755 -p /etc/apt/keyrings
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
RUN echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null
RUN apt-get update
RUN apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

COPY --from=builder /usr/local/cargo/bin/simple-server-deployment /usr/local/bin/simple-server-deployment
RUN mkdir -p /etc/simple-server-deployment/checkouts/ \
    && mkdir -p /etc/simple-server-deployment/conf/

ENV ROCKET_ADDRESS=0.0.0.0

CMD ["/usr/local/bin/simple-server-deployment"]
