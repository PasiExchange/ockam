rm -rf /tmp/registry
mkdir /tmp/registry
cp -a ../index/. /tmp/registry/index

pushd /tmp/registry
# We clone crates.io index so that we are sure we can not
# re-upload crates that are already released.
# git clone https://github.com/rust-lang/crates.io-index.git index
rm -rf index/config.json
popd

echo "Starting local registry"
estuary --base-url http://localhost:7878 --crate-dir /tmp/registry --index-dir /tmp/registry/index
