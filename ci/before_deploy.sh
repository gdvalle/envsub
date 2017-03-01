# This script takes care of building your crate and packaging it for release

BIN="envsub"

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin "$BIN" --target $TARGET --release -- -C lto -C "panic=abort"

    cp "target/$TARGET/release/$BIN" $stage/

    # Strip and compress the executable.
    # Binary strip will fail on non x86 platforms because we're using the
    # host's strip binary.
    strip -s "$stage/$BIN" || echo "Failed stripping binary; proceeding anyway"
    # Some platforms, like mips, return an UnknownExecutableFormatException.
    upx --brute "$stage/$BIN" || echo "Failed compressing binary; proceeding anyway"

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
