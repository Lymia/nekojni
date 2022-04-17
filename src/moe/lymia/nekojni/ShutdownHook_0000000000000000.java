package moe.lymia.nekojni;

public final class ShutdownHook_0000000000000000 extends Thread {
   private static volatile boolean IS_SHUTDOWN;
   private static volatile boolean IS_INSTALLED;

   private static native void native_shutdown();

   public synchronized void run() {
      if (IS_SHUTDOWN) {
         throw new RuntimeException("native_shutdown already run!");
      } else {
         IS_SHUTDOWN = true;
         native_shutdown();
      }
   }

   public static synchronized void install() {
      if (IS_INSTALLED) {
         throw new RuntimeException("install already run!");
      } else {
         IS_INSTALLED = true;
         Runtime.getRuntime().addShutdownHook(new ShutdownHook_0000000000000000());
      }
   }
}
