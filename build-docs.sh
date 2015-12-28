#!/bin/bash -x

# Kill the existing target directory and built docs
cargo clean
cargo doc --no-deps
rev=`git rev-parse --short HEAD`

# Create a new target directory with an empty git repo
prev=`pwd`
mkdir target/uploads
cd target/uploads
git init

# Add generated docs and a redirect page to our new repo
cp -R ../doc/* .
cat <<EOF > index.html
<!doctype html>
<html>
<head>
<title></title>
<meta http-equiv="refresh" content="0; url=./cadence/" />
</head>
</html>
EOF

# Add the real github repo as a remote and force-push everything
# we just committed to our new repo locally to the gh-pages branch
git add --all .
git commit -m "Building Cadence docs at ${rev}"
git checkout -b gh-pages
git remote add github git@github.com:tshlabs/cadence.git
git push --force github gh-pages

# Kill the target directory and hence the local repo
cd $prev
cargo clean
