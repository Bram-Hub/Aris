package edu.rpi.aris.gui;

import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.input.KeyCombination;
import javafx.scene.input.KeyEvent;

import java.util.HashMap;

public class ConfigurationManager {

    private static final HashMap<String, String> KEY_MAP = new HashMap<>();
    private static final String[][] keyMap = new String[][]{{"&", "∧"}, {"|", "∨"}, {"!", "¬"}, {"~", "¬"}, {"$", "→"}, {"%", "↔"}, {"^", "⊥"}};

    static {
        for (String[] s : keyMap)
            KEY_MAP.put(s[0], s[1]);
    }

    public SimpleObjectProperty<KeyCombination> newProofLineKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+A"));
    public SimpleObjectProperty<KeyCombination> deleteProofLineKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+D"));
    public SimpleObjectProperty<KeyCombination> newPremiseKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+R"));
    public SimpleObjectProperty<KeyCombination> startSubProofKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+P"));
    public SimpleObjectProperty<KeyCombination> endSubProofKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+E"));
    public SimpleObjectProperty<KeyCombination> verifyLineKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+F"));
    public SimpleObjectProperty<KeyCombination> addGoalKey = new SimpleObjectProperty<>(KeyCombination.keyCombination("Ctrl+G"));

    private SimpleObjectProperty[] accelerators = new SimpleObjectProperty[]{newProofLineKey, deleteProofLineKey, startSubProofKey, endSubProofKey, newPremiseKey, verifyLineKey, addGoalKey};

    public static String replaceText(String text) {
        for (int i = 0; i < text.length(); ++i) {
            String replace;
            if ((replace = KEY_MAP.get(String.valueOf(text.charAt(i)))) != null)
                text = text.substring(0, i) + replace + text.substring(i + 1);
        }
        return text;
    }

    public boolean ignore(KeyEvent event) {
        for (SimpleObjectProperty a : accelerators)
            if (((KeyCombination) a.get()).match(event))
                return true;
        return false;
    }
}
