# Maintainer: Turann_ <turanull000@gmail.com>
# Packager: Turann_ <turanull000@gmail.com>

pkgname=fruitjuice
pkgver=1
pkgrel=1
pkgdesc="FruitJuice: iDevice Battery Bridge (DKMS driver and daemon)"
arch=('x86_64')
url="https://github.com/turannul/fruitjuice"
license=('GPL-2.0-only')
depends=('dkms' 'glibc' 'netmuxd' 'libimobiledevice')
optdepends=('linux-headers')
makedepends=('cargo' 'git')
provides=("$pkgname-dkms" "$pkgname-git")
conflicts=("$pkgname-git")

pkgver() {
    local src="$startdir/src/driver"
    local ver
    ver=$(sed -n 's/^[[:space:]]*PACKAGE_VERSION[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$src/dkms.conf")
    printf "%s.r%s.%s" "${ver:-1.0}" "$(git -C "$startdir" rev-list --count HEAD -- .)" "$(git -C "$startdir" log -1 --format="%h" -- .)"
}

build() {
    cd "$startdir/src/daemon"
    cargo build --release --locked
}

package() {
    local src="$startdir/src/driver"
    local dest="$pkgdir/usr/src/$pkgname-$pkgver"

    install -Dm755 "$startdir/src/daemon/target/release/fruitjuiced" "$pkgdir/usr/bin/fruitjuiced"
    install -Dm644 "$startdir/fruitjuiced.service" "$pkgdir/usr/lib/systemd/user/fruitjuiced.service"

    install -d -m0755 "$pkgdir/usr/lib/modules-load.d"
    echo "$pkgname" > "$pkgdir/usr/lib/modules-load.d/$pkgname.conf"

    install -d -m0755 "$dest"
    cp -a "$src/." "$dest/"
    find "$dest" -type f \( -name '*.o' -o -name '*.ko' -o -name '*.ko.zst' -o -name '*.mod' -o -name '*.mod.c' -o -name '*.mod.o' -o -name '.*.cmd' -o -name 'Module.symvers' -o -name 'modules.order' \) -delete
    sed -i "s/^PACKAGE_VERSION=.*/PACKAGE_VERSION=\"$pkgver\"/" "$dest/dkms.conf"
}
