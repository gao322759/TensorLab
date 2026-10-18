//! tl-cli 黑盒测试：以独立进程运行 `tensorlab` 二进制，
//! 只通过 argv / stdin / stdout / stderr / 退出码 观察行为。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tensorlab"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn demo(name: &str) -> PathBuf {
    repo_root().join("demos").join(name)
}

fn tmp_dir() -> PathBuf {
    let d = std::env::temp_dir().join("tensorlab-cli-tests");
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

fn stdout_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

// ---------------- 元信息子命令 ----------------

#[test]
fn version_prints_banner() {
    let out = bin().arg("version").output().expect("spawn");
    assert!(out.status.success(), "exit={:?}", out.status.code());
    let s = stdout_of(&out);
    assert!(
        s.contains(concat!("TensorLab ", env!("CARGO_PKG_VERSION"))),
        "{s}"
    );
    assert!(s.contains("RunMat 0.6.2"), "{s}");
}

#[test]
fn help_lists_host_commands() {
    let out = bin().arg("help").output().expect("spawn");
    assert!(out.status.success());
    let s = stdout_of(&out);
    for want in ["vars", "clear", "figures", "version"] {
        assert!(s.contains(want), "help output missing `{want}`:\n{s}");
    }
}

#[test]
fn unknown_subcommand_fails_loudly() {
    let out = bin().arg("bogus_subcommand").output().expect("spawn");
    assert!(!out.status.success(), "unknown subcommand must fail");
    let err = stderr_of(&out);
    assert!(err.contains("unknown command"), "stderr: {err}");
}

// ---------------- run 子命令 ----------------

#[test]
fn every_demo_script_exits_zero_and_prints() {
    for name in [
        "hello_world.m",
        "matrix_ops.m",
        "plot_demo.m",
        "fft_demo.m",
        "solve_demo.m",
    ] {
        let out = bin()
            .arg("run")
            .arg(demo(name))
            .output()
            .expect("spawn run");
        assert!(
            out.status.success(),
            "`{name}` failed: exit={:?} stderr={}",
            out.status.code(),
            stderr_of(&out)
        );
        assert!(!out.stdout.is_empty(), "`{name}` produced no stdout");
    }
}

#[test]
fn fprintf_output_is_visible_on_stdout() {
    // 回归：控制台输出（fprintf / disp）必须真的写到 stdout
    let out = bin()
        .arg("run")
        .arg(demo("hello_world.m"))
        .output()
        .unwrap();
    let s = stdout_of(&out);
    assert!(s.contains("Hello from TensorLab!"), "{s}");
    assert!(s.contains("concat: TensorLab"), "{s}");
}

#[test]
fn vm_or_jit_tier_is_reported_on_stderr() {
    let out = bin().arg("run").arg(demo("matrix_ops.m")).output().unwrap();
    let err = stderr_of(&out);
    assert!(
        err.contains("[VM ") || err.contains("[JIT "),
        "expected a tier/timing status line on stderr, got: {err}"
    );
}

#[test]
fn missing_script_argument_is_a_usage_error() {
    let out = bin().arg("run").output().expect("spawn");
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("usage"), "{}", stderr_of(&out));
}

#[test]
fn missing_script_file_is_an_error() {
    let out = bin()
        .arg("run")
        .arg(tmp_dir().join("definitely_absent_xyz.m"))
        .output()
        .expect("spawn");
    assert!(!out.status.success());
}

#[test]
fn script_with_runtime_error_exits_one_and_writes_stderr() {
    let path = tmp_dir().join("bad_script.m");
    std::fs::write(&path, "this_is_not_defined_xyz();\n").unwrap();

    let out = bin().arg("run").arg(&path).output().expect("spawn");
    assert_eq!(out.status.code(), Some(1), "runtime error must exit 1");
    let err = stderr_of(&out);
    assert!(err.contains("Error"), "stderr: {err}");
}

// ---------------- REPL ----------------

#[test]
fn repl_evaluates_expressions_and_exits() {
    let mut child = bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn repl");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"1 + 1\ns = 'hi'\nvars\nexit\n")
        .expect("write stdin");

    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "repl exit={:?}", out.status.code());
    let s = stdout_of(&out);
    // 无分号表达式回显为 `ans = ...`（避免用单个数字断言，banner 里也有数字）
    assert!(s.contains("ans"), "expression echo missing:\n{s}");
    assert!(s.contains("hi"), "char variable echo missing:\n{s}");
    // vars 宿主命令应列出 char 类型的 s
    assert!(s.contains("char"), "vars output missing char class:\n{s}");
}

#[test]
fn repl_exits_cleanly_on_stdin_eof() {
    let mut child = bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn repl");
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"2 * 21\n").unwrap();
    }
    // 不发送 exit，直接关闭 stdin → 应在 EOF 处正常退出
    drop(child.stdin.take());
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "EOF must exit 0");
    assert!(stdout_of(&out).contains("42"), "{}", stdout_of(&out));
}

#[test]
fn repl_survives_an_error_and_keeps_going() {
    let mut child = bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn repl");
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"not_defined_xyz()\n3 + 4\nexit\n")
            .unwrap();
    }
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "repl must survive errors");
    let s = stdout_of(&out);
    assert!(
        s.contains('7'),
        "session should continue after an error:\n{s}"
    );
}
