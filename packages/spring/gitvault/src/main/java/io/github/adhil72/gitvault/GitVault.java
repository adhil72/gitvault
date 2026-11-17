package io.github.adhil72.gitvault;

import java.time.Duration;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

/**
 * Reads one environment's secrets through the installed {@code gitvault} CLI, which signs in
 * through the GitHub CLI (or {@code GITVAULT_TOKEN} on servers), checks vault access and decrypts.
 * Values are held only in the returned map: they are never logged and never written to disk.
 *
 * <pre>{@code
 * Map<String, String> secrets = GitVault.create().exportEnvironment("production");
 * }</pre>
 */
public final class GitVault {

    private static final String DEFAULT_BIN = "gitvault";
    private static final Duration DEFAULT_TIMEOUT = Duration.ofSeconds(60);

    private final String bin;
    private final String vault;
    private final Duration timeout;

    private GitVault(Builder builder) {
        this.bin = firstNonBlank(builder.bin, System.getenv("GITVAULT_BIN"), DEFAULT_BIN);
        this.vault = firstNonBlank(builder.vault, System.getenv("GITVAULT_VAULT"), null);
        this.timeout = builder.timeout;
    }

    /** A client using {@code GITVAULT_BIN} / {@code GITVAULT_VAULT} or the CLI defaults. */
    public static GitVault create() {
        return builder().build();
    }

    public static Builder builder() {
        return new Builder();
    }

    /**
     * Secrets of the environment plus shared ones (no environment); the environment's own value
     * wins. Keys are the secret names, e.g. {@code DATABASE_URL}.
     *
     * @throws GitVaultException when the vault is locked, access is denied, the CLI is missing...
     */
    public Map<String, String> exportEnvironment(String environment) {
        if (environment == null || environment.isBlank()) {
            throw new GitVaultException("E_USAGE", "no environment given", "pass one, e.g. \"production\"");
        }
        CliRunner.Output output = CliRunner.run(command(environment.trim()), timeout);
        Map<String, Object> body = parseBody(output);
        if (output.exitCode() != 0 || body.get("error") != null) {
            throw failure(body.get("error"));
        }
        return variables(body.get("variables"));
    }

    private List<String> command(String environment) {
        List<String> command = new ArrayList<>(List.of(bin, "--json", "--non-interactive"));
        if (vault != null) {
            command.addAll(List.of("--vault", vault));
        }
        command.addAll(List.of("env", "export", environment));
        return command;
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> parseBody(CliRunner.Output output) {
        try {
            Object parsed = MiniJson.parse(output.stdout());
            if (parsed instanceof Map<?, ?> map) {
                return (Map<String, Object>) map;
            }
        } catch (IllegalArgumentException ignored) {
            // Falls through to the error below; the parser's message never holds output text.
        }
        String detail = output.stderr().isBlank() ? "unreadable output from gitvault" : output.stderr().trim();
        throw new GitVaultException("E_INTERNAL", detail);
    }

    private static GitVaultException failure(Object error) {
        if (error instanceof Map<?, ?> map) {
            return new GitVaultException(
                    text(map.get("code"), "E_INTERNAL"), text(map.get("message"), "gitvault failed"), text(map.get("hint"), null));
        }
        return new GitVaultException("E_INTERNAL", "gitvault failed");
    }

    private static Map<String, String> variables(Object raw) {
        Map<String, String> result = new LinkedHashMap<>();
        if (raw instanceof Map<?, ?> map) {
            map.forEach((key, value) -> result.put(String.valueOf(key), String.valueOf(value)));
        }
        return result;
    }

    private static String text(Object value, String fallback) {
        return value instanceof String s && !s.isBlank() ? s : fallback;
    }

    private static String firstNonBlank(String... candidates) {
        for (String candidate : candidates) {
            if (candidate != null && !candidate.isBlank()) {
                return candidate;
            }
        }
        return null;
    }

    public static final class Builder {
        private String bin;
        private String vault;
        private Duration timeout = DEFAULT_TIMEOUT;

        private Builder() {}

        /** Path of the gitvault binary. Defaults to {@code GITVAULT_BIN}, then {@code gitvault} on PATH. */
        public Builder bin(String bin) {
            this.bin = bin;
            return this;
        }

        /** Vault id. Defaults to {@code GITVAULT_VAULT}, then the vault chosen with {@code gitvault vault use}. */
        public Builder vault(String vault) {
            this.vault = vault;
            return this;
        }

        /** How long to wait for the CLI (it may sync with GitHub). Default 60 seconds. */
        public Builder timeout(Duration timeout) {
            this.timeout = Objects.requireNonNull(timeout, "timeout");
            return this;
        }

        public GitVault build() {
            return new GitVault(this);
        }
    }
}
