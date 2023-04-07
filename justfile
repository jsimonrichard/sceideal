build-server:
    cd ./server; cargo build
    typeshare
    
typeshare:
    typeshare ./server --lang=typescript --output-file=./frontend/shared-types.ts

run-dev:
    mprocs "pushd ./server; cargo run" "pushd ./frontend; npm run dev"

check:
    cd ./server; cargo clippy