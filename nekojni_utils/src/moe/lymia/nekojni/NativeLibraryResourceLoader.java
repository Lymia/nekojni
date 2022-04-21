package moe.lymia.nekojni;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.stream.Stream;

// we do this on purpose to allow the generated class to be modified.
@SuppressWarnings({"FieldMayBeFinal", "FieldCanBeLocal"})
public final class NativeLibraryResourceLoader {
    private NativeLibraryResourceLoader() {}

    private static volatile boolean IS_INIT_COMPLETED;
    private static volatile boolean IS_POISONED;
    private static volatile FileLock NATIVE_LOCK;

    private static final int OS_WINDOWS = 1;
    private static final int OS_MACOS = 2;
    private static final int OS_LINUX = 3;

    private static final int ARCH_X86 = 4;
    private static final int ARCH_AMD64 = 5;
    private static final int ARCH_AARCH64 = 6;

    private static String LIBRARY_NAME = "[LIBRARY_NAME]";
    private static String LIBRARY_VERSION = "[LIBRARY_VERSION]";
    private static String IMAGE_RESOURCE_PREFIX = "[IMAGE_RESOURCE_PREFIX]";

    private static int getOperatingSystem() {
        String osName = System.getProperty("os.name");
        if (osName.startsWith("Windows ")) {
            return OS_WINDOWS;
        } else if (osName.startsWith("Mac ")) {
            return OS_MACOS;
        } else if (osName.startsWith("Linux")) {
            return OS_LINUX;
        } else {
            throw new RuntimeException("Your operating system (" + osName + ") is not supported!");
        }
    }
    private static int getArchitecture() {
        String archName = System.getProperty("os.arch");
        switch (archName) {
            case "x86":
            case "i386":
            case "i486":
            case "i586":
            case "i686":
                return ARCH_X86;
            case "amd64":
            case "x86_64":
                return ARCH_AMD64;
            case "aarch64":
                return ARCH_AARCH64;
            default:
                throw new RuntimeException("Your CPU architecture (" + archName + ") is not supported!");
        }
    }
    private static String getTargetName(int os, int arch) {
        String accum = "";
        accum += arch == ARCH_X86 ? "x86" :
                 arch == ARCH_AMD64 ? "x86_64" :
                 arch == ARCH_AARCH64 ? "aarch64" : null;
        accum += os == OS_WINDOWS ? "-pc-windows-msvc" :
                 os == OS_MACOS ? "-apple-darwin" :
                 os == OS_LINUX ? "-unknown-linux-gnu" : null;
        return accum;
    }
    private static String getLibraryName(int os, int arch, String target, String hash, boolean isBinary) {
        String accum = "";
        if (os == OS_MACOS || os == OS_LINUX) accum += "lib";
        accum += LIBRARY_NAME + "-" + LIBRARY_VERSION + "." + target;
        if (isBinary) {
            accum += "." + hash;
            accum += os == OS_WINDOWS ? ".dll" :
                     os == OS_MACOS ? ".dylib" :
                     os == OS_LINUX ? ".so" : null;
        } else {
            accum += ".hash";
        }
        return accum;
    }
    private static Path getLibraryStore() throws IOException {
        Path userHome = Paths.get(System.getProperty("user.home"));
        userHome = userHome.resolve(".nekojni");
        userHome = userHome.resolve("native_libs");
        userHome = userHome.resolve(LIBRARY_NAME);
        Files.createDirectories(userHome);
        return userHome;
    }

    private static byte[] readStream(InputStream inputStream) throws IOException {
        byte[] buf = new byte[16 * 1024];
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int bytesRead;
        while ((bytesRead = inputStream.read(buf, 0, buf.length)) != -1)
            bytes.write(buf, 0, bytesRead);
        return bytes.toByteArray();
    }
    private static synchronized void loadNativeLibrary() throws IOException {
        // gather information about the platform
        int os = getOperatingSystem();
        int arch = getArchitecture();
        String target = getTargetName(os, arch);
        String hashFileName = getLibraryName(os, arch, target, null, false);

        // retrieve the hash metadata for the file
        String hashResName = "/" + IMAGE_RESOURCE_PREFIX + "/" + hashFileName;
        String hash = "";
        try (InputStream hashData = NativeLibraryResourceLoader.class.getResourceAsStream(hashResName)) {
            if (hashData == null) {
                throw new RuntimeException("Native binary for your platform was not found: "+hashResName);
            }
            hash = new String(readStream(hashData), StandardCharsets.UTF_8);
        }

        // find paths on-disk for our caches
        Path imageCachePath = getLibraryStore();
        String binaryResName = getLibraryName(os, arch, target, hash, true);

        // load the native library file to allow deletion to work properly
        Path imageLockPath = imageCachePath.resolve(binaryResName+".lock");
        FileChannel channel = FileChannel.open(imageLockPath, StandardOpenOption.READ, StandardOpenOption.WRITE,
                StandardOpenOption.CREATE);
        FileLock sharedLock = channel.lock(0, 0, true);
        if (!sharedLock.isShared()) {
            throw new RuntimeException("Could not create shared lock for native binary!");
        }
        NATIVE_LOCK = sharedLock;

        // copy the native library to disk
        Path imageTargetPath = imageCachePath.resolve(binaryResName);
        if (!Files.exists(imageTargetPath)) {
            String binResName = "/" + IMAGE_RESOURCE_PREFIX + "/" + binaryResName;
            try (InputStream resourceData = NativeLibraryResourceLoader.class.getResourceAsStream(binResName)) {
                if (resourceData == null) {
                    throw new RuntimeException("Native binary for your platform was not found: "+binResName);
                }
                Files.write(imageTargetPath, readStream(resourceData));
            }
        }

        // load the native library itself
        System.load(imageTargetPath.toString());

        // try to clean up old binaries
        try (Stream<Path> path = Files.list(imageCachePath)) {
            for (Path binaryName : path.toArray(Path[]::new)) {
                if (binaryName.getFileName().toString().endsWith(".lock")) continue;
                if (binaryName.getFileName().toString().equals(binaryResName)) continue;

                Path lockName = binaryName.getParent().resolve(binaryName.getFileName().toString() + ".lock");
                if (!Files.exists(lockName)) {
                    // deleting an old binary without even a lock file existing
                    Files.deleteIfExists(binaryName);
                } else {
                    // try locking the lock exclusively.
                    try (FileChannel deleteLockChannel = FileChannel.open(lockName, StandardOpenOption.READ,
                            StandardOpenOption.WRITE, StandardOpenOption.CREATE)) {
                        try (FileLock lock = deleteLockChannel.tryLock()) {
                            if (lock != null) {
                                Files.deleteIfExists(binaryName);
                                lock.close();
                                Files.deleteIfExists(lockName);
                            }
                        }
                    }
                }
            }
        }
    }

    private static synchronized void checkInit() {
        if (IS_POISONED) throw new RuntimeException("Native library already failed to load, refusing to try again.");

        try {
            loadNativeLibrary();
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
