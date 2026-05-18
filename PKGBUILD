# Maintainer: Bukutsu <bukutsu@users.noreply.github.com>
pkgname=frost-tune
pkgver=0.9.5
pkgrel=1
pkgdesc="Native parametric EQ editor for USB DACs"
arch=('x86_64')
url="https://github.com/Bukutsu/frost-tune"
license=('MIT')
depends=('polkit')
makedepends=('rust' 'cargo' 'desktop-file-utils' 'mold')
provides=('frost-tune')
conflicts=('frost-tune-bin' 'frost-tune-git')
install=packaging/arch/frost-tune.install
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

# Redirect to packaging/arch/PKGBUILD if run from the repository root
if [ "$PWD" = "$startdir" ] && [ -f "packaging/arch/PKGBUILD" ]; then
    echo ">>> Redirecting makepkg to packaging/arch/PKGBUILD to avoid Rust src/ directory collisions..."
    mapfile -d '' ARGS < /proc/self/cmdline
    start_idx=0
    for ((i=0; i<${#ARGS[@]}; i++)); do
        if [[ "${ARGS[i]}" == *"/makepkg" || "${ARGS[i]}" == "makepkg" ]]; then
            start_idx=$((i + 1))
            break
        fi
    done
    actual_args=("${ARGS[@]:$start_idx}")
    cd packaging/arch
    exec makepkg "${actual_args[@]}"
fi

build() {
    cd "$pkgname-$pkgver"
    export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 "assets/frost-tune.desktop" "$pkgdir/usr/share/applications/frost-tune.desktop"
    install -Dm644 "assets/frost-tune.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/frost-tune.svg"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
