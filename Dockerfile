# Base image: lightweight Debian 12
FROM debian:bookworm-slim

# Install system dependencies
# ca-certificates: SSL root certificates for HTTPS requests
# jq: CLI JSON parser, used by LVM to parse version API responses
# libterm-readline-perl-perl: Perl terminal library for command history & tab completion
# curl / wget: HTTP download tools for fetching scripts, Node & Go binaries
# sudo: Privilege escalation utility for running admin commands as regular user
# zsh: Enhanced interactive shell to auto-load LVM environment config
RUN apt-get update && apt-get install -y ca-certificates jq libterm-readline-perl-perl curl sudo wget zsh \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* # Clear apt cache to reduce image size

# Domestic download address
# ENV LVM_DOWNLOAD_URL="https://cnb.cool/zhiqiangwang/lvm/-/releases/latest/download"

# Copy and run LVM installation script, delete temp script after installation
ADD install.sh /tmp/install.sh
RUN chmod +x /tmp/install.sh && bash /tmp/install.sh && rm -rf /tmp/install.sh

# Install Node.js 20 and Go via LVM version manager
RUN lvm install node 20 \
    && lvm install go

# Configure zsh to automatically load LVM environment variables and hooks on shell startup
RUN cat <<'EOF' >> ~/.zshrc
# Load LVM Manager
eval "$(lvm env)"
eval "$(lvm hook)"
EOF

# Build image command: docker build -t lvm:latest .
# Default container command: print LVM version
# Run interactive zsh shell command: docker run -it --name lvm --rm lvm:latest zsh
# Append bash/zsh when running docker run to override and enter interactive shell
CMD ["/usr/local/bin/lvm", "--version"]