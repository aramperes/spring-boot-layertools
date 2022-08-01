FROM alpine:latest
RUN adduser --disabled-password layertools
RUN apk --no-cache add dumb-init
WORKDIR /home/layertools

COPY spring-boot-layertools /usr/local/bin/spring-boot-layertools
RUN chmod +x /usr/local/bin/spring-boot-layertools

# Run as non-root
USER layertools
ENTRYPOINT ["dumb-init", "/usr/local/bin/spring-boot-layertools"]
