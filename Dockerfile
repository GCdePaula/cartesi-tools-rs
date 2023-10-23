FROM cartesi/machine-emulator:0.15.2

USER 0
RUN apt-get -y update; apt-get -y install curl git build-essential; apt-get install -y procps
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . .
WORKDIR ./examples/echo/

ENTRYPOINT ["cargo", "run"]
