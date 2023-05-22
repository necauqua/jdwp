import java.util.HashMap;

class Basic implements Runnable {

    static int staticInt = 42;
    public static Basic secondInstance = new Basic();

    public long ticks = 0;

    final String unused = "hello";

    public void tick() {
        ++ticks;
    }

    @Override
    public void run() {
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
                Thread.sleep(50L);
            } catch (InterruptedException e) {
                break;
            }
        }
    }

    public static void main(String[] args) throws Exception {
        new Basic().run();
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
