import java.util.HashMap;
import java.util.function.IntSupplier;

class Basic implements IntSupplier {

    static int staticInt = 42;
    public static Basic running = new Basic();
    public static Basic secondInstance = new Basic();

    public long ticks = 0;

    final String unused = "hello";

    public void tick() {
        ++ticks;
    }

    @Override
    public int getAsInt() {
        try {
            // make sure nested is absolutely totally surely loaded
            Class.forName("Basic$NestedClass");
        } catch (ClassNotFoundException e) {
            throw new RuntimeException(e);
        }

        // load the inner classes
        ping(getClass().getClasses());
        ping(HashMap.class.getClasses());

        System.out.println("up"); // tell the test we're ready

        while (true) {
            tick();
            try {
                Thread.sleep(500L);
            } catch (InterruptedException e) {
                break;
            }
        }
        return 0;
    }

    public static void main(String[] args) throws Exception {
        System.exit(running.getAsInt());
    }

    private static void ping(Object ignored) {
        // noop lol
    }

    class NestedClass {
        float field;
    }

    interface NestedInterface {

        void call();
    }
}
