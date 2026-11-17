package io.github.adhil72.gitvault;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.Map;
import org.junit.jupiter.api.Test;

class MiniJsonTest {

    @Test
    void readsNestedValues() {
        Object parsed = MiniJson.parse(" {\"a\":[1,true,null,\"x\"],\"b\":{\"c\":-2.5e1}} ");

        Map<?, ?> map = (Map<?, ?>) parsed;
        assertEquals(java.util.Arrays.asList(1.0, true, null, "x"), map.get("a"));
        assertEquals(-25.0, ((Map<?, ?>) map.get("b")).get("c"));
    }

    @Test
    void decodesSurrogatePairs() {
        assertEquals("\uD83D\uDE00", MiniJson.parse("\"\\ud83d\\ude00\""));
    }

    @Test
    void rejectsMalformedInputWithoutQuotingIt() {
        for (String bad : new String[] {"{", "{\"a\":}", "[1,]", "\"abc", "{\"a\":1} x", "\"\\q\""}) {
            IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> MiniJson.parse(bad));
            assertEquals(false, e.getMessage().contains("abc"));
        }
    }
}
