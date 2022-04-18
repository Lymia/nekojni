package moe.lymia.nekojni;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class NativeLibraryResourceLoader {
    private NativeLibraryResourceLoader() {}

    private static volatile boolean IS_INIT_COMPLETED;
    private static volatile boolean IS_POISONED;

    private static final int OS_WINDOWS = 1;
    private static final int OS_MACOS = 2;
    private static final int OS_LINUX = 3;

    private static final int ARCH_X86 = 4;
    private static final int ARCH_AMD64 = 5;
    private static final int ARCH_AARCH64 = 6;

    private static final String LIBRARY_NAME = "[LIBRARY_NAME]";
    private static final String LIBRARY_VERSION = "[LIBRARY_VERSION]";
    private static final String IMAGE_RESOURCE_PREFIX = "[IMAGE_RESOURCE_PREFIX]";

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
        accum += os == OS_WINDOWS ? "-windows-msvc" :
                 os == OS_MACOS ? "-apple-darwin" :
                 os == OS_LINUX ? "-unknown-linux-gnu" : null;
        return accum;
    }
    private static String getLibraryName(int os, int arch, boolean isDisk) {
        String accum = "";
        if (os == OS_MACOS || os == OS_LINUX) accum += "lib";
        accum += LIBRARY_NAME;
        if (isDisk) {
            accum += "-" + LIBRARY_VERSION;
            accum += arch == ARCH_X86 ? ".x86" :
                     arch == ARCH_AMD64 ? ".x86_64" :
                     arch == ARCH_AARCH64 ? ".aarch64" : null;
        }
        accum += os == OS_WINDOWS ? ".dll" :
                 os == OS_MACOS ? ".dylib" :
                 os == OS_LINUX ? ".so" : null;
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
        int os = getOperatingSystem();
        int arch = getArchitecture();

        String target = getTargetName(os, arch);
        String libraryNameResource = getLibraryName(os, arch, false);
        String libraryNameDisk = getLibraryName(os, arch, true);

        Path imageCachePath = getLibraryStore();
        Path imageTargetPath = imageCachePath.resolve(libraryNameResource);
        if (!Files.exists(imageTargetPath)) {
            String resourceName = IMAGE_RESOURCE_PREFIX + "/" + target + "/" + libraryNameDisk;
            try (InputStream resourceData = NativeLibraryResourceLoader.class.getResourceAsStream(resourceName)) {
                if (resourceData == null) {
                    throw new RuntimeException("Native binary for your platform was not found.");
                }
                Files.write(imageTargetPath, readStream(resourceData));
            }
        }

        System.loadLibrary(imageTargetPath.toString());

        // TODO: Code for cleaning up old binaries, don't just dump them to disk.
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
