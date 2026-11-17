package io.github.adhil72.gitvault.spring;

import io.github.adhil72.gitvault.GitVault;
import io.github.adhil72.gitvault.GitVaultException;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Map;
import org.apache.commons.logging.Log;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.context.config.ConfigDataEnvironmentPostProcessor;
import org.springframework.boot.env.EnvironmentPostProcessor;
import org.springframework.boot.logging.DeferredLogFactory;
import org.springframework.core.Ordered;
import org.springframework.core.env.ConfigurableEnvironment;
import org.springframework.core.env.MapPropertySource;
import org.springframework.core.env.MutablePropertySources;
import org.springframework.core.env.StandardEnvironment;
import org.springframework.core.env.SystemEnvironmentPropertySource;

/**
 * Adds a {@code gitvault} property source with the secrets of one vault environment, before the
 * application context is created, so {@code ${DATABASE_PASSWORD}} placeholders, {@code @Value}
 * and {@code @ConfigurationProperties} see them like real environment variables.
 *
 * <p>The source behaves like OS environment variables: {@code DATABASE_PASSWORD} is also found as
 * {@code database.password}. It sits behind the OS environment, system properties and command-line
 * arguments (like dotenv, existing values win) unless {@code gitvault.override=true}.
 *
 * <p>It runs just after {@code application.yml} is read, so {@code gitvault.*} may be set there.
 * Only the number of variables is logged, never names or values.
 */
public class GitVaultEnvironmentPostProcessor implements EnvironmentPostProcessor, Ordered {

    /** Name of the property source this class adds. */
    public static final String PROPERTY_SOURCE_NAME = "gitvault";

    private final Log log;

    public GitVaultEnvironmentPostProcessor(DeferredLogFactory logFactory) {
        this.log = logFactory.getLog(GitVaultEnvironmentPostProcessor.class);
    }

    @Override
    public int getOrder() {
        return ConfigDataEnvironmentPostProcessor.ORDER + 1;
    }

    @Override
    public void postProcessEnvironment(ConfigurableEnvironment environment, SpringApplication application) {
        GitVaultSettings settings = GitVaultSettings.from(environment);
        if (!settings.enabled()) {
            return;
        }
        if (settings.environment() == null) {
            log.warn("GitVault is on the classpath but no environment is set: secrets were not loaded. "
                    + "Set gitvault.environment, GITVAULT_ENV, or exactly one active Spring profile.");
            return;
        }
        try {
            load(environment, settings);
        } catch (GitVaultException e) {
            if (settings.failFast()) {
                throw e;
            }
            log.warn("GitVault secrets were not loaded: " + e.getMessage());
        }
    }

    private void load(ConfigurableEnvironment environment, GitVaultSettings settings) {
        Map<String, String> secrets = GitVault.builder()
                .bin(settings.bin())
                .vault(settings.vault())
                .timeout(Duration.ofSeconds(settings.timeoutSeconds()))
                .build()
                .exportEnvironment(settings.environment());
        // System-environment semantics give relaxed names (DB_URL is found as db.url) for free.
        MapPropertySource source = new SystemEnvironmentPropertySource(PROPERTY_SOURCE_NAME, new LinkedHashMap<>(secrets));
        add(environment.getPropertySources(), source, settings.override());
        log.info("GitVault loaded " + secrets.size() + " variable(s) for environment '" + settings.environment() + "'");
    }

    private static void add(MutablePropertySources sources, MapPropertySource source, boolean override) {
        String anchor = StandardEnvironment.SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME;
        if (!sources.contains(anchor)) {
            sources.addLast(source);
        } else if (override) {
            sources.addBefore(anchor, source);
        } else {
            sources.addAfter(anchor, source);
        }
    }
}
