package dev.grafeo.gwp;

import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** Unit tests for {@link Counters}. */
class CountersTest {

    @Test
    void missingKeysReadAsZero() {
        Counters counters = Counters.fromMap(Map.of());
        assertEquals(Counters.NONE, counters);
        assertFalse(counters.containsUpdates());
    }

    @Test
    void readsEveryKeyAndIgnoresOthers() {
        Map<String, Long> raw = new HashMap<>();
        raw.put("execution_time_ms", 99L);
        raw.put("rows_scanned", 1000L);
        for (int i = 0; i < Counters.KEYS.size(); i++) {
            raw.put(Counters.KEYS.get(i), (long) i + 1);
        }

        Counters counters = Counters.fromMap(raw);
        assertEquals(new Counters(1, 2, 3, 4, 5, 6, 7), counters);
        assertTrue(counters.containsUpdates());
    }

    @Test
    void negativeValuesReadAsZero() {
        assertEquals(0, Counters.fromMap(Map.of(Counters.NODES_CREATED, -5L)).nodesCreated());
    }

    @Test
    void singleCounterIsAnUpdate() {
        for (String key : Counters.KEYS) {
            assertTrue(Counters.fromMap(Map.of(key, 1L)).containsUpdates(), key);
        }
    }
}
