FROM alpine:latest
RUN apk --no-cache add dumb-init

COPY spring-boot-layertools /usr/local/bin/spring-boot-layertools
RUN chmod +x /usr/local/bin/spring-boot-layertools

# Run as non-root
USER 1001
ENTRYPOINT ["dumb-init", "/usr/local/bin/spring-boot-layertools"]
