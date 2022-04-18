package moe.lymia.nekojni;

public final class ShutdownHook extends Thread {
    private static native void native_shutdown();

    private static volatile boolean IS_SHUTDOWN;
    public synchronized void run() {
        if (IS_SHUTDOWN) {
            throw new RuntimeException("native_shutdown already run!");
        } else {
            IS_SHUTDOWN = true;
            native_shutdown();
        }
    }

    private static volatile boolean IS_INSTALLED;
    public static synchronized void install() {
        if (IS_INSTALLED) {
            throw new RuntimeException("install already run!");
        } else {
            IS_INSTALLED = true;
            Runtime.getRuntime().addShutdownHook(new ShutdownHook());
        }
    }
}
