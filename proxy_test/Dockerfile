FROM rust:1.92-alpine AS build

WORKDIR /usr/src/tollkeeper
COPY . .
RUN cargo install --path app
RUN cp -r ./app/templates /usr/local/cargo/bin/

FROM nginxdemos/hello:latest
WORKDIR /usr/local/bin/
COPY --from=build /usr/src/tollkeeper/proxy_test/start_env.sh tollkeeper/start_env.sh
COPY --from=build /usr/local/cargo/bin/app app
COPY --from=build /usr/local/cargo/bin/templates tollkeeper/templates
RUN chmod u+x ./tollkeeper/start_env.sh
CMD ["./tollkeeper/start_env.sh"]
