# Maintainer: Ethan Budd <budde25@protonmail.com>
_pkgname=keysync
_version=[version]
pkgname=$_pkgname-bin
pkgver=$_version
pkgrel=1
arch=('x86_64')
pkgdesc="A utility to sync local authorized_keys file updated with your with Github, Gitlab or Launchpad public keys"
url="https://github.com/budde25/ssh-key-sync/"
source=("https://github.com/budde25/ssh-key-sync/releases/download/v${_version}/${_pkgname}_${_version}_amd64.deb")
sha256sums=('[sha256]')
license=('GPL-3.0-or-later')
provides=('$_pkgname')

package() {
    tar -xf data.tar.xz -C ${pkgdir}
    cd ${pkgdir}
    mv lib usr/lib
}
