FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

ARG MAELSTROM_VERSION=0.2.4

# Maelstrom runtime deps:
#   openjdk  - Maelstrom is a Clojure app on the JVM
#   graphviz - Lamport diagrams
#   gnuplot  - latency / rate timeseries plots
#   ruby     - the bundled demo/ruby/echo.rb, useful for verifying the install
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
  && apt-get install -y --no-install-recommends \
  openjdk-17-jre-headless \
  graphviz \
  gnuplot \
  ruby \
  bzip2 \
  curl \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Maelstrom to /opt/maelstrom and symlink the launcher onto PATH
RUN curl -fsSL \
  "https://github.com/jepsen-io/maelstrom/releases/download/v${MAELSTROM_VERSION}/maelstrom.tar.bz2" \
  -o /tmp/maelstrom.tar.bz2 \
  && mkdir -p /opt \
  && tar -xjf /tmp/maelstrom.tar.bz2 -C /opt \
  && rm /tmp/maelstrom.tar.bz2 \
  && ln -s /opt/maelstrom/maelstrom /usr/local/bin/maelstrom \
  && chmod +x /opt/maelstrom/maelstrom

# Handy cargo subcommands
RUN su vscode -c "cargo install cargo-watch cargo-nextest --locked" || true

WORKDIR /workspaces