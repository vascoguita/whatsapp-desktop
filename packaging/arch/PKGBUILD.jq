.packages[0] |
"pkgname=\(.name)
pkgver=\(.version)
pkgrel=1
pkgdesc=\"\(.description)\"
arch=('x86_64')
url=\"\(.repository)\"
license=('\(.license)')
depends=('webkit2gtk-4.1' 'gtk3' 'libayatana-appindicator' 'gst-plugins-base' 'gst-plugins-good' 'gst-plugins-bad' 'gst-plugins-ugly' 'gst-libav')
install='\(.name).install'
package() { ar x ../../../deb/*_\(.version)_amd64.deb; tar -xf data.tar.* -C \"${pkgdir}\"; }"
