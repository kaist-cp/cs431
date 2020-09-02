#!/usr/bin/env bash

if ! [ -z "$(git status --porcelain)" ]; then
    echo "Error: repository is not clean."
    exit 1
fi

echo "[replace solution with skeleton]"
for skel in `find . -regex '.*/[^/]*.skeleton.rs'`; do
    echo "    replacing $skel"
    mv $skel ${skel//.skeleton.rs/.rs}
done

echo "[copy to public repo]"
rsync --exclude=".git" --delete --archive ./ ../cs492-concur/homework
