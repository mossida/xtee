FROM arm64v8/ubuntu:latest

RUN apt-get -qq update
RUN apt-get install -y -q \
    build-essential \
    nodejs \
    npm \
    curl

RUN apt-get install -y gcc-aarch64-linux-gnu

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add aarch64-unknown-linux-gnu && \
    cargo install tauri-cli

# Install Tauri dependencies
RUN apt-get install -y \
libwebkit2gtk-4.1-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev

# -------------- [End of install tauri] ----------------

RUN mkdir -p /project
WORKDIR /project

CMD ["/bin/bash"]