package io.github.adhil72.gitvault;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class GitVaultTest {

    @TempDir
    Path dir;

    @BeforeEach
    void posixOnly() {
        assumeTrue(FakeCli.supported(), "the fake CLI is a shell script");
    }

    @Test
    void exportsVariablesAndUnescapesValues() throws IOException {
        FakeCli cli = FakeCli.withVariables(dir, "{\"API_KEY\":\"dev-key\",\"TRICKY\":\"a\\\"b\\\\c\\n\\u00e9\"}");

        Map<String, String> result = GitVault.builder().bin(cli.bin()).build().exportEnvironment("development");

        assertEquals("dev-key", result.get("API_KEY"));
        assertEquals("a\"b\\c\n\u00e9", result.get("TRICKY"));
        assertEquals(List.of("--json", "--non-interactive", "env", "export", "development"), cli.args());
    }

    @Test
    void passesTheVaultWhenGiven() throws IOException {
        FakeCli cli = FakeCli.withVariables(dir, "{}");

        GitVault.builder().bin(cli.bin()).vault("v1").build().exportEnvironment("production");

        assertEquals(List.of("--json", "--non-interactive", "--vault", "v1", "env", "export", "production"), cli.args());
    }

    @Test
    void cliErrorsKeepTheirStableCode() throws IOException {
        String body = "{\"error\":{\"code\":\"E_LOCKED\",\"message\":\"vault is locked\",\"hint\":\"trust this device\"}}";
        FakeCli cli = FakeCli.create(dir, 1, body, "");

        GitVaultException e =
                assertThrows(GitVaultException.class, () -> GitVault.builder().bin(cli.bin()).build().exportEnvironment("x"));

        assertEquals("E_LOCKED", e.getCode());
        assertEquals("trust this device", e.getHint());
    }

    @Test
    void missingBinaryIsNotInstalled() {
        GitVaultException e = assertThrows(
                GitVaultException.class,
                () -> GitVault.builder().bin(dir.resolve("nope").toString()).build().exportEnvironment("x"));

        assertEquals("E_NOT_INSTALLED", e.getCode());
    }

    @Test
    void unreadableOutputReportsStderr() throws IOException {
        FakeCli cli = FakeCli.create(dir, 2, "not json", "boom happened\n");

        GitVaultException e =
                assertThrows(GitVaultException.class, () -> GitVault.builder().bin(cli.bin()).build().exportEnvironment("x"));

        assertEquals("E_INTERNAL", e.getCode());
        assertEquals("boom happened", e.getMessage());
    }

    @Test
    void blankEnvironmentIsAUsageError() {
        GitVaultException e = assertThrows(GitVaultException.class, () -> GitVault.create().exportEnvironment(" "));

        assertEquals("E_USAGE", e.getCode());
    }
}
