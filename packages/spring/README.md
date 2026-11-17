# GitVault for the JVM (Spring Boot, Java, Kotlin)

Load your GitVault secrets into a Spring Boot app at startup. No `.env` files, no secrets in
`application.yml`. It is the JVM counterpart of the [Node package](../node/README.md).

| Artifact                                         | Use it for                                                       |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| `io.github.adhil72:gitvault-spring-boot-starter` | Spring Boot 3.x apps (adds the secrets to the `Environment`)     |
| `io.github.adhil72:gitvault`                     | Anything else on the JVM: plain Java, Kotlin, Quarkus, Micronaut |

Both need Java 17+ and the `gitvault` CLI on `PATH` (or `GITVAULT_BIN`). The core has no dependencies.

## Add the dependency

Not on Maven Central yet. Until it is, build and install it locally (see [Development](#development)) and add `mavenLocal()`.

**Gradle (Kotlin DSL, `build.gradle.kts`)**

```kotlin
repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    implementation("io.github.adhil72:gitvault-spring-boot-starter:1.0.0")
}
```

**Gradle (Groovy, `build.gradle`)**

```groovy
repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    implementation 'io.github.adhil72:gitvault-spring-boot-starter:1.0.0'
}
```

**Maven (`pom.xml`)**

```xml
<dependency>
  <groupId>io.github.adhil72</groupId>
  <artifactId>gitvault-spring-boot-starter</artifactId>
  <version>1.0.0</version>
</dependency>
```

## Use it in Spring Boot

Nothing to call. Pick the vault environment and read the secrets like any property:

```yaml
# application.yml
gitvault:
  environment: production # or set GITVAULT_ENV, or use exactly one active profile

spring:
  datasource:
    password: ${DATABASE_PASSWORD} # a secret named DATABASE_PASSWORD in the vault
```

```kotlin
@Value("\${stripe.api.key}") lateinit var stripeKey: String // secret STRIPE_API_KEY
```

- Secrets are exposed like OS environment variables, so `STRIPE_API_KEY` is also found as `stripe.api.key` and binds to `@ConfigurationProperties`.
- **Environment**: `gitvault.environment`, then `GITVAULT_ENV`, then the active profile when there is exactly one. With none of them, nothing is loaded and a warning is logged. Secrets of that environment plus shared ones (no environment) load; the environment's own value wins.
- **Precedence**: OS environment variables, system properties and command-line arguments win over the vault (like dotenv). Set `gitvault.override=true` to put the vault ahead of the OS environment.
- **Failing**: a locked vault, missing CLI or denied access stops the startup with a `GitVaultException` (`getCode()` is `E_LOCKED`, `E_NOT_INSTALLED`, ...). Set `gitvault.fail-fast=false` to log a warning and continue.

| Property                   | Default                                        | Meaning                                  |
| -------------------------- | ---------------------------------------------- | ---------------------------------------- |
| `gitvault.enabled`         | `true`                                         | Turn loading off (e.g. in tests)         |
| `gitvault.environment`     | `GITVAULT_ENV`, then the single active profile | Vault environment to load                |
| `gitvault.vault`           | `GITVAULT_VAULT`, then `gitvault vault use`    | Vault id                                 |
| `gitvault.bin`             | `GITVAULT_BIN`, then `gitvault` on `PATH`      | Path of the CLI                          |
| `gitvault.override`        | `false`                                        | Vault wins over OS environment variables |
| `gitvault.fail-fast`       | `true`                                         | Fail the startup on errors               |
| `gitvault.timeout-seconds` | `60`                                           | How long to wait for the CLI             |

It logs only how many variables were loaded, never names or values. Actuator's `/env` masks values by
default in Spring Boot 3; keep `management.endpoint.env.show-values` at `never` (or `when-authorized`).

### Running locally, in CI and on servers

- **Developers**: sign in once with `gh auth login`, then trust the device once (`gitvault device trust --name laptop`). After that `./gradlew bootRun` and `mvn spring-boot:run` unlock silently. To limit it to one project, run `gitvault trust add . --env development` inside it.
- **Servers/CI**: `gitvault device trust --name prod-server --export` prints a key once. Set it as `GITVAULT_DEVICE_KEY` together with `GITVAULT_TOKEN` (read-only GitHub token for the vault repo). `GITVAULT_PASSWORD` also works. Docker images must contain the `gitvault` binary.
- **Tests**: set `gitvault.enabled=false` (for example in `src/test/resources/application.properties`) so tests never call the CLI.

## Without Spring (Java, Kotlin)

```kotlin
val secrets: Map<String, String> = GitVault.builder().vault("my-vault").build().exportEnvironment("production")
```

```java
Map<String, String> secrets = GitVault.create().exportEnvironment("production");
```

Errors are `GitVaultException` with a stable `getCode()` and a `getHint()`. The JVM cannot remove
variables from its own process environment, so keep `GITVAULT_PASSWORD` / `GITVAULT_DEVICE_KEY` out of
processes that also run untrusted code.

## Development

```bash
./gradlew build                 # compile, test, sources + javadoc jars
./gradlew publishToMavenLocal   # install into ~/.m2 so other projects can use it
pnpm test:spring                # from the repo root: builds the CLI first, then runs the tests
```

The starter is compiled against Spring Boot 3.5 and uses only APIs present in every 3.x release.
Spring Boot 4 moved `EnvironmentPostProcessor` to another package, so it needs its own build (not done yet).
Tests use a fake `gitvault` shell script, so they are skipped on Windows.

## License

MIT. See [LICENSE](./LICENSE).
