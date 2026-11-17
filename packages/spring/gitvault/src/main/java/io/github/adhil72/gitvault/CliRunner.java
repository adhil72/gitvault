package io.github.adhil72.gitvault;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;

/** Runs the {@code gitvault} executable once and captures its output. */
final class CliRunner {

    record Output(int exitCode, String stdout, String stderr) {}

    private CliRunner() {}

    static Output run(List<String> command, Duration timeout) {
        Process process = start(command);
        // Never wait for input: a locked vault must fail fast instead of hanging a startup.
        closeQuietly(process.getOutputStream());
        CompletableFuture<String> stdout = drain(process.getInputStream());
        CompletableFuture<String> stderr = drain(process.getErrorStream());
        try {
            if (!process.waitFor(timeout.toMillis(), TimeUnit.MILLISECONDS)) {
                process.destroyForcibly();
                throw new GitVaultException("E_INTERNAL", "gitvault did not answer within " + timeout.toSeconds() + "s");
            }
            return new Output(process.exitValue(), stdout.join(), stderr.join());
        } catch (InterruptedException e) {
            process.destroyForcibly();
            Thread.currentThread().interrupt();
            throw new GitVaultException("E_INTERNAL", "interrupted while waiting for gitvault");
        }
    }

    private static Process start(List<String> command) {
        try {
            return new ProcessBuilder(command).start();
        } catch (IOException e) {
            throw new GitVaultException(
                    "E_NOT_INSTALLED",
                    "the gitvault CLI could not be started",
                    "install GitVault, or set GITVAULT_BIN to its path");
        }
    }

    // Both streams are read at the same time, otherwise a full pipe buffer blocks the child.
    private static CompletableFuture<String> drain(InputStream stream) {
        return CompletableFuture.supplyAsync(() -> {
            try (InputStream in = stream) {
                ByteArrayOutputStream buffer = new ByteArrayOutputStream();
                in.transferTo(buffer);
                return buffer.toString(StandardCharsets.UTF_8);
            } catch (IOException e) {
                return "";
            }
        });
    }

    private static void closeQuietly(java.io.OutputStream stream) {
        try {
            stream.close();
        } catch (IOException ignored) {
            // The child already exited or never read stdin; nothing to do.
        }
    }
}
