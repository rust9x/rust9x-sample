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

fn main() {
    test_home_dir();
    test_hashset_random_init();
    test_time();
    test_backtrace();

    test_file_seek_truncate_append_fileext();

    test_process_stdio_redirect();

    test_thread_locals();
    test_mutex();
    test_rwlock();
    test_condvar();

    test_panic_unwind();

    #[cfg(feature = "network")]
    {
        test_sockaddr();
        test_tcp();
    }
}

fn test_home_dir() {
    #[allow(deprecated)]
    let home_dir = std::env::home_dir();
    println!("Home dir: {:?}", home_dir.as_ref().map(|p| p.display()));
}

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

fn test_time() {
    let now = SystemTime::now();
    println!("System time: {now:?}");
    match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => println!("  Duration since unix epoch: {}s", d.as_secs()),
        Err(_) => println!("  Duration since unix epoch: Error: SystemTime before UNIX EPOCH!"),
    }
}

#[inline(never)]
fn test_backtrace() {
    let backtrace = Backtrace::capture();
    println!("Testing backtrace, might need RUST_BACKTRACE=1 or =full:\n{backtrace}");
}

fn test_file_seek_truncate_append_fileext() {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
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
        println!(
            "Thread local dropped, if unchanged, this should be 42: {}",
            self.val
        );
    }
}

thread_local! {
    static FOO: RefCell<ThreadLocalPrintOnDrop> =
        RefCell::new(ThreadLocalPrintOnDrop{ val: 42 });
}

fn test_thread_locals() {
    let j = thread::spawn(|| {
        FOO.with(|n| *n.borrow_mut() = ThreadLocalPrintOnDrop { val: 43 });
        println!("set one thread's local to be 43, ending that thread now...");
    });

    j.join().unwrap();
}

fn test_process_stdio_redirect() {
    let output = Command::new("./hh3gf.golden.exe")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    if let Ok(s) = std::str::from_utf8(&output.stdout) {
        println!("Redirected stdout: {}", s);
        println!("Redirected stderr len: {}", output.stderr.len());
        assert_eq!(s, "Hello, World!\r\n");
    } else {
        panic!("Output was not valid utf8");
    }
}

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
            println!("    {} woke up", n);
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
fn test_sockaddr() {
    println!("Socket addr check for google.com:80:",);
    for addr in ("google.com", 80u16).to_socket_addrs().unwrap() {
        println!("  {:?}", addr);
    }
}

#[cfg(feature = "network")]
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
fn test_panic_unwind() {
    // can't test unwinding without unwind
}

#[cfg(panic = "unwind")]
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
