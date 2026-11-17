package io.github.adhil72.gitvault;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/** A stand-in {@code gitvault} executable (POSIX shell) that prints canned output and records its arguments. */
public final class FakeCli {

    private final Path dir;

    private FakeCli(Path dir) {
        this.dir = dir;
    }

    public static boolean supported() {
        return !System.getProperty("os.name", "").toLowerCase().contains("win");
    }

    public static FakeCli create(Path dir, int exitCode, String stdout, String stderr) throws IOException {
        Files.writeString(dir.resolve("stdout.txt"), stdout, StandardCharsets.UTF_8);
        Files.writeString(dir.resolve("stderr.txt"), stderr, StandardCharsets.UTF_8);
        Path script = dir.resolve("gitvault");
        Files.writeString(
                script,
                "#!/bin/sh\nd=$(dirname \"$0\")\nprintf '%s\\n' \"$@\" > \"$d/args.txt\"\n"
                        + "cat \"$d/stdout.txt\"\ncat \"$d/stderr.txt\" >&2\nexit " + exitCode + "\n",
                StandardCharsets.UTF_8);
        if (!script.toFile().setExecutable(true)) {
            throw new IOException("cannot make the fake CLI executable");
        }
        return new FakeCli(dir);
    }

    public static FakeCli withVariables(Path dir, String variablesJson) throws IOException {
        return create(dir, 0, "{\"variables\":" + variablesJson + "}", "");
    }

    public String bin() {
        return dir.resolve("gitvault").toString();
    }

    /** The arguments of the last call. */
    public List<String> args() throws IOException {
        return Files.readAllLines(dir.resolve("args.txt"));
    }
}
