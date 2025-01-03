use std::backtrace::Backtrace;
use std::cell::RefCell;
#[cfg(feature = "network")]
use std::io::Read;
use std::io::{stdout, Seek, SeekFrom, Write};
#[cfg(feature = "network")]
use std::net::{TcpListener, ToSocketAddrs};
use std::os::windows::fs::FileExt;
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

/// This needs to be set for libucrt to be able to do "dynamic" dispatch to optimized float
/// intrinsics. We just hardcode it to 0 here - feel free to implement CPUID detection and set it to
/// one of the following values:
///
/// __ISA_AVAILABLE_X86     equ 0
/// __ISA_AVAILABLE_SSE2    equ 1
/// __ISA_AVAILABLE_SSE42   equ 2
/// __ISA_AVAILABLE_AVX     equ 3
#[cfg(feature = "float")]
#[unsafe(no_mangle)]
#[used]
#[allow(non_upper_case_globals)]
pub static __isa_available: std::ffi::c_int = 0;

fn main() {
    test_stdout();
    test_thread_locals();
    test_mutex();
    test_rwlock();
    test_condvar();

    test_panic_unwind();
    test_backtrace();

    test_time_and_sleep();
    test_home_dir();
    test_hashset_random_init();

    test_file_seek_truncate_append_fileext();
    test_readdir();
    test_delete_dir_all();

    test_process_stdio_redirect();

    #[cfg(feature = "float")]
    {
        test_float_intrinsics();
    }

    #[cfg(feature = "stdin")]
    {
        test_stdin();
    }

    #[cfg(feature = "network")]
    {
        test_sockaddr();
        test_tcp();
    }
}

#[inline(never)]
fn test_readdir() {
    println!("Reading current directory:");
    for entry in std::fs::read_dir(".").unwrap().flatten() {
        println!("  - {}", entry.path().display());
    }
}

#[inline(never)]
fn test_delete_dir_all() {
    let base_folder = r".\_r9xtmp";
    let folder = &format!(r"{base_folder}\a\b\c");
    std::fs::create_dir_all(folder).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!(r"{folder}\rust9x.txt"))
        .unwrap();
    file.write_all(b"Hello world!").unwrap();
    file.flush().unwrap();
    drop(file);

    std::thread::sleep(Duration::from_millis(500));

    std::fs::remove_dir_all(base_folder).unwrap();
    assert!(!std::fs::exists(base_folder).unwrap());
}

#[inline(never)]
fn test_stdout() {
    println!("Testing UTF-8 console stdout fallback: ÄÖÜß, 你好，世界 🦀🦀");
}

#[cfg(feature = "stdin")]
#[inline(never)]
fn test_stdin() {
    println!("input some string, enter to continue");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    println!("{}", buffer);
}

#[inline(never)]
fn test_home_dir() {
    #[allow(deprecated)]
    let home_dir = std::env::home_dir();
    println!("Home dir: {:?}", home_dir.as_ref().map(|p| p.display()));
}

#[inline(never)]
fn test_hashset_random_init() {
    let mut set = std::collections::HashSet::with_capacity(16);
    for i in 0..16 {
        set.insert(i);
    }

    print!("Hashset:");
    for i in set {
        print!(" {}", i);
    }
    println!();
}

#[inline(never)]
fn test_time_and_sleep() {
    let now = SystemTime::now();
    println!("System time: {now:?}");
    match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => println!("  Duration since unix epoch: {}s", d.as_secs()),
        Err(_) => println!("  Duration since unix epoch: Error: SystemTime before UNIX EPOCH!"),
    }
    println!("Testing sleep");
    thread::sleep(Duration::from_millis(10));
    thread::yield_now();
    let now = SystemTime::now();
    println!("System time: {now:?}");
}

#[cfg(feature = "float")]
#[inline(never)]
fn test_float_intrinsics() {
    let a: f64 = 0.75;
    println!("1: {}, 0: {}", a.round(), a.trunc());
}

#[inline(never)]
fn test_backtrace() {
    let backtrace = Backtrace::capture();
    println!("Testing backtrace, might need RUST_BACKTRACE=1 or =full:\n{backtrace}");
}

#[inline(never)]
fn test_file_seek_truncate_append_fileext() {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("rust9x.txt")
        .unwrap();

    file.write_all(b"Hello world!").unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(b"bye  ").unwrap();
    file.set_len(9).unwrap();
    file.flush().unwrap();
    drop(file);

    let s = std::fs::read_to_string("rust9x.txt").unwrap();

    println!("File after write/seek/set_len: {:?}", s);
    assert_eq!(&s, "bye   wor");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open("rust9x.txt")
        .unwrap();

    // just assuming that the whole buffer gets written in this case
    file.seek_write(b"Hello", 0).unwrap();
    file.flush().unwrap();
    drop(file);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("rust9x.txt")
        .unwrap();

    file.write_all(b"ld!").unwrap();
    file.flush().unwrap();
    drop(file);

    let s = std::fs::read_to_string("rust9x.txt").unwrap();

    println!("File after seek_write/append: {:?}", s);
    assert_eq!(&s, "Hello world!");

    std::fs::remove_file("rust9x.txt").unwrap();
}

struct ThreadLocalPrintOnDrop {
    val: u32,
}

impl Drop for ThreadLocalPrintOnDrop {
    fn drop(&mut self) {
        println!("Thread local dropped, value: {}", self.val);
    }
}

thread_local! {
    #[allow(clippy::missing_const_for_thread_local)]
    static FOO: RefCell<ThreadLocalPrintOnDrop> =
        RefCell::new(ThreadLocalPrintOnDrop { val: 0 });
}

#[inline(never)]
fn test_thread_locals() {
    let i = thread::spawn(|| {
        FOO.with(|n| n.borrow_mut().val = 42);
        println!(
            "set one thread's local to be 42, ending that thread now. It should print the value..."
        );
    });

    i.join().unwrap();
}

#[inline(never)]
fn test_process_stdio_redirect() {
    println!(r"Running `hh3gf.golden.exe`, should print `Hello, World!\r\n`");
    let output = Command::new("./hh3gf.golden.exe")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    if let Ok(s) = std::str::from_utf8(&output.stdout) {
        println!("  Redirected stdout: {}", s);
        println!("  Redirected stderr len: {}", output.stderr.len());
        assert_eq!(s, "Hello, World!\r\n");
    } else {
        panic!("Output was not valid utf8");
    }
}

#[inline(never)]
fn test_mutex() {
    let m = Arc::new(Mutex::new(5usize));

    let mut map = std::collections::HashMap::with_capacity(16);

    let mut num_guard = m.lock().unwrap();
    *num_guard = 0;

    {
        let mut l = stdout().lock();
        write!(l, "Mutex: ").unwrap();
        l.flush().unwrap();
    }

    for n in 0usize..16 {
        let m = m.clone();
        map.insert(
            n,
            thread::spawn(move || {
                thread::sleep(Duration::from_millis((n * 100) as u64));
                loop {
                    let mut num_guard = m.lock().unwrap();

                    if *num_guard == n {
                        *num_guard += 1;
                        break;
                    }

                    drop(num_guard);

                    thread::sleep(Duration::from_millis(100))
                }
            }),
        );
    }

    drop(num_guard);

    for (_, v) in map {
        v.join().unwrap();
    }

    println!("done ({:?})", *m.lock().unwrap());
}

#[inline(never)]
fn test_rwlock() {
    let m = Arc::new(RwLock::new(5usize));

    let mut map = std::collections::HashMap::with_capacity(16);

    let mut num_guard = m.write().unwrap();
    *num_guard = 0;

    {
        let mut l = stdout().lock();
        write!(l, "RwLock:").unwrap();
        l.flush().unwrap();
    }

    for n in 0usize..16 {
        let m = m.clone();
        map.insert(
            n,
            thread::spawn(move || {
                thread::sleep(Duration::from_millis((n * 100) as u64));
                let mut num_guard = m.write().unwrap();

                *num_guard += 1;

                drop(num_guard);
            }),
        );
    }

    {
        let m = m.clone();
        thread::spawn(move || loop {
            let read = m.read().unwrap();

            if *read == 16 {
                return;
            }

            {
                let mut l = stdout().lock();
                write!(l, " {read}").unwrap();
                l.flush().unwrap();
            }

            drop(read);

            thread::sleep(Duration::from_millis(50));
        });
    }

    drop(num_guard);

    for (_, v) in map {
        v.join().unwrap();
    }

    println!(" done ({:?})", *m.read().unwrap());
}

#[inline(never)]
fn test_condvar() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    println!("Condvar: starting threads");
    let mut threads = Vec::with_capacity(16);
    for n in 0usize..16 {
        let pair = pair.clone();
        threads.push(thread::spawn(move || {
            let (lock, cvar) = &*pair;
            let mut guard = lock.lock().unwrap();
            while !*guard {
                guard = cvar.wait(guard).unwrap();
            }
            println!("    {:2} woke up", n);
        }));
    }

    thread::sleep(Duration::from_millis(100));
    println!("  causing a spurious wakeup...");
    pair.1.notify_all();

    thread::sleep(Duration::from_millis(100));
    println!("  proper wakeup...");
    *pair.0.lock().unwrap() = true;
    pair.1.notify_all();

    for t in threads {
        t.join().unwrap();
    }
}

#[cfg(feature = "network")]
#[inline(never)]
fn test_sockaddr() {
    println!("Socket addr check for google.com:80:",);
    for addr in ("google.com", 80u16).to_socket_addrs().unwrap() {
        println!("  {:?}", addr);
    }
}

#[cfg(feature = "network")]
#[inline(never)]
fn test_tcp() {
    println!("Tcp: starting server at 0.0.0.0:40004, CTRL+C to exit");
    let listener = TcpListener::bind("0.0.0.0:40004").unwrap();

    let mut v = Vec::with_capacity(1024);

    loop {
        v.clear();

        let (mut stream, addr) = listener.accept().unwrap();
        println!("  connection from {addr:?}");
        let mut buf = [0u8; 1024];

        loop {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            v.extend_from_slice(&buf[..n]);
            if v.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        if v.starts_with(b"GET /") {
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                    Server: rust9x\r\n\
                    Content-Length: 38\r\n\
                    Content-Type: text/html\r\n\
                    Connection: Closed\r\n\r\n\
                    <html><body>Hello World!</body></html>",
                )
                .unwrap();
            stream.flush().unwrap();
        }

        let _ = stream.shutdown(std::net::Shutdown::Both);
        drop(stream);
    }
}

#[cfg(panic = "abort")]
#[inline(never)]
fn test_panic_unwind() {
    // can't test unwinding without unwind
}

#[cfg(panic = "unwind")]
#[inline(never)]
fn test_panic_unwind() {
    fn inner() {
        panic!("woop");
    }

    println!("Testing catch_unwind");
    if std::panic::catch_unwind(inner).is_err() {
        println!("  unwind caught panic successfully");
    } else {
        // if the panic *wasn't* caught we wouldn't get here anyways
        unreachable!();
    }
}
