# spring-boot-layertools

Extract [Spring Boot layered JARs](https://spring.io/blog/2020/08/14/creating-efficient-docker-images-with-spring-boot-2-3)
up to ✨10x faster✨ than the built-in Java CLI.

[![docker hub](https://img.shields.io/docker/v/aramperes/spring-boot-layertools?color=%232496ed&label=docker%20hub&logo=docker&logoColor=fff&sort=semver)](https://hub.docker.com/r/aramperes/spring-boot-layertools)
[![crates.io](https://img.shields.io/crates/v/spring-boot-layertools.svg?logo=rust)](https://crates.io/crates/spring-boot-layertools)

## Usage

This tool is intended to be used as a
Docker [multi-stage builder](https://docs.docker.com/develop/develop-images/multistage-build/) image. For example:

```dockerfile
FROM aramperes/spring-boot-layertools:latest as layertools
# Copy your 'fat layered jar'
COPY ./target/*.jar layered.jar
# Extract layers
RUN spring-boot-layertools layered.jar extract

# Copy layers to your final image
FROM eclipse-temurin:17-jre-alpine
COPY --from=layertools /home/layertools/spring-boot-loader /
RUN true
COPY --from=layertools /home/layertools/dependencies /
RUN true
COPY --from=layertools /home/layertools/snapshot-dependencies /
RUN true
COPY --from=layertools /home/layertools/application /
RUN true

ENTRYPOINT ["java", "org.springframework.boot.loader.JarLauncher"]
```

## Command-line Options

```
USAGE:
    spring-boot-layertools <jar> <SUBCOMMAND>

ARGS:
    <jar>    The layered Spring Boot jar to extract

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    classpath    List classpath dependencies from the jar
    extract      Extracts layers from the jar for image creation
    help         Print this message or the help of the given subcommand(s)
    list         List layers from the jar that can be extracted
```

## License

MIT License. See `LICENSE` for details. Copyright &copy; 2022 Aram Peres.

"Spring" and "Spring Boot" are [trademarks](https://spring.io/trademarks) of Pivotal Software, Inc.
