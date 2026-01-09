FROM rust:1.92-alpine AS build

WORKDIR /usr/src/tollkeeper
COPY . .
RUN cargo install --path app
RUN cp -r ./app/templates /usr/local/cargo/bin/

FROM nginxdemos/hello:latest
WORKDIR /usr/local/bin/
COPY --from=build /usr/src/tollkeeper/proxy_test/start_env.sh app/start_env.sh
COPY --from=build /usr/local/cargo/bin/app tollkeeper
RUN chmod u+x ./app/start_env.sh
CMD ["./app/start_env.sh"]
