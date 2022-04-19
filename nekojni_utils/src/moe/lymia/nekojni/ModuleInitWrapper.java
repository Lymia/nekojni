package moe.lymia.nekojni;

public final class ModuleInitWrapper {
    private ModuleInitWrapper() {}

    private static volatile boolean IS_INIT_COMPLETED;
    private static volatile boolean IN_NATIVE_INIT;
    private static volatile boolean IS_POISONED;

    private static native void initialize();

    private static synchronized void checkInit() {
        if (IS_POISONED) throw new RuntimeException("Native library already failed to load, refusing to try again.");
        if (IS_INIT_COMPLETED) return;
        if (IN_NATIVE_INIT) return;

        try {
            NativeLibraryNullLoader.init();
            IN_NATIVE_INIT = true;
            initialize();
            IS_INIT_COMPLETED = true;
        } catch (Exception e) {
            IS_POISONED = true;
            throw new RuntimeException("Failed to load native library.", e);
        }
    }

    public static void init() {
        if (!IS_INIT_COMPLETED) {
            checkInit();
        }
    }
}
