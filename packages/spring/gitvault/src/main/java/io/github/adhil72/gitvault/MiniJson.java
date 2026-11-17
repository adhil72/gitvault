package io.github.adhil72.gitvault;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A small strict JSON reader, so the core library needs no dependency (a secrets library should
 * not pull a JSON stack into the application's classpath). Objects become {@code Map}, arrays
 * {@code List}, strings {@code String}, numbers {@code Double}, plus {@code Boolean} and null.
 */
final class MiniJson {

    private static final int MAX_DEPTH = 64;

    private final String text;
    private int pos;

    private MiniJson(String text) {
        this.text = text;
    }

    static Object parse(String text) {
        MiniJson reader = new MiniJson(text);
        Object value = reader.value(0);
        reader.skipWhitespace();
        if (reader.pos != text.length()) {
            throw reader.fail("unexpected trailing data");
        }
        return value;
    }

    private Object value(int depth) {
        if (depth > MAX_DEPTH) {
            throw fail("nesting too deep");
        }
        skipWhitespace();
        char c = peek();
        switch (c) {
            case '{':
                return object(depth);
            case '[':
                return array(depth);
            case '"':
                return string();
            case 't':
                return literal("true", Boolean.TRUE);
            case 'f':
                return literal("false", Boolean.FALSE);
            case 'n':
                return literal("null", null);
            default:
                return number();
        }
    }

    private Map<String, Object> object(int depth) {
        Map<String, Object> map = new LinkedHashMap<>();
        expect('{');
        skipWhitespace();
        if (tryConsume('}')) {
            return map;
        }
        do {
            skipWhitespace();
            String key = string();
            skipWhitespace();
            expect(':');
            map.put(key, value(depth + 1));
            skipWhitespace();
        } while (tryConsume(','));
        expect('}');
        return map;
    }

    private List<Object> array(int depth) {
        List<Object> list = new ArrayList<>();
        expect('[');
        skipWhitespace();
        if (tryConsume(']')) {
            return list;
        }
        do {
            list.add(value(depth + 1));
            skipWhitespace();
        } while (tryConsume(','));
        expect(']');
        return list;
    }

    private String string() {
        expect('"');
        StringBuilder out = new StringBuilder();
        while (true) {
            char c = next();
            if (c == '"') {
                return out.toString();
            }
            if (c < 0x20) {
                throw fail("control character in string");
            }
            out.append(c == '\\' ? escape() : c);
        }
    }

    private char escape() {
        char c = next();
        switch (c) {
            case '"':
            case '\\':
            case '/':
                return c;
            case 'b':
                return '\b';
            case 'f':
                return '\f';
            case 'n':
                return '\n';
            case 'r':
                return '\r';
            case 't':
                return '\t';
            case 'u':
                return unicode();
            default:
                throw fail("bad escape");
        }
    }

    private char unicode() {
        if (pos + 4 > text.length()) {
            throw fail("bad unicode escape");
        }
        try {
            char c = (char) Integer.parseInt(text.substring(pos, pos + 4), 16);
            pos += 4;
            return c;
        } catch (NumberFormatException e) {
            throw fail("bad unicode escape");
        }
    }

    private Object literal(String word, Object result) {
        if (!text.startsWith(word, pos)) {
            throw fail("unexpected token");
        }
        pos += word.length();
        return result;
    }

    private Double number() {
        int start = pos;
        while (pos < text.length() && "+-0123456789.eE".indexOf(text.charAt(pos)) >= 0) {
            pos++;
        }
        try {
            return Double.valueOf(text.substring(start, pos));
        } catch (NumberFormatException e) {
            throw fail("unexpected token");
        }
    }

    private void skipWhitespace() {
        while (pos < text.length() && Character.isWhitespace(text.charAt(pos))) {
            pos++;
        }
    }

    private char peek() {
        if (pos >= text.length()) {
            throw fail("unexpected end of input");
        }
        return text.charAt(pos);
    }

    private char next() {
        char c = peek();
        pos++;
        return c;
    }

    private void expect(char c) {
        if (next() != c) {
            pos--;
            throw fail("expected '" + c + "'");
        }
    }

    private boolean tryConsume(char c) {
        if (pos < text.length() && text.charAt(pos) == c) {
            pos++;
            return true;
        }
        return false;
    }

    // The position is safe to report; the text itself may hold secret values, so it is never quoted.
    private IllegalArgumentException fail(String reason) {
        return new IllegalArgumentException("invalid JSON at offset " + pos + ": " + reason);
    }
}
