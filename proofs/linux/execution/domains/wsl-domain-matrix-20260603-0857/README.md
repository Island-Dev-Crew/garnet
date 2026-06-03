# WSL Domain Matrix Scope

This bundle records execution/portability proof for the Garnet Studio domain
matrix under WSL2 Ubuntu. It is not Linux seccomp proof, OS-sandbox proof,
Wasmtime fuel proof, or Linux desktop/Tauri GUI launch proof.

The matrix passed 20/20 current examples and 60/60 parse/check/run commands,
including the expected signed-hot-reload BLAKE3 mismatch rejection.
