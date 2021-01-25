# Maintainer: Ethan Budd <budde25@protonmail.com>
pkgname=keysync
pkgver=3.0.2
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
pkgdesc="A utility to sync local authorized_keys file updated with your with Github, Gitlab, and Launchpad public keys"
url="https://github.com/budde25/ssh-key-sync"
license=('MIT OR Apache-2.0')

build() {
    return 0
}

package() {
    cd $srcdir
    cargo install --root="$pkgdir" --git=https://github.com/budde25/ssh-key-sync
}
