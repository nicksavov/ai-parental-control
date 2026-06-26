# Build a static binary, then ship it on a minimal base.
FROM golang:1.26-alpine AS build
WORKDIR /src
COPY go.mod ./
# No third-party modules yet; this caches once they are added.
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /out/api ./...

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=build /out/api /api
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/api"]
