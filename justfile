RAM_LENGTH := "128Mi"

download-deps:
  wget https://github.com/cartesi/image-kernel/releases/download/v0.20.0/linux-6.5.13-ctsi-1-v0.20.0.bin \
    -O ./out/linux.bin

clean-deps:
  rm -f ./out/linux.bin

build-dapp:
  mkdir -p ./out
  DOCKER_DEFAULT_PLATFORM=linux/amd64 cross build --bin echo-dapp --target riscv64gc-unknown-linux-musl --release
  mv target/riscv64gc-unknown-linux-musl/release/echo-dapp ./out/dapp

build-rootfs: build-dapp
  mkdir -p ./out
  docker buildx build \
    --platform linux/riscv64 \
    --output type=tar,dest=./out/rootfs.tar \
    --file Dockerfile \
    .
  xgenext2fs -f -z -B 4096 -i 4096 -r +4096 -a ./out/rootfs.tar -L rootfs ./out/rootfs.ext2

clean-cartesi-image:
  rm -rf ./out/machine-image

build-cartesi-image: clean-cartesi-image build-rootfs
  cartesi-machine \
    --ram-length={{RAM_LENGTH}} \
    --ram-image=./out/linux.bin \
    --flash-drive=label:root,data_filename:./out/rootfs.ext2 \
    --append-init=WORKDIR="/dapp" \
    --append-entrypoint="/dapp/dapp" \
    --assert-rolling-template --final-hash \
    --store=./out/machine-image


test: build-cartesi-image
  cargo run --bin echo-test -- --nocapture
