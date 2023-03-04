# We use the latest Rust stable release as base image
FROM rust:1.65.0-alpine AS builder
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apk update && apk add lld clang
# Copy all files from our working environment to our Docker image
COPY . .
# Let's build our binary!
#Let’s set the SQLX_OFFLINE environment variable to true in our Dockerfile to force sqlx to look at
#the saved metadata instead of trying to query a live database:
ENV SQLX_OFFLINE true
# We'll use the release profile to make it faaaast
RUN cargo build --release --bin zero2prod
# COPY --from=builder /app/target/zero2prod zero2prod
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
# Build a docker image tagged as "zero2prod" according to the recipe
# specified in `Dockerfile`
