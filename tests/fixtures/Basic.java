public final class Basic {

    public long ticks = 0;

    public void tick() {
        ++ticks;
    }

    public void start() {
        // write a few bytes to stdout once we're running
        System.out.println("up");
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
        new Basic().start();
    }
}
