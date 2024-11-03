FROM rust:1 AS build
WORKDIR /app
COPY . .
ENV XESS_PATH=/app/static/css
RUN cargo install --path .

FROM debian:bookworm
WORKDIR /app
ENV XESS_PATH=/app/static/css
COPY --from=build /app/target/release/pronouns /app/bin/pronouns
COPY --from=build /app/static/css /app/static/css
COPY --from=build /app/dhall /app/dhall
RUN apt-get update \
  && apt-get -y install libssl3
CMD ["/app/bin/pronouns"]