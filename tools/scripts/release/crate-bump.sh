#!/usr/bin/env bash

# Check if file has been updated since last tag.
last_git_tag=$(eval "git describe --tags --abbrev=0");
updated_crates="";

for path in $(ls "implementations/rust/ockam"); do 
    if git diff $last_git_tag --quiet --name-status -- implementations/rust/ockam/$path/src; then
        git diff $last_git_tag --quiet --name-status -- implementations/rust/ockam/$path/Cargo.toml || updated_crates="$updated_crates $path"
    else
        updated_crates="$updated_crates $path"
    fi
done


# Update changelogs
for crate in ${updated_crates[@]}; do
    git cliff --unreleased --commit-path implementations/rust/ockam/$crate --prepend implementations/rust/ockam/$crate/CHANGELOG.md
done

# Commit all files
git add --all
if git commit -m "ci: update crates changelog"; then
    echo "changelog was generated successfully"
else
    echo "no changelog was generated"
fi


crate_array=($MODIFIED_RELEASE)

declare -A specified_crate_version

for word in ${crate_array[@]}; do
    key="${word%%:*}"
    value="${word##*:}"
    specified_crate_version[$key]=$value
done

for to_update in ${updated_crates[@]}; do
    # If the bump version is indicated as release, we don't bump
    # or publish the crate.
    version="$RELEASE_VERSION"
    if [[ ! -z "${specified_crate_version[$to_update]}" ]]; then
        echo "bumping $to_update as ${specified_crate_version[$to_update]}"
        version="${specified_crate_version[$to_update]}"
    fi

    echo y | cargo release $version --no-push --no-publish --no-tag -p $to_update --execute
done