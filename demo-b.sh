#!/usr/bin/env bash -e

# We need to download all 3rd party dependencies from crates.io
# using cargo local registry
cargo update
if git add --all; then
    git commit -m "ci: update cargo.lock"
fi

cargo local-registry --sync Cargo.lock /tmp/registry --no-delete

source tools/scripts/release/publish-order.sh

declare -A crates_to_publish

# We try to bump crate with a specified tracking git tag.
last_git_tag=$GIT_TAG

# Check if git tag is valid.
if git show-ref --tags $last_git_tag --quiet; then
    echo "Specified $last_git_tag as tag to track updated crates.";
else
    echo "Specified git tag used to track updated crates invalid."
    exit 1
fi

to_download_crates=""

# Bump all crates and update cargo.lock
./tools/scripts/release/crate-bump.sh

for crate in $(ls "implementations/rust/ockam"); do
    echo "Checking $crate if publishable"
    is_publish=$(eval "tomlq package.publish -f implementations/rust/ockam/$crate/Cargo.toml")

    if [[ $is_publish == false ]]; then
        echo "$crate was indicated as non-publish"
        continue
    fi

    if git diff $last_git_tag --quiet --name-status -- implementations/rust/ockam/$crate/src; then
        git diff $last_git_tag --quiet --name-status -- implementations/rust/ockam/$crate/Cargo.toml || crates_to_publish[$crate]=true
    else
        crates_to_publish[$crate]=true
    fi

    if [[ -z ${crates_to_publish[$crate]} ]]; then
        echo "Crate $crate has not been bumped so is not published"
        # Crate has not been updated, download from crates.io
        to_download_crates="$to_download_crates\\n$crate = '*'"
    fi
done

# Not all Ockam crates will be uploaded to local registry
# so we need to still download missing crates from crates.io
# and store to our registry.
if [[ ! -z $to_download_crates ]]; then
    rm -rf /tmp/to_download_crates
    mkdir /tmp/to_download_crates
    pushd /tmp/to_download_crates

    cargo init
    echo -e "$to_download_crates" >> Cargo.toml
    cat Cargo.toml

    cargo update

    # Download crates to /tmp/registry
    cargo local-registry --sync Cargo.lock /tmp/registry --no-delete
    popd
fi

# Publish crates that have been bumped.
for package in ${sorted_packages[@]}; do
    if [[ -z ${crates_to_publish[$package]} ]]; then
        echo "skipping $package crate"
        continue
    fi

    echo "publishing $package"

    version=$(eval "tomlq package.version -f implementations/rust/ockam/$package/Cargo.toml")
    name=$(eval "tomlq package.name -f implementations/rust/ockam/$package/Cargo.toml")

    cargo publish -p $name --token null --no-verify

    # We still need to copy .crate files to /tmp/registry
    # as estuary receives the .crate file in a folder.
    cp ./target/package/$name-$version.crate /tmp/registry
done
