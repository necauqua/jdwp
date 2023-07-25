use std::{
    error::Error,
    fmt::Debug,
    format,
    io::{BufRead, BufReader, ErrorKind},
    net::TcpListener,
    ops::{Deref, DerefMut},
    process::{Child, Command, Stdio},
};

use jdwp::{client::JdwpClient, highlevel::VM};
use lazy_static::lazy_static;

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct JvmHandle<T> {
    pub jvm_process: Child,
    port: u16,
    value: Option<T>,
}

impl<T> JvmHandle<T> {
    // only used in the vm exit test where the vm is taken by value
    // but other tests including the common value flag this as unused
    #[allow(unused)]
    pub fn take(&mut self) -> T {
        self.value.take().expect("Value was moved out")
    }
}

impl<T> Deref for JvmHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().expect("Value was moved out")
    }
}

impl<T> DerefMut for JvmHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().expect("Value was moved out")
    }
}

impl<T> Drop for JvmHandle<T> {
    fn drop(&mut self) {
        match self.jvm_process.kill() {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::InvalidInput => {} // already dead
            #[cfg(unix)]
            r => r.expect("Failed to kill the JVM"),
            #[cfg(not(unix))]
            Err(e) => log::error!("Failed to kill the JVM: {:?}", e),
            // ^ windows gives a PermissionDenied on CI instead of
            // InvalidInput if the process is already dead
        }

        let status = self
            .jvm_process
            .wait()
            .expect("Failed to wait for JVM to die");
        // ^ just in case

        log::info!(
            "JVM with JDWP port {} finished, exit status: {}",
            self.port,
            status.code().unwrap_or_default()
        );
    }
}

fn ensure_fixture_is_compiled(fixture: &str) -> Result<(String, String)> {
    let java_version = java_version();

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
    let dir = format!("target/java_{java_version}");
    let class = format!("{dir}/{capitalized}.class");

    // make sure we don't compile the same thing more than once
    if std::fs::metadata(&class).is_ok() {
        return Ok((dir, capitalized));
    }
    let lock = named_lock::NamedLock::create(&format!(
        "jdwp_tests_java{java_version}_fixture_compilation_{fixture}"
    ))?;
    let _guard = lock.lock()?;

    if std::fs::metadata(&class).is_ok() {
        return Ok((dir, capitalized));
    }

    std::fs::create_dir_all(&dir)?;

    log::info!("Compiling the java fixture: {fixture}");

    Command::new("javac")
        .args([&format!("tests/fixtures/{capitalized}.java"), "-d", &dir])
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    Ok((dir, capitalized))
}

fn launch(fixture: &str) -> Result<(Child, u16)> {
    // ensure the logger was init
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let (classpath, class_name) = ensure_fixture_is_compiled(fixture)?;

    let port = TcpListener::bind(("localhost", 0))?.local_addr()?.port();
    log::info!("Starting a JVM with JDWP port: {port}");

    let mut jvm_process = Command::new("java")
        .arg(format!(
            "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address={port}",
        ))
        .args(["-cp", &classpath, &class_name])
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

    Ok((jvm_process, port))
}

#[allow(unused)] // it's flagged as unused in test binaries that don't use it ¯\_(ツ)_/¯
pub fn launch_and_attach(fixture: &str) -> Result<JvmHandle<JdwpClient>> {
    let (jvm_process, port) = launch(fixture)?;
    let client = JdwpClient::connect(("localhost", port)).expect("Can't connect to the JVM");
    Ok(JvmHandle {
        jvm_process,
        port,
        value: Some(client),
    })
}

#[allow(unused)] // ditto
pub fn launch_and_attach_vm(fixture: &str) -> Result<JvmHandle<VM>> {
    let (jvm_process, port) = launch(fixture)?;
    let vm = VM::connect(("localhost", port)).expect("Can't connect to the JVM");

    Ok(JvmHandle {
        jvm_process,
        port,
        value: Some(vm),
    })
}

pub fn java_version() -> u32 {
    fn call_javac() -> Result<u32> {
        let mut output = Command::new("javac").arg("-version").output()?;
        output.stderr.extend(output.stdout); // stderr hacks for java 8

        let version = String::from_utf8_lossy(&output.stderr)
            .lines()
            .last() // last line for java 8 as well, I personally have the _JAVA_OPTIONS cluttering stderr
            .unwrap()
            .chars()
            .skip(6) // 'javac '
            .take_while(|ch| ch.is_numeric())
            .collect::<String>()
            .parse()?;

        Ok(match version {
            1 => 8,
            v => v,
        })
    }

    lazy_static! {
        static ref JAVA_VERSION: u32 = call_javac().expect("Failed to get java version");
    };

    *JAVA_VERSION
}

#[macro_export]
macro_rules! assert_snapshot {
    ($e:expr, @$lit:literal) => {
        insta::with_settings!({
            filters => vec![
                (r"((?:ClassLoader|Field|Method|Object|Class|Interface|ArrayType)ID)\(\d+\)", "$1(opaque)"),
            ]
        }, {
            insta::assert_debug_snapshot!($e, @$lit);
        });
    };
}
