package moe.lymia.nekojni;

public final class LoadBinaryExternally extends Thread {
    private static volatile boolean IS_INIT_COMPLETED;
    private static volatile boolean IS_POISONED;

    private static native void initialize();

    private static synchronized void checkInit() {
        if (IS_POISONED) throw new RuntimeException("Native library already failed to load, refusing to try again.");

        try {
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
