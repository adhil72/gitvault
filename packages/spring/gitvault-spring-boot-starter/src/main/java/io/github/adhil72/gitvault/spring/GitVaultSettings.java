package io.github.adhil72.gitvault.spring;

import org.springframework.core.env.ConfigurableEnvironment;

/**
 * The {@code gitvault.*} settings. They are read straight from the {@link ConfigurableEnvironment}
 * because this runs before {@code @ConfigurationProperties} binding exists.
 */
record GitVaultSettings(
        boolean enabled,
        String environment,
        String vault,
        String bin,
        boolean override,
        boolean failFast,
        int timeoutSeconds) {

    static GitVaultSettings from(ConfigurableEnvironment env) {
        return new GitVaultSettings(
                env.getProperty("gitvault.enabled", Boolean.class, true),
                resolveEnvironment(env),
                blankToNull(env.getProperty("gitvault.vault")),
                blankToNull(env.getProperty("gitvault.bin")),
                env.getProperty("gitvault.override", Boolean.class, false),
                env.getProperty("gitvault.fail-fast", Boolean.class, true),
                env.getProperty("gitvault.timeout-seconds", Integer.class, 60));
    }

    // gitvault.environment, then GITVAULT_ENV (same as the Node library), then a single active profile.
    private static String resolveEnvironment(ConfigurableEnvironment env) {
        String explicit = blankToNull(env.getProperty("gitvault.environment"));
        if (explicit != null) {
            return explicit;
        }
        String fromEnv = blankToNull(env.getProperty("GITVAULT_ENV"));
        if (fromEnv != null) {
            return fromEnv;
        }
        String[] profiles = env.getActiveProfiles();
        return profiles.length == 1 ? profiles[0] : null;
    }

    private static String blankToNull(String value) {
        return value == null || value.isBlank() ? null : value.trim();
    }
}
