# Maintainer: Turann_ <turanull000@gmail.com>
# Packager: Turann_ (turannul) <turanull000@gmail.com>

pkgname=fruitjuice
pkgver=0.0-r0-g0
pkgrel=1
pkgdesc="iDevice battery bridge"
arch=('x86_64')
url="https://github.com/turannul/fruitjuice"
license=('GPL-2.0-only')
depends=('dkms' 'netmuxd' 'libimobiledevice')
optdepends=('linux-headers: build the driver module against the Arch kernel')
makedepends=('cargo' 'git')
provides=("$pkgname-dkms" "$pkgname-git")
conflicts=("$pkgname-git")
source=("git+https://github.com/turannul/fruitjuice.git")
sha256sums=('SKIP')

pkgver() {
  local _daemon _module _revision _commit
  local _p="$srcdir/fruitjuice" _dp="$srcdir/fruitjuice/src/daemon/Cargo.toml" _mp="$srcdir/fruitjuice/src/module/dkms.conf"
  _daemon=$(awk -F'"' '/^\[package\]/{p=1} p && /^version *=/{print $2; exit}' "$_dp")
  _module=$(sed -n 's/^[[:space:]]*PACKAGE_VERSION[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$_mp")
  _revision=$(git -C "$_p" rev-list --count HEAD)
  _commit=$(git -C "$_p" rev-parse --short=7 HEAD)
  printf '%s_%s.r%s.g%s' "${_daemon:?}" "${_module:?}" "${_revision:?}" "${_commit:?}"
}

build() {
  cargo build --release --frozen --locked --manifest-path="$srcdir/fruitjuice/src/daemon/Cargo.toml" --target-dir="$srcdir/target"
}

package() {
  install -d -m0755 "$pkgdir/usr/src/fruitjuice-$pkgver"
  cp -a "$srcdir/fruitjuice/src/module/." "$pkgdir/usr/src/fruitjuice-$pkgver/"
  sed -i "s/^PACKAGE_VERSION=.*/PACKAGE_VERSION=\"$pkgver\"/" "$pkgdir/usr/src/fruitjuice-$pkgver/dkms.conf"  # This is prepare step?
  install -Dm755 "$srcdir/target/release/fruitjuiced" "$pkgdir/usr/bin/fruitjuiced"
  install -Dm644 "$srcdir/fruitjuice/src/daemon/fruitjuiced.service" "$pkgdir/usr/lib/systemd/user/fruitjuiced.service"
  install -D -m0644 "$srcdir/fruitjuice/modules-load.d/fruitjuice.conf" "$pkgdir/usr/lib/modules-load.d/fruitjuice.conf"
  install -D -m0644 "$srcdir/fruitjuice/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
