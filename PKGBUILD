# Maintainer: Ethan Budd <budde25@protonmail.com>
_pkgname=keysync
_version=0.2.0
pkgname=$_pkgname-bin
pkgver=$_version
pkgrel=1
arch=('x86_64')
pkgdesc="A utility to sync local authorized_keys file updated with your with Github, Gitlab or Launchpad public keys"
url="https://github.com/budde25/ssh-key-sync/"
source=("https://github.com/budde25/ssh-key-sync/releases/download/v${_version}/${_pkgname}_${_version}_amd64.deb")
sha256sums=('a4cf299cf14684573e61d2e3dcd2a0dbf7d9ed4d655138868cf75c580126feab')
license=('GPL-3.0-or-later')
provides=('$_pkgname')

package() {
    tar -xf data.tar.xz -C ${pkgdir}
    cd ${pkgdir}
    mv lib usr/lib
}