# Maintainer: Turann_ <turanull000@gmail.com>
# Packager: Turann_ <turanull000@gmail.com>

pkgname=fruitjuice
pkgver=2.0.0_1.0.r18.g92aac61
pkgrel=1
pkgdesc="FruitJuice: iDevice Battery Bridge (DKMS driver and daemon)"
arch=('any')
url="https://github.com/turannul/fruitjuice.git"
license=('GPL-2.0-only')
depends=('dkms' 'glibc' 'netmuxd' 'libimobiledevice')
optdepends=('linux-headers')
makedepends=('cargo' 'git')
provides=("$pkgname-dkms" "$pkgname-git")
conflicts=("$pkgname-git")
replaces=("$pkgname-dkms")
source=("git+https://github.com/turannul/fruitjuice.git")
sha256sums=('SKIP')

pkgver() {
  local _daemon_version _driver_version _revision _commit _driver_p="$srcdir/driver/" _daemon_p="$srcdir/daemon/"
  _driver_version=$(sed -n 's/^[[:space:]]*PACKAGE_VERSION[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$_driver_p/dkms.conf")
  _daemon_version=$(awk -F'"' '/^\[package\]/{p=1} p && /^version *=/{print $2; exit}' "$_daemon_p/Cargo.toml")
  _revision=$(git -C "$_driver_p" rev-list --count HEAD)
  _commit=$(git -C "$_driver_p" rev-parse --short=7 HEAD)
  printf '%s_%s.r%s.g%s' "${_daemon_version:?}" "${_driver_version:?}" "${_revision:?}" "${_commit:?}"
}

build() {
    cargo build --release --locked --manifest-path "$startdir/src/daemon/Cargo.toml"
}

package() {
    install -Dm755 "$startdir/src/daemon/target/release/fruitjuiced" "$pkgdir/usr/bin/fruitjuiced"
    install -Dm644 "$startdir/fruitjuiced.service" "$pkgdir/usr/lib/systemd/user/fruitjuiced.service"
    install -d -m0755 "$pkgdir/usr/lib/modules-load.d"
    echo "$pkgname" > "$pkgdir/usr/lib/modules-load.d/$pkgname.conf"
    install -d -m0755 "$pkgdir/usr/src/$pkgname-$pkgver"
    cp -a "$startdir/src/driver/." "$pkgdir/usr/src/$pkgname-$pkgver/"
    sed -i "s/^PACKAGE_VERSION=.*/PACKAGE_VERSION=\"$pkgver\"/" "$pkgdir/usr/src/$pkgname-$pkgver/dkms.conf"
}
