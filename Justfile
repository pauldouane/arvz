# List available recipes
@list:
    just --list
    
# Launch linters / fmt / advisors for project
@lint:
    cargo check && \
        cargo clippy && \
        cargo fmt