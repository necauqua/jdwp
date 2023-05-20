use std::{
    error::Error,
    format,
    io::{BufRead, BufReader},
    net::TcpListener,
    ops::{Deref, DerefMut},
    process::{Child, Command, Stdio},
};

use jdwp::client::JdwpClient;

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct JvmHandle {
    jdwp_client: JdwpClient,
    jvm_process: Child,
    port: u16,
}

impl Deref for JvmHandle {
    type Target = JdwpClient;

    fn deref(&self) -> &Self::Target {
        &self.jdwp_client
    }
}

impl DerefMut for JvmHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.jdwp_client
    }
}

impl Drop for JvmHandle {
    fn drop(&mut self) {
        self.jvm_process.kill().expect("Failed to kill the JVM");
        self.jvm_process
            .wait()
            .expect("Failed to wait for JVM to die");
        // just in case
        log::info!("killed a JVM with JDWP port: {}", self.port);
    }
}

fn ensure_fixture_is_compiled(fixture: &str) -> Result<String> {
    // omg wtf is this, Rust, no capitalize?
    let capitalized = {
        let mut s = String::new();
        let mut c = fixture.chars();
        if let Some(ch) = c.next() {
            s.push(ch.to_ascii_uppercase());
        }
        c.for_each(|ch| s.push(ch));
        s
    };

    let class = format!("target/java/{}.class", capitalized);

    // make sure we don't compile the same thing more than once
    if std::fs::metadata(&class).is_ok() {
        return Ok(capitalized);
    }
    let lock =
        named_lock::NamedLock::create(&format!("jdwp_tests_java_fixture_compilation_{fixture}"))?;
    let _guard = lock.lock()?;

    if std::fs::metadata(&class).is_ok() {
        return Ok(capitalized);
    }

    std::fs::create_dir_all("target/java")?;

    log::info!("Compiling the java fixture: {fixture}");

    Command::new("javac")
        .args([
            &format!("tests/fixtures/{}.java", capitalized),
            "-d",
            "target/java",
        ])
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    Ok(capitalized)
}

pub fn launch_and_attach(fixture: &str) -> Result<JvmHandle> {
    // ensure the logger was init
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let class = ensure_fixture_is_compiled(fixture)?;

    let port = TcpListener::bind(("localhost", 0))?.local_addr()?.port();
    log::info!("starting a JVM with JDWP port: {}", port);

    let mut jvm_process = Command::new("java")
        .arg(format!(
            "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address={}",
            port
        ))
        .args(["-cp", "target/java", &class])
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // literally to disable _JAVA_OPTIONS spam
        .spawn()
        .expect("Failed to start the JVM");

    // Wait for the output to ensure this JVM is fully up

    let mut stdout = BufReader::new(jvm_process.stdout.take().unwrap()).lines();

    // "Listening for transport dt_socket at address: {port}"
    // for some reason this is printed directly to stdout, not stderr
    let _debug_line = stdout.next().unwrap()?;

    // "up" is printed by the java fixture class
    assert_eq!(stdout.next().unwrap()?, "up");

    let jdwp_client = JdwpClient::attach(("localhost", port)).expect("Can't connect to the JVM");

    Ok(JvmHandle {
        jdwp_client,
        jvm_process,
        port,
    })
}
