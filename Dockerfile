FROM rust:1.92-alpine AS build

WORKDIR /usr/src/tollkeeper
COPY . .
RUN cargo install --path app

FROM alpine:3.23
WORKDIR /usr/local/bin/
COPY --from=build /usr/src/tollkeeper/app/config.toml app/config.toml
COPY --from=build /usr/src/tollkeeper/app/templates app/templates
COPY --from=build /usr/local/cargo/bin/app tollkeeper
CMD ["tollkeeper"]
