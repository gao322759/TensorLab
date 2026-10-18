//! TensorLab CLI：`tensorlab` REPL + `tensorlab run <script.m>`。

use std::io::{self, BufRead, Write};
use tl_commands::{repl_line, run_file, CommandResult, Engine};
use tl_types::StreamEntryPayload;

fn main() -> anyhow::Result<()> {
    // RunMat 内核的 parser/eval 深递归需要大栈：在 64 MiB 栈的工作线程上运行
    // （测试线程默认栈更大，故 cargo test 中不复现；主线程默认栈不够）
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("spawn engine thread")
        .join()
        .map_err(|_| anyhow::anyhow!("engine thread panicked"))?
}

fn run() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => repl(),
        Some("run") => {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("usage: tensorlab run <script.m>"))?;
            run_script(&path)
        }
        Some("bench") => {
            let rest: Vec<String> = args.collect();
            let no_jit = rest.iter().any(|arg| arg == "--no-jit");
            let aot = rest.iter().any(|arg| arg == "--aot");
            run_bench(!no_jit, aot)
        }
        Some("help") | Some("--help") | Some("-h") => {
            println!("{}", tl_toolbox::help_overview());
            Ok(())
        }
        Some("version") | Some("--version") => {
            println!(
                "TensorLab {} (kernel: RunMat 0.6.2)",
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        }
        Some(other) => Err(anyhow::anyhow!(
            "unknown command '{other}'. usage: tensorlab [run <script.m> | bench [--no-jit] [--aot] | help | version]"
        )),
    }
}

fn run_script(path: &str) -> anyhow::Result<()> {
    let mut engine = Engine::new()?;
    let report = run_file(&mut engine, path)?;
    print_report(&report);
    if let Some(err) = &report.error {
        let line = err.line.map(|l| format!(" (line {l})")).unwrap_or_default();
        eprintln!("Error{line}: {}", err.message);
        std::process::exit(1);
    }
    Ok(())
}

fn run_bench(enable_jit: bool, aot: bool) -> anyhow::Result<()> {
    let mut engine = Engine::with_jit(enable_jit)?;
    let dir = std::env::temp_dir().join("tensorlab-bench");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(
        dir.join("tl_bench_loop.m"),
        "function s = tl_bench_loop(n)\n\
             s = 0;\n\
             for i = 1:n\n\
                 s = s + i;\n\
             end\n\
         end\n",
    )?;
    let dir_path = dir.to_string_lossy().replace('\\', "/");
    let setup = engine.run("<bench-setup>", &format!("addpath('{dir_path}');"));
    if let Some(err) = setup.error {
        anyhow::bail!("bench setup failed: {}", err.message);
    }

    let workloads: &[(&str, &str)] = &[
        (
            "script_loop_5k",
            "n = 5000 + (rand() > 2); benchmark = 0; for i = 1:n; benchmark = benchmark + i; end",
        ),
        (
            "function_calls_100",
            "m = 50 + (rand() > 2); benchmark = 0; for k = 1:100; benchmark = benchmark + tl_bench_loop(m); end",
        ),
        (
            "matmul_300",
            "A = ones(300); B = ones(300); for k = 1:5; C = A * B; end",
        ),
        (
            "fft_16k",
            "x = ones(1, 16384); for k = 1:20; y = fft(x); end",
        ),
        (
            "vector_math_100k",
            "x = (1:100000); for k = 1:10; y = sin(x) + cos(x); end",
        ),
    ];

    const WARMUP: usize = 10;
    const REPS: usize = 5;
    println!(
        "TensorLab JIT benchmark (RunMat 0.6.2, JIT {})",
        if enable_jit { "on" } else { "off" }
    );
    println!("workload             cold_ms  steady_ms    tier  speedup");
    for (name, src) in workloads {
        let mut cold = None;
        let mut jit = false;
        let mut steady = Vec::with_capacity(REPS);
        for run in 0..(WARMUP + REPS) {
            let report = engine.run(name, src);
            if let Some(err) = report.error {
                anyhow::bail!("benchmark {name} failed: {}", err.message);
            }
            if run == 0 {
                cold = Some(report.duration_ms);
            }
            jit |= report.used_jit;
            if run >= WARMUP {
                steady.push(report.duration_ms);
            }
        }
        let steady_ms = median(&steady);
        let cold_ms = cold.unwrap_or(steady_ms);
        let speedup = if steady_ms > 0.0 {
            cold_ms / steady_ms
        } else {
            1.0
        };
        println!(
            "{:<20} {:>8.1} {:>10.1} {:>7} {:>8.2}x",
            name,
            cold_ms,
            steady_ms,
            if jit { "JIT" } else { "VM" },
            speedup
        );
    }
    println!("\n注：JIT 采用分层编译，前 {WARMUP} 次为预热；steady_ms 取最后 {REPS} 次的中位数。");
    if aot {
        run_aot_probe(&mut engine, &dir)?;
    }
    let stats = engine.kernel_stats();
    println!(
        "内核统计：executions={} jit_compiled={} interpreter_fallback={} osr={} vectorized_regions={}",
        stats.total_executions,
        stats.jit_compiled,
        stats.interpreter_fallback,
        stats.native_osr_transfers,
        stats.vectorized_native_regions
    );
    println!(
        "MATLAB 对比请运行 bench/matlab_bench.m（同一 workload），记录到 docs/jit-benchmarks.md。"
    );
    Ok(())
}

fn run_aot_probe(engine: &mut Engine, dir: &std::path::Path) -> anyhow::Result<()> {
    let path = dir.join("tl_bench_loop.m").to_string_lossy().to_string();
    let compiled = match engine.compile_function_path(&path, "tl_bench_loop") {
        Ok(compiled) => compiled,
        Err(err) => {
            eprintln!("AOT 探针跳过：{err}");
            return Ok(());
        }
    };
    for _ in 0..3 {
        let _ = engine.invoke_compiled_function(&compiled, &[50.0]);
    }
    let started = std::time::Instant::now();
    let mut sum = 0.0;
    for _ in 0..100 {
        match engine.invoke_compiled_function(&compiled, &[50.0]) {
            Ok(value) => sum += value,
            Err(err) => {
                eprintln!("AOT 探针失败：{err}");
                return Ok(());
            }
        }
    }
    let total = started.elapsed().as_secs_f64() * 1000.0;
    println!(
        "AOT function calls: 100 × tl_bench_loop(50) = {total:.1} ms total, {:.3} ms/call (sum={sum})",
        total / 100.0
    );
    Ok(())
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted[sorted.len() / 2]
}

fn repl() -> anyhow::Result<()> {
    let mut engine = Engine::new()?;
    println!(
        "TensorLab {} (kernel: RunMat 0.6.2) — type 'help' for host commands",
        env!("CARGO_PKG_VERSION")
    );
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!(">> ");
        stdout.flush()?;
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line)?;
        if n == 0 {
            println!();
            break; // EOF
        }
        let trimmed = line.trim();
        if matches!(trimmed, "exit" | "quit" | "exit()" | "quit()") {
            break;
        }
        match repl_line(&mut engine, trimmed) {
            CommandResult::None => {}
            CommandResult::Text(t) => {
                print!("{t}");
                if !t.ends_with('\n') {
                    println!();
                }
            }
            CommandResult::Report(r) => print_report(&r),
        }
    }
    Ok(())
}

fn print_report(report: &tl_types::RunReport) {
    let mut out = String::new();
    for e in &report.events {
        match e {
            StreamEntryPayload::Output { text } => {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            StreamEntryPayload::Error { text } => {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            StreamEntryPayload::ClearConsole => out.clear(),
        }
    }
    print!("{out}");
    if let Some(err) = &report.error {
        let line = err.line.map(|l| format!(" (line {l})")).unwrap_or_default();
        eprintln!("Error{line}: {}", err.message);
    }
    for w in &report.warnings {
        eprintln!("Warning: {}", w.message);
    }
    let tier = if report.used_jit { "JIT" } else { "VM" };
    eprintln!("[{tier} {:.1} ms]", report.duration_ms);
}
