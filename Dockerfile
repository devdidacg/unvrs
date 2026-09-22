FROM archlinux:latest

RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm rust git base-devel && \
    pacman -Scc --noconfirm

WORKDIR /build
COPY . .

RUN cargo build --release && \
    cp target/release/unvrs /usr/local/bin/

RUN unvrs --version

CMD ["unvrs", "doctor"]
