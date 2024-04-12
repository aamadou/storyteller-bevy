FROM x11docker/xfce
# as rust-base

# NB: This can be "stable" or another version.
ARG RUST_TOOLCHAIN="1.75.0"

# NB: cmake is required for freetype-sys-0.13.1, which in turn has only been added for egui.
# NB: fontconfig is required by servo-fontconfig-sys, which is in the dependency chain for egui.
# NB: libfontconfig-dev is required by servo-fontconfig-sys, which is in the dependency chain for egui.
# NB: pkg-config and libssl are for container TLS; we may switch to rustls in the future.
# BEVY SPECIFIC -
# NB: libasound2-dev for alsa-sys
# NB: libudev-dev for libudev-sys
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC apt-get install -y \
        build-essential \
        cmake \
        mesa-utils \
        curl \
        fontconfig \
        libfontconfig1-dev \
        libssl-dev \
        pkg-config \
        g++ \
        pkg-config \
        libx11-dev \
        libasound2-dev \
        libudev-dev \
        libxkbcommon-x11-0 \
        libwayland-dev \
        libxkbcommon-dev \
        mesa-vulkan-drivers \
        xterm

ARG USERNAME=ubuntu
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    #
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh  -s -- --default-toolchain $RUST_TOOLCHAIN -y

# Report Rust version for build debugging
RUN $HOME/.cargo/bin/rustup show
RUN $HOME/.cargo/bin/rustc --version
RUN $HOME/.cargo/bin/cargo --version

WORKDIR /src

COPY Cargo.lock .
COPY Cargo.toml .

COPY . .

ARG WORKDIR=/storyteller
ENV CARGO_TARGET_DIR=$WORKDIR

RUN LD_LIBRARY_PATH=/usr/lib:${LD_LIBRARY_PATH} \
  $HOME/.cargo/bin/cargo build \
  --release \
  --bin studio-headless

RUN cp runner.sh $WORKDIR

WORKDIR $WORKDIR

RUN mv release/studio-headless $WORKDIR/studio-headless

COPY --from=inference-job-local inference-job $WORKDIR/inference-job

RUN chown -R $USERNAME $WORKDIR 

USER $USERNAME
