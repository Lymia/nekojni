package moe.lymia.nekojni;

import java.io.PrintStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.function.Consumer;

public final class ModuleException extends RuntimeException {
    @SuppressWarnings("unused")
    private static final int njni$$EXCEPTION_CLASS_MARKER_v1 = 0;

    private final ArrayList<String> rustTraces = new ArrayList<>();

    public ModuleException() {
    }

    public ModuleException(String message) {
        super(message);
    }

    public ModuleException(String message, Throwable cause) {
        super(message, cause);
    }

    public ModuleException(Throwable cause) {
        super(cause);
    }

    public ModuleException(String message, Throwable cause, boolean enableSuppression, boolean writableStackTrace) {
        super(message, cause, enableSuppression, writableStackTrace);
    }

    @Override
    public void printStackTrace(PrintStream s) {
        synchronized (s) {
            printStackTrace(s::println);
        }
    }

    @Override
    public void printStackTrace(PrintWriter s) {
        synchronized (s) {
            printStackTrace(s::println);
        }
    }

    private void printStackTrace(Consumer<Object> s) {
        StringWriter sw = new StringWriter();
        PrintWriter pw = new PrintWriter(sw);
        super.printStackTrace(pw);
        String stackTraceString = sw.toString().trim();

        String[] split = stackTraceString.split("\n", 3);
        s.accept(split[0]);
        for (String line : rustTraces)
            s.accept(line);
        if (split.length > 1) {
            if (rustTraces.isEmpty() || !split[1].contains("(Native Method)"))
                s.accept(split[1]);
        }
        if (split.length > 2) s.accept(split[2]);
    }

    public void addRustTraceLine(String s) {
        rustTraces.add(s);
    }
}
