package io.github.adhil72.gitvault.spring;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import io.github.adhil72.gitvault.FakeCli;
import io.github.adhil72.gitvault.GitVaultException;
import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.springframework.boot.WebApplicationType;
import org.springframework.boot.builder.SpringApplicationBuilder;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.core.env.ConfigurableEnvironment;

class GitVaultEnvironmentPostProcessorTest {

    @TempDir
    Path dir;

    @BeforeEach
    void posixOnly() {
        assumeTrue(FakeCli.supported(), "the fake CLI is a shell script");
    }

    private ConfigurableEnvironment start(String... properties) {
        ConfigurableApplicationContext context = new SpringApplicationBuilder(Empty.class)
                .web(WebApplicationType.NONE)
                .properties(properties)
                .run();
        context.close();
        return context.getEnvironment();
    }

    @Test
    void loadsSecretsWithRelaxedNames() throws IOException {
        FakeCli cli = FakeCli.withVariables(dir, "{\"MY_API_KEY\":\"s3cret\"}");

        ConfigurableEnvironment env = start("gitvault.bin=" + cli.bin(), "gitvault.environment=production");

        assertEquals("s3cret", env.getProperty("MY_API_KEY"));
        assertEquals("s3cret", env.getProperty("my.api.key"));
        assertEquals(List.of("--json", "--non-interactive", "env", "export", "production"), cli.args());
    }

    @Test
    void usesTheSingleActiveProfileAsEnvironment() throws IOException {
        FakeCli cli = FakeCli.withVariables(dir, "{}");

        start("gitvault.bin=" + cli.bin(), "spring.profiles.active=staging", "gitvault.vault=v9");

        assertEquals(List.of("--json", "--non-interactive", "--vault", "v9", "env", "export", "staging"), cli.args());
    }

    @Test
    void osEnvironmentWinsUnlessOverrideIsSet() throws IOException {
        // PATH exists in every OS environment, so no process-level hacks are needed.
        FakeCli cli = FakeCli.withVariables(dir, "{\"PATH\":\"from-vault\"}");
        String[] base = {"gitvault.bin=" + cli.bin(), "gitvault.environment=production"};

        assertNotEquals("from-vault", start(base).getProperty("PATH"));
        assertEquals("from-vault", start(base[0], base[1], "gitvault.override=true").getProperty("PATH"));
    }

    @Test
    void doesNothingWhenDisabledOrWithoutEnvironment() throws IOException {
        FakeCli cli = FakeCli.withVariables(dir, "{\"X_ONLY\":\"1\"}");

        assertNull(start("gitvault.bin=" + cli.bin(), "gitvault.environment=p", "gitvault.enabled=false")
                .getProperty("X_ONLY"));
        assertNull(start("gitvault.bin=" + cli.bin()).getProperty("X_ONLY"));
    }

    @Test
    void failFastStopsTheStartupWithTheCliError() throws IOException {
        FakeCli cli = FakeCli.create(dir, 1, "{\"error\":{\"code\":\"E_LOCKED\",\"message\":\"locked\"}}", "");

        Throwable failure = assertThrows(
                Throwable.class, () -> start("gitvault.bin=" + cli.bin(), "gitvault.environment=production"));

        Throwable cause = failure;
        while (cause != null && !(cause instanceof GitVaultException)) {
            cause = cause.getCause();
        }
        assertTrue(cause instanceof GitVaultException);
        assertEquals("E_LOCKED", ((GitVaultException) cause).getCode());
    }

    @Test
    void canContinueWithoutSecretsWhenFailFastIsOff() throws IOException {
        FakeCli cli = FakeCli.create(dir, 1, "{\"error\":{\"code\":\"E_LOCKED\",\"message\":\"locked\"}}", "");

        ConfigurableEnvironment env = start(
                "gitvault.bin=" + cli.bin(), "gitvault.environment=production", "gitvault.fail-fast=false");

        assertFalse(env.getPropertySources().contains(GitVaultEnvironmentPostProcessor.PROPERTY_SOURCE_NAME));
    }

    static class Empty {}
}
