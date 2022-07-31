# spring-boot-layertools

Extract [Spring Boot layered JARs](https://spring.io/blog/2020/08/14/creating-efficient-docker-images-with-spring-boot-2-3)
up to 10x faster than the built-in Java CLI.

## Usage

This tool uses `mmap` to load the layered JAR in memory. This is to improve the performance of random-access reads
when extracting the layers. As such, this tool is only compatible with Linux.

```shell
$ ./spring-boot-layertools layered.jar extract --destination ./extracted
# Java equivalent: java -Djarmode=layertools -jar layered.jar extract --destination ./extracted

$ find ./extracted

./extracted
./extracted/spring-boot-loader
./extracted/spring-boot-loader/org
...

./extracted/dependencies
./extracted/dependencies/BOOT-INF
./extracted/dependencies/BOOT-INF/lib
...
```

## License

MIT License. See `LICENSE` for details. Copyright &copy; 2022 Aram Peres.

"Spring" and "Spring Boot" are [trademarks](https://spring.io/trademarks) of Pivotal Software, Inc.
