FROM alpine:3.21 AS builder

ARG http_proxy
ARG https_proxy
ARG HTTP_PROXY
ARG HTTPS_PROXY

RUN apk add --no-cache redsocks iptables

# Copy cloudflared binary from official image
COPY --from=cloudflare/cloudflared:latest /usr/local/bin/cloudflared /usr/local/bin/cloudflared

FROM alpine:3.21

RUN apk add --no-cache redsocks iptables

COPY --from=builder /usr/local/bin/cloudflared /usr/local/bin/cloudflared
COPY redsocks.conf /etc/redsocks.conf
COPY start.sh /start.sh
RUN chmod +x /start.sh

ENTRYPOINT ["/start.sh"]
