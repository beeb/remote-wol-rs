FROM debian:bullseye-slim AS builder

ARG TARGETOS
ARG TARGETARCH

WORKDIR /remote_wol

RUN apt-get update && apt install -y libcap2-bin

# Copy binary and adjust permissions
COPY ./${TARGETOS}_${TARGETARCH}/remote_wol .
RUN chmod +x remote_wol && setcap 'cap_net_raw+epi' remote_wol

FROM gcr.io/distroless/cc:nonroot

WORKDIR /remote_wol

# Copy the binary with correct permissions
COPY --from=builder /remote_wol/remote_wol .

USER nonroot:nonroot

# We use entrypoint to allow passing arguments to the binary using `CMD`
ENTRYPOINT ["/remote_wol/remote_wol"]
