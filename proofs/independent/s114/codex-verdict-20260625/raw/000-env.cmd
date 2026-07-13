date -u; uname -a; git status --short --branch; git rev-parse HEAD; git rev-parse origin/main; rustc --version; cargo --version; ./target/debug/garnet --help | sed -n "1,80p"
