# Base image: lightweight Debian 12 slim variant with minimal footprint
FROM debian:bookworm-slim

# Install required system dependencies
# ca-certificates: SSL root certificates for secure HTTPS file downloads
# jq: CLI JSON processor, used by LVM to parse remote version API responses
# libterm-readline-perl-perl: Perl terminal library for command history & tab completion
# curl / wget: HTTP download utilities for fetching install scripts, Node.js & Go binaries
# sudo: Privilege escalation tool to run admin operations under regular user accounts
# zsh: Enhanced interactive shell that auto sources LVM environment configurations
RUN apt-get update && apt-get install -y ca-certificates jq libterm-readline-perl-perl curl sudo wget zsh \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* # Clear apt cache to shrink final image size

# Domestic mirror download URL (commented out for optional use)
# ENV LVM_DOWNLOAD_URL="https://cnb.cool/zhiqiangwang/lvm/-/releases/latest/download"

# Copy LVM installation script to temporary directory inside container
ADD install.sh /tmp/install.sh
# Make script executable, run LVM installer, delete temp script to reduce image bloat
RUN chmod +x /tmp/install.sh && bash /tmp/install.sh && rm -rf /tmp/install.sh

# Install Node.js 20 and latest stable Go via LVM version manager
RUN lvm install node 20 \
    && lvm install go

# Configure zsh startup file to load LVM environment variables & lifecycle hooks automatically on shell launch
RUN cat <<'EOF' >> ~/.zshrc
# Load LVM version manager environment variables
eval "$(lvm env)"
# Load LVM auto-switching and path injection hook logic
eval "$(lvm hook)"
EOF

# Configure bash startup file for bash interactive shell compatibility
RUN cat <<'EOF' >> ~/.bashrc
# Load LVM version manager environment variables
eval "$(lvm env)"
# Load LVM auto-switching and path injection hook logic
eval "$(lvm hook)"
EOF

# ---------------------- Build & Run Command Reference ----------------------
# Build image command: docker build -t lvm:latest .
# Launch interactive zsh shell container (recommended): docker run -it --name lvm --rm lvm:latest zsh
# Launch interactive bash shell container: docker run -it --name lvm --rm lvm:latest bash
# Append zsh/bash after image tag to override default entrypoint and drop into interactive terminal
# Default container entrypoint: Print installed LVM manager version
CMD ["/usr/local/bin/lvm", "--version"]
