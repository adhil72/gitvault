package io.github.adhil72.gitvault;

/**
 * A failure of the {@code gitvault} CLI or of this library. {@link #getCode()} is one of the
 * stable CLI codes ({@code E_LOCKED}, {@code E_PERMISSION_DENIED}, {@code E_NOT_AUTHENTICATED},
 * {@code E_NOT_FOUND}, ...) or one this library adds ({@code E_NOT_INSTALLED}, {@code E_USAGE},
 * {@code E_INTERNAL}). The message never contains a secret value.
 */
public class GitVaultException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    private final String code;
    private final String hint;

    public GitVaultException(String code, String message, String hint) {
        super(hint == null || hint.isBlank() ? message : message + " (" + hint + ")");
        this.code = code;
        this.hint = hint;
    }

    public GitVaultException(String code, String message) {
        this(code, message, null);
    }

    public String getCode() {
        return code;
    }

    /** A short suggestion on how to fix the problem, or {@code null}. */
    public String getHint() {
        return hint;
    }
}
