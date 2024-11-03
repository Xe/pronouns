FROM rust:1 AS build
WORKDIR /app
COPY . .
ENV XESS_PATH=/app/static/css
RUN cargo install --path .

FROM debian:bullseye
WORKDIR /app
ENV XESS_PATH=/app/static/css
COPY --from=build /app/target/release/pronouns /app/bin/pronouns
COPY --from=build /app/static/css /app/static/css
RUN apt-get update \
  && apt-get -y install openssl
CMD ["/app/bin/pronouns"]