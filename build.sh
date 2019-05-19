set -xe
tag=v1.0.1

mk() {
            target=$1
            src=$2
            for binary in $(find tmp/$src -type f); do
              chmod +x $binary
              name=$(basename $binary .exe)-$tag-$target
#              find .
              mkdir -p tmp/$name
              cp README.md \
                LICENSE-MIT \
                LICENSE-APACHE \
                $binary  \
                                tmp/$name
              tar czvf gh-release/$name.tar.gz -C tmp $name
            done
          }
          mk x86_64-unknown-linux-gnu linux
          mk x86_64-apple-darwin darwin
          mk x86_64-pc-windows-msvc windows

