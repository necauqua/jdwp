public final class Test {

    public long ticks = 0;

    public void tick() {
        ++ticks;
    }

    public void start() {
        System.out.println("Started");
        while (true) {
            tick();
            try {
                Thread.sleep(50L);
            } catch (InterruptedException e) {
                break;
            }
        }
        System.out.println("Finished");
    }

    public static void main(String[] args) throws Exception {
        Runtime.getRuntime().addShutdownHook(new Thread(() -> System.out.println("Shutdown")));
        new Test().start();
    }
}
