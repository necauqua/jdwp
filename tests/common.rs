use std::{
    error::Error,
    format,
    io::{BufRead, BufReader, ErrorKind},
    net::TcpListener,
    ops::{Deref, DerefMut},
    process::{Child, Command, Stdio},
};

use jdwp::client::JdwpClient;
use lazy_static::lazy_static;

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct JvmHandle {
    jdwp_client: JdwpClient,
    pub jvm_process: Child,
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
        match self.jvm_process.kill() {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::InvalidInput => {} // already dead
            r => r.expect("Failed to kill the JVM"),
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

pub fn launch_and_attach(fixture: &str) -> Result<JvmHandle> {
    // ensure the logger was init
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let (classpath, class_name) = ensure_fixture_is_compiled(fixture)?;

    let port = TcpListener::bind(("localhost", 0))?.local_addr()?.port();
    log::info!("Starting a JVM with JDWP port: {}", port);

    let mut jvm_process = Command::new("java")
        .arg(format!(
            "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address={}",
            port
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

    let jdwp_client = JdwpClient::attach(("localhost", port)).expect("Can't connect to the JVM");

    Ok(JvmHandle {
        jdwp_client,
        jvm_process,
        port,
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
                (r"(?:ClassLoader|Field|Method|Object|Class|Interface|ArrayType)ID\(\d+\)", "[opaque_id]"),
            ]
        }, {
            insta::assert_debug_snapshot!($e, @$lit);
        });
    };
}

pub trait TryMapExt<T> {
    fn try_map<E, F, U, M>(self, f: F) -> std::result::Result<Vec<U>, E>
    where
        F: FnMut(T) -> std::result::Result<U, M>,
        E: From<M>;
}

impl<T, I> TryMapExt<T> for I
where
    I: IntoIterator<Item = T>,
{
    fn try_map<E, F, U, M>(self, mut f: F) -> std::result::Result<Vec<U>, E>
    where
        F: FnMut(T) -> std::result::Result<U, M>,
        E: From<M>,
    {
        self.into_iter().try_fold(Vec::new(), move |mut acc, item| {
            acc.push(f(item)?);
            Ok(acc)
        })
    }
}
