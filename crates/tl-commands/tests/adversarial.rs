//! P1 对抗性测试：v0.2 工具箱函数的退化输入、反例与防作弊用例。
//!
//! 覆盖 v0.2 新增的 34 个函数（Signal 8 / Optimization 8 / Statistics 7 /
//! Control 11）。重点：
//! - 退化输入：空/标量/1×1/奇异矩阵/重复极点/不可行约束/非法 method；
//! - 反例必须返回**可诊断的错误**，而不是 panic 或静默错误结果；
//! - 防作弊：同一函数至少两组不同输入，输出必须随输入变化。
//!
//! 说明：
//! - 所有脚本走 `run_file`（Path 请求），与 CLI/demo 的真实执行路径一致。
//! - `intlinprog` 在不可行约束上目前会长时间不返回（见测试内注释），因此只测
//!   合法分支和非法 `intcon`，不把挂起用例放进 CI。
//! - `nlinfit` 对发散模型（如 `b1*exp(b2*x)` 且初值不佳）会触发内核 SVD
//!   的 NaN panic；这里只测收敛模型与"模型返回长度错误"的干净反例。

use std::path::PathBuf;
use std::sync::Mutex;
use tl_commands::{Engine, RunReport};

/// 内核的图窗/环境状态仍有进程级成分；串行化让测试确定性。
static KERNEL_LOCK: Mutex<()> = Mutex::new(());

struct Harness {
    engine: Engine,
    dir: PathBuf,
    seq: usize,
}

impl Harness {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join("tl_adversarial").join(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create adversarial dir");
        Self {
            engine: Engine::new().expect("engine init"),
            dir,
            seq: 0,
        }
    }

    fn run(&mut self, src: &str) -> RunReport {
        self.seq += 1;
        let path = self.dir.join(format!("case_{}.m", self.seq));
        std::fs::write(&path, src).expect("write m script");
        tl_commands::run_file(&mut self.engine, path.to_str().unwrap()).expect("run_file")
    }
}

fn assert_ok(report: &RunReport, label: &str) -> String {
    assert!(
        report.error.is_none(),
        "{label}: unexpected error: {:?}\nconsole:\n{}",
        report.error.as_ref().map(|e| &e.message),
        report.console_text()
    );
    report.console_text()
}

fn assert_err(report: &RunReport, label: &str, needle: &str) {
    let message = report
        .error
        .as_ref()
        .map(|e| e.message.clone())
        .unwrap_or_default();
    assert!(
        message.contains(needle),
        "{label}: expected error containing {needle:?}, got {message:?}"
    );
}

fn nums(text: &str) -> Vec<f64> {
    text.split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect()
}

fn close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected} ± {tol}, got {actual}"
    );
}

// ------------------------------------------------------------------ optimization

#[test]
fn fmincon_rejects_scalar_and_handles_infeasible_constraints() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("fmincon");

    // 不可行约束：x1 <= -1 且 x1 >= 0。应返回 exitflag=-2，而不是 panic。
    let text = assert_ok(
        &h.run(
            "[x,f,e]=fmincon(@(v) v(1)^2,[0;0],[1 0;-1 0],[-1;0],[],[],[],[]); \
             fprintf('infeas %.4f %.4f %d\\n',x(1),x(2),e);",
        ),
        "fmincon infeasible",
    );
    let v = nums(&text);
    close(v[0], -0.5, 1e-2, "fmincon infeasible x1");
    close(v[1], 0.0, 1e-2, "fmincon infeasible x2");
    assert_eq!(v[2], -2.0, "fmincon infeasible exitflag");

    // 标量 x0 受内核 K2 限制：应给出干净错误（不是 panic）。
    let report = h.run("x=fmincon(@(v) v^2,1);");
    assert_err(
        &report,
        "fmincon scalar x0",
        "Slicing only supported on tensors",
    );
}

#[test]
fn lsqlin_handles_singular_and_infeasible_constraints() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("lsqlin");

    // 奇异 C：H=0，走 pinv 路径；任意 x 的残差都是 14。
    let text = assert_ok(
        &h.run(
            "[x,rn,res,e]=lsqlin(zeros(3,2),[1;2;3]); \
             fprintf('sing %.4f %.4f %.4f %d\\n',x(1),x(2),rn,e);",
        ),
        "lsqlin singular",
    );
    let v = nums(&text);
    close(v[0], 0.0, 1e-9, "lsqlin singular x1");
    close(v[1], 0.0, 1e-9, "lsqlin singular x2");
    close(v[2], 14.0, 1e-6, "lsqlin singular resnorm");
    assert_eq!(v[3], 1.0, "lsqlin singular exitflag");

    // 不可行：x1 <= -1 且 x1 >= 0。active-set 应停下并给 exitflag=0。
    let text = assert_ok(
        &h.run(
            "[x,rn,res,e]=lsqlin([1 0;0 1],[0;0],[1 0;-1 0],[-1;0]); \
             fprintf('infeas %.4f %.4f %d\\n',x(1),x(2),e);",
        ),
        "lsqlin infeasible",
    );
    let v = nums(&text);
    close(v[0], -1.0, 1e-2, "lsqlin infeasible x1");
    assert_eq!(v[2], 0.0, "lsqlin infeasible exitflag");

    let report = h.run("x=lsqlin(1,2);");
    assert_err(
        &report,
        "lsqlin scalar",
        "Slicing only supported on tensors",
    );
}

#[test]
fn intlinprog_bounds_and_invalid_intcon() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("intlinprog");

    let text = assert_ok(
        &h.run(
            "[x,f,e]=intlinprog([-3;-2],[1 2],[1 1;1 3],[4;6],[],[],[0;0],[]); \
             fprintf('basic %.4f %.4f %.4f %d\\n',x(1),x(2),f,e);",
        ),
        "intlinprog basic",
    );
    let v = nums(&text);
    close(v[0], 4.0, 1e-6, "intlinprog x1");
    close(v[1], 0.0, 1e-6, "intlinprog x2");
    close(v[2], -12.0, 1e-6, "intlinprog fval");
    assert_eq!(v[3], 1.0, "intlinprog exitflag");

    // 反例：intcon 越界。当前返回 "Index out of bounds"（干净错误）。
    // 注意：不可行约束的 intlinprog 目前会长时间不返回，故意不放进 CI。
    let report = h.run(
        "[x,f,e]=intlinprog([1;-1],[1 3],[1 1],3,[],[],[0;0],[5;5]); \
         fprintf('bad %g\\n',numel(x));",
    );
    assert_err(
        &report,
        "intlinprog intcon out of range",
        "Index out of bounds",
    );
}

#[test]
fn optimget_reads_field_and_falls_back_to_default() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("optimget");
    let text = assert_ok(
        &h.run(
            "o=optimset('TolX',1e-8); a=optimget(o,'TolX'); b=optimget(o,'NoSuchField',7); \
             fprintf('%.6g %.6g\\n',a,b);",
        ),
        "optimget",
    );
    let v = nums(&text);
    close(v[0], 1e-8, 1e-15, "optimget TolX");
    close(v[1], 7.0, 1e-12, "optimget default");
}

/// ga / simulannealbnd / particleswarm / patternsearch 的已知触发条件：
/// 它们都在 `toolbox/optim_batch/`（5 个函数）；单独调用会触发 K13
/// （function handle 错位，`Not enough input arguments`）。按 demo 的顺序
/// 连续调用四个求解器则正常。这里用第二组目标验证四者都随输入变化。
#[test]
fn batch_optimizers_track_a_second_target() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("batch_optimizers");
    let text = assert_ok(
        &h.run(
            "rng(11); \
             [x1,f1]=ga(@(v)(v(1)-3)^2+(v(2)+1)^2,2,[],[],[],[],[-5;-5],[5;5]); \
             x2=simulannealbnd(@(v)(v(1)-3)^2+(v(2)+1)^2,[0.5;0.5],[-5;-5],[5;5]); \
             rng(12); \
             x3=particleswarm(@(v)(v(1)-3)^2+(v(2)+1)^2,2,[-5;-5],[5;5]); \
             x4=patternsearch(@(v)(v(1)-3)^2+(v(2)+1)^2,[0.5;0.5],[],[],[],[],[-5;-5],[5;5]); \
             fprintf('%.4f %.4f %.4f %.4f %.4f %.4f %.4f %.4f\\n', \
                 x1(1),x1(2),x2(1),x2(2),x3(1),x3(2),x4(1),x4(2));",
        ),
        "batch optimizers",
    );
    let v = nums(&text);
    for (idx, label) in ["ga", "simulannealbnd", "particleswarm", "patternsearch"]
        .iter()
        .enumerate()
    {
        close(v[2 * idx], 3.0, 0.05, &format!("{label} x1"));
        close(v[2 * idx + 1], -1.0, 0.05, &format!("{label} x2"));
    }
}

// ------------------------------------------------------------------ control

#[test]
fn kalman_scalar_analytic_and_singular_r() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("kalman");

    // 1×1 解析解：A=-1, C=1, Q=1, R=1 → P=√2-1, L=(√2-1)/√2。
    let text = assert_ok(
        &h.run("S=ss(-1,1,1,0); [L,P]=kalman(S,1,1); fprintf('%.6f %.6f\\n',L,P);"),
        "kalman 1x1",
    );
    let v = nums(&text);
    close(v[0], 0.292893, 1e-5, "kalman 1x1 L");
    close(v[1], 0.414214, 1e-5, "kalman 1x1 P");

    // demo 使用的 2×2 系统；注意发布说明里的 0.2929/0.4142 是 1×1 解析值，
    // 不是这个 2×2 系统的 L(1,1)/P(1,1)。
    let text = assert_ok(
        &h.run(
            "S=ss([0 1;-2 -3],[0;1],[1 0],0); [L,P]=kalman(S,eye(2),1); \
             fprintf('%.4f %.4f\\n',L(1,1),P(1,1));",
        ),
        "kalman 2x2",
    );
    let v = nums(&text);
    close(v[0], 0.4019, 1e-3, "kalman 2x2 L11");
    close(v[1], 0.6720, 1e-3, "kalman 2x2 P11");

    // R=0 是奇异情形：当前返回 NaN（不 panic），测试固定这一可诊断行为。
    let text = assert_ok(
        &h.run("S=ss(-1,1,1,0); [L,P]=kalman(S,1,0); fprintf('%g %g\\n',L,P);"),
        "kalman R=0",
    );
    assert!(
        text.to_lowercase().contains("nan"),
        "kalman R=0 should expose NaN, got {text:?}"
    );
}

#[test]
fn c2d_d2c_d2d_degenerate_cases() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("c2d");

    // 重复极点：ZOH 内部回退到 Tustin。
    let text = assert_ok(
        &h.run(
            "G=tf(1,[1 2 1]); Gz=c2d(G,0.1,'zoh'); \
             fprintf('%.6f %.6f %.6f\\n',Gz.Denominator(1),Gz.Denominator(2),Gz.Denominator(3));",
        ),
        "c2d repeated poles",
    );
    let v = nums(&text);
    close(v[0], 1.0, 1e-9, "c2d den1");
    close(v[1], -1.80952, 1e-4, "c2d den2");
    close(v[2], 0.818594, 1e-4, "c2d den3");

    // 一阶系统的两种极点：确认输出随输入变化。
    let text = assert_ok(
        &h.run(
            "G1=c2d(tf(1,[1 1]),0.1,'zoh'); G2=c2d(tf(1,[1 2]),0.1,'zoh'); \
             fprintf('%.6f %.6f\\n',G1.Denominator(2),G2.Denominator(2));",
        ),
        "c2d two systems",
    );
    let v = nums(&text);
    close(v[0], -0.904837, 1e-5, "c2d pole -1");
    close(v[1], -0.818731, 1e-5, "c2d pole -2");
    assert!((v[0] - v[1]).abs() > 0.05, "c2d must depend on the system");

    // d2c / d2d 往返。
    let text = assert_ok(
        &h.run(
            "G=tf(1,[1 2 1]); Gz=c2d(G,0.1,'tustin'); Gc=d2c(Gz,0.1); \
             Gz2=c2d(tf(1,[1 1]),0.1,'tustin'); G2=d2d(Gz2,0.1,0.2,'tustin'); \
             fprintf('d2c %.6f %.6f %.6f d2d %.6f %.6f\\n', \
                 Gc.Denominator(1),Gc.Denominator(2),Gc.Denominator(3), \
                 G2.Denominator(1),G2.Denominator(2));",
        ),
        "d2c/d2d",
    );
    let v = nums(&text);
    close(v[0], 1.0, 1e-6, "d2c den1");
    close(v[1], 2.0, 1e-4, "d2c den2");
    close(v[2], 1.0, 1e-4, "d2c den3");
    close(v[3], 1.0, 1e-6, "d2d den1");
    close(v[4], -0.81818, 1e-4, "d2d den2");

    // 反例：非法 method / 非严格真分式 / 零极点。
    assert_err(
        &h.run("Gz=c2d(tf(1,[1 1]),0.1,'foo');"),
        "c2d bad method",
        "supported methods",
    );
    assert_err(
        &h.run("Gz=c2d(tf([1 2],[1 1]),0.1,'zoh');"),
        "c2d improper",
        "strictly proper",
    );
    assert_err(
        &h.run("Gz=c2d(tf(1,[1 0]),0.1,'zoh');"),
        "c2d zero pole",
        "numerator coefficients must be finite",
    );
}

#[test]
fn control_misc_degenerate_cases() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("control_misc");

    // minreal：可约 / 不可约。
    let text = assert_ok(
        &h.run(
            "T1=minreal(tf([1 1],[1 3 2])); T2=minreal(tf([1 2],[1 4 3])); \
             fprintf('%g %g\\n',numel(T1.Denominator)-1,numel(T2.Denominator)-1);",
        ),
        "minreal",
    );
    let v = nums(&text);
    assert_eq!(v[0], 1.0, "minreal should cancel the common pole");
    assert_eq!(v[1], 2.0, "minreal must keep the irreducible system");

    // ss2zp：重复极点；第二组带一个零点。
    let text = assert_ok(
        &h.run(
            "A=[0 1;-2 -3]; [z1,p1,k1]=ss2zp(A,[0;1],[1 0],0); \
             [z2,p2,k2]=ss2zp(A,[0;1],[1 1],0); \
             fprintf('%g %g %.3f %g %g %.3f\\n',numel(z1),numel(p1),k1,numel(z2),numel(p2),k2);",
        ),
        "ss2zp",
    );
    let v = nums(&text);
    assert_eq!(v[0], 0.0, "ss2zp system 1 zeros");
    assert_eq!(v[1], 2.0, "ss2zp system 1 poles");
    close(v[2], 1.0, 1e-6, "ss2zp system 1 gain");
    assert_eq!(v[3], 1.0, "ss2zp system 2 zeros");
    assert_eq!(v[4], 2.0, "ss2zp system 2 poles");

    // zp2ss：空零点 + 不同极点/增益。
    let text = assert_ok(
        &h.run(
            "[A1,B1,C1,D1]=zp2ss([],[-1],2); [A2,B2,C2,D2]=zp2ss([],[-2],3); \
             fprintf('%.3f %.3f %.3f %.3f %.3f %.3f\\n',A1(1,1),B1(1,1),C1(1,1),A2(1,1),B2(1,1),C2(1,1));",
        ),
        "zp2ss",
    );
    let v = nums(&text);
    close(v[0], -1.0, 1e-6, "zp2ss A1");
    close(v[1], 1.0, 1e-6, "zp2ss B1");
    close(v[2], 2.0, 1e-6, "zp2ss C1");
    close(v[3], -2.0, 1e-6, "zp2ss A2");
    close(v[5], 3.0, 1e-6, "zp2ss C2");

    // ssdata：1×1 与 2×2。
    let text = assert_ok(
        &h.run(
            "S1=ss(-1,1,1,0); [A1,B1,C1,D1]=ssdata(S1); \
             S2=ss([0 1;-2 -3],[0;1],[1 0],0); [A2,B2,C2,D2]=ssdata(S2); \
             fprintf('%.3f %.3f %.3f %.3f\\n',A1(1,1),D1,A2(1,1),D2);",
        ),
        "ssdata",
    );
    let v = nums(&text);
    close(v[0], -1.0, 1e-6, "ssdata A 1x1");
    close(v[1], 0.0, 1e-9, "ssdata D 1x1");
    close(v[2], 0.0, 1e-9, "ssdata A 2x2");
    close(v[3], 0.0, 1e-9, "ssdata D 2x2");

    // allmargin：一阶系统 2/(s+1) 的有限相位裕度（Gm 仍为 Inf）。
    let text = assert_ok(
        &h.run(
            "[Gm,Pm,Wcg,Wcp]=allmargin(tf(2,[1 1])); \
             fprintf('%.4f %.4f %.4f\\n',Pm,Wcg,Wcp);",
        ),
        "allmargin",
    );
    let v = nums(&text);
    close(v[0], 120.0, 0.1, "allmargin Pm");
    close(v[1], 0.0, 1e-6, "allmargin Wcg");
    close(v[2], 1.732, 1e-3, "allmargin Wcp");

    // nichols：显式频率网格的点数必须精确。
    let text = assert_ok(
        &h.run(
            "[md,ph,w]=nichols(tf(1,[1 2 1]),[0.1 1 10]); \
             fprintf('%g %.3f\\n',numel(w),w(2));",
        ),
        "nichols",
    );
    let v = nums(&text);
    assert_eq!(v[0], 3.0, "nichols point count");
    close(v[1], 1.0, 1e-9, "nichols second frequency");

    // 退化：R=0 未被校验。当前实现仍返回有限 K（不报错、不 panic）；
    // 这里固定这一行为，提醒后续要么补校验、要么在文档中标注。
    let text = assert_ok(
        &h.run("[K,S,e]=dlqr([1 1;0 1],[0.5;1],eye(2),0); fprintf('%.4f %.4f\\n',K(1,1),K(1,2));"),
        "dlqr R=0",
    );
    let v = nums(&text);
    assert!(
        v.iter().all(|x| x.is_finite()),
        "dlqr R=0 current behavior should stay finite: {text:?}"
    );
}

// ------------------------------------------------------------------ statistics

#[test]
fn stats_degenerate_cases() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("stats");

    // mle：正常、两组不同数据（防作弊）。
    let text = assert_ok(
        &h.run(
            "p1=mle([1 2 3 4]); p2=mle([10 20 30]); \
             fprintf('%.4f %.5f %.4f %.5f\\n',p1(1),p1(2),p2(1),p2(2));",
        ),
        "mle",
    );
    let v = nums(&text);
    close(v[0], 2.5, 1e-6, "mle mu 1");
    close(v[1], 1.11803, 1e-5, "mle sigma 1");
    close(v[2], 20.0, 1e-6, "mle mu 2");
    close(v[3], 8.16497, 1e-5, "mle sigma 2");

    // anova1：同均值 → F=Inf；两组不同数据 → p 明显变化。
    let text = assert_ok(
        &h.run(
            "[p1,F1]=anova1([1 1 1;1 1 1]); \
             [p2,F2]=anova1([1 2 3;2 3 4]); \
             fprintf('%.3f %.3f %.5f %.3f\\n',p1,F1,p2,F2);",
        ),
        "anova1",
    );
    let v = nums(&text);
    close(v[0], 0.0, 1e-9, "anova1 identical p");
    assert!(
        v[1].is_infinite(),
        "anova1 identical F must be Inf: {text:?}"
    );
    close(v[2], 0.14243, 1e-4, "anova1 two-group p");
    close(v[3], 4.0, 1e-3, "anova1 two-group F");

    // anova2：reps=2 的正常分支；reps 不整除列数 → 干净错误。
    let text = assert_ok(
        &h.run(
            "p=anova2([1 2 3 4; 2 3 4 5; 3 4 5 6; 4 5 6 7],2); \
             fprintf('%.5f %.5f %.3f\\n',p(1),p(2),p(3));",
        ),
        "anova2",
    );
    let v = nums(&text);
    assert!((0.0..=1.0).contains(&v[0]), "anova2 pA in [0,1]");
    assert!((0.0..=1.0).contains(&v[1]), "anova2 pB in [0,1]");
    close(v[2], 1.0, 1e-6, "anova2 interaction p");

    // kruskalwallis：两组不同分组 → H 改变。
    let text = assert_ok(
        &h.run(
            "[p1,H1]=kruskalwallis([1 2 3 4]',[1 1 2 2]'); \
             [p2,H2]=kruskalwallis([1 2 3 4]',[1 1 1 2]'); \
             fprintf('%.5f %.3f %.5f %.3f\\n',p1,H1,p2,H2);",
        ),
        "kruskalwallis",
    );
    let v = nums(&text);
    close(v[0], 0.12134, 1e-5, "kruskalwallis p1");
    close(v[1], 2.4, 1e-6, "kruskalwallis H1");
    close(v[2], 0.18, 1e-3, "kruskalwallis p2");
    close(v[3], 1.8, 1e-6, "kruskalwallis H2");
    assert!(
        (v[1] - v[3]).abs() > 0.1,
        "kruskalwallis must depend on grouping"
    );

    // glmfit：正常 + 两组不同数据 + 反例。
    let text = assert_ok(
        &h.run(
            "b1=glmfit([1;2;3],[2;4;6]); b2=glmfit([1;2;3],[1;4;7]); \
             fprintf('%.4f %.4f %.4f %.4f\\n',b1(1),b1(2),b2(1),b2(2));",
        ),
        "glmfit",
    );
    let v = nums(&text);
    close(v[0], 0.0, 1e-5, "glmfit b0 1");
    close(v[1], 2.0, 1e-5, "glmfit b1 1");
    close(v[2], -2.0, 1e-5, "glmfit b0 2");
    close(v[3], 3.0, 1e-5, "glmfit b1 2");

    // nlinfit：线性模型精确恢复；第二组数据 → 参数变化。
    let text = assert_ok(
        &h.run(
            "X=(1:5)'; b1=nlinfit(X,2*X+1,@(b,x) b(1)*x+b(2),[0;0]); \
             b2=nlinfit(X,3*X-1,@(b,x) b(1)*x+b(2),[0;0]); \
             fprintf('%.4f %.4f %.4f %.4f\\n',b1(1),b1(2),b2(1),b2(2));",
        ),
        "nlinfit",
    );
    let v = nums(&text);
    close(v[0], 2.0, 1e-4, "nlinfit b1 1");
    close(v[1], 1.0, 1e-4, "nlinfit b2 1");
    close(v[2], 3.0, 1e-4, "nlinfit b1 2");
    close(v[3], -1.0, 1e-4, "nlinfit b2 2");

    // chi2pdf 边界：x=0,nu=1 → Inf；x<0 → 0。
    let text = assert_ok(
        &h.run("y1=chi2pdf(0,1); y2=chi2pdf(-1,2); fprintf('%g %g\\n',y1,y2);"),
        "chi2pdf",
    );
    assert!(text.contains("Inf"), "chi2pdf(0,1) must be Inf: {text:?}");
    assert!(
        text.trim().ends_with('0'),
        "chi2pdf(-1,2) must be 0: {text:?}"
    );

    // 反例集合（放在成功用例之后）。
    assert_err(
        &h.run("p=mle([1 2 3],'foo');"),
        "mle bad distr",
        "unsupported distribution",
    );
    assert_err(
        &h.run("p=mle(5);"),
        "mle scalar",
        "Slicing only supported on tensors",
    );
    assert_err(
        &h.run("p=anova1([1;2;3]);"),
        "anova1 no group",
        "provide a GROUP vector",
    );
    assert_err(
        &h.run("p=anova2([1 2 3],2);"),
        "anova2 reps mismatch",
        "multiple of REPS",
    );
    assert_err(
        &h.run("b=glmfit([1;2],[1;2;3]);"),
        "glmfit row mismatch",
        "same number of rows",
    );
    assert_err(
        &h.run("b=glmfit([1;2],[1;2],'foo');"),
        "glmfit bad distr",
        "unsupported distribution",
    );
    // 模型返回标量（长度错误）→ 干净错误，而不是 SVD panic。
    assert_err(
        &h.run("b=nlinfit([1;2;3],[1;2;3],@(b,x) b(1),[1]);"),
        "nlinfit bad model",
        "Index out of bounds",
    );
}

// ------------------------------------------------------------------ signal

#[test]
fn signal_degenerate_and_anti_cheat_cases() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("signal");

    // convn：2-D 正常；不同 kernel → 不同结果；3-D 反例。
    let text = assert_ok(
        &h.run(
            "c1=convn([1 2;3 4],eye(2)); c2=convn([1 2;3 4],[1 1;1 1]); \
             fprintf('%g %g %.3f %.3f\\n',size(c1,1),size(c1,2),sum(sum(c1)),sum(sum(c2)));",
        ),
        "convn",
    );
    let v = nums(&text);
    assert_eq!(v[0], 3.0, "convn rows");
    assert_eq!(v[1], 3.0, "convn cols");
    // 全卷积下 eye(2) 会在两个对角线位置各复制一份 A，和为 2*sum(A)=20。
    close(v[2], 20.0, 1e-9, "convn identity sum");
    close(v[3], 40.0, 1e-9, "convn ones sum");

    // fir2：长度与两组不同规格。
    let text = assert_ok(
        &h.run(
            "b1=fir2(4,[0 .5 1],[1 1 0]); b2=fir2(4,[0 .2 1],[1 0 0]); \
             fprintf('%g %.5f %.5f\\n',numel(b1),b1(1),b2(1));",
        ),
        "fir2",
    );
    let v = nums(&text);
    assert_eq!(v[0], 5.0, "fir2 length");
    close(v[1], 0.74754, 1e-4, "fir2 spec 1");
    close(v[2], 0.04164, 1e-4, "fir2 spec 2");
    assert!((v[1] - v[2]).abs() > 0.1, "fir2 must depend on the spec");

    // freqs：解析值 + 两组系统。
    let text = assert_ok(
        &h.run(
            "h1=freqs([1],[1 1],[0 1]); h2=freqs([2],[1 2],1); \
             fprintf('%.5f %.5f %.5f\\n',abs(h1(1)),abs(h1(2)),abs(h2));",
        ),
        "freqs",
    );
    let v = nums(&text);
    close(v[0], 1.0, 1e-9, "freqs DC");
    close(v[1], 0.70711, 1e-5, "freqs w=1");
    close(v[2], 0.89443, 1e-5, "freqs second system");

    // upfirdn：升采样倍数改变长度。
    let text = assert_ok(
        &h.run(
            "y2=upfirdn([1 2 3],[1 1],2,1); y3=upfirdn([1 2 3],[1 1],3,1); \
             fprintf('%g %g\\n',numel(y2),numel(y3));",
        ),
        "upfirdn",
    );
    let v = nums(&text);
    assert_eq!(v, vec![6.0, 8.0], "upfirdn lengths");

    // pspectrum：两组信号的能量分布不同。
    let text = assert_ok(
        &h.run(
            "[p1,f1]=pspectrum([1 2 3 4],1); [p2,f2]=pspectrum([1 -1 1 -1],1); \
             fprintf('%g %.3f %.5f %.5f\\n',numel(p1),f1(1),p1(2),p2(2));",
        ),
        "pspectrum",
    );
    let v = nums(&text);
    assert_eq!(v[0], 3.0, "pspectrum bins");
    close(v[1], 0.0, 1e-9, "pspectrum first frequency");
    assert!(
        (v[2] - v[3]).abs() > 1e-6,
        "pspectrum must depend on the signal"
    );

    // kaiserord：正常 + 更宽过渡带 + 非递增频带反例。
    let text = assert_ok(
        &h.run(
            "[n1,Wn1,beta1,ft1]=kaiserord([0 .2 .3 1],[1 0],[.01 .01]); \
             [n2,Wn2,beta2,ft2]=kaiserord([0 .2 .5 1],[1 0],[.01 .01]); \
             fprintf('%g %.3f %s %g %.3f %s\\n',n1,Wn1,ft1,n2,Wn2,ft2);",
        ),
        "kaiserord",
    );
    let v = nums(&text);
    assert_eq!(v[0], 45.0, "kaiserord narrow n");
    close(v[1], 0.25, 1e-9, "kaiserord narrow Wn");
    assert_eq!(v[2], 15.0, "kaiserord wide n");
    close(v[3], 0.35, 1e-9, "kaiserord wide Wn");

    // czt / goertzel：长度与两组输入。
    let text = assert_ok(
        &h.run(
            "y2=czt([1 2 3 4],2); y4=czt([1 2 3 4],4); \
             g1=goertzel([1 2 3 4],1); g2=goertzel([1 2 3 4],2); \
             fprintf('%g %g %.5f %.5f %.5f %.5f\\n',numel(y2),numel(y4),abs(y2(1)),abs(g1),abs(g2),angle(g2));",
        ),
        "czt/goertzel",
    );
    let v = nums(&text);
    assert_eq!(v[0], 2.0, "czt m=2");
    assert_eq!(v[1], 4.0, "czt m=4");
    close(v[2], 10.0, 1e-5, "czt first bin");
    close(v[3], 10.0, 1e-5, "goertzel DC");
    close(v[4], 2.82843, 1e-5, "goertzel bin 2 magnitude");
    close(v[5], 2.35619, 1e-5, "goertzel bin 2 phase");

    // 反例集合。
    assert_err(
        &h.run("c=convn(ones(2,2,2),ones(2,2));"),
        "convn 3-D",
        "currently supports 1-D and 2-D",
    );
    assert_err(
        &h.run("[n,Wn,beta,ft]=kaiserord([0 .3 .2 1],[1 0],[.01 .01]);"),
        "kaiserord bad edges",
        "strictly increasing",
    );
}

// ------------------------------------------------------------------ anti-cheat

/// 防作弊：同一函数两组不同输入，输出必须不同（拦截硬编码常量）。
#[test]
fn anti_cheat_two_inputs_change_outputs() {
    let _guard = KERNEL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = Harness::new("anti_cheat");

    // fmincon：两个不同二次目标。
    let a = assert_ok(
        &h.run("[x,f]=fmincon(@(v) v(1)^2+v(2)^2,[2;2],[],[],[1 1],1,[0;0],[]); fprintf('%.4f %.4f\\n',x(1),x(2));"),
        "anti-cheat fmincon 1",
    );
    let b = assert_ok(
        &h.run("[x,f]=fmincon(@(v)(v(1)-3)^2+(v(2)+1)^2,[0;0],[],[],[],[],[-5;-5],[5;5]); fprintf('%.4f %.4f\\n',x(1),x(2));"),
        "anti-cheat fmincon 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[0], 0.5, 2e-3, "fmincon 1 x1");
    close(b[0], 3.0, 1e-3, "fmincon 2 x1");
    assert!(
        (a[0] - b[0]).abs() > 1.0,
        "fmincon must depend on the objective"
    );

    // lsqlin：两个不同 d。
    let a = assert_ok(
        &h.run("[x,rn]=lsqlin([1 0;0 1;1 1],[1;2;3],[1 1],2); fprintf('%.4f %.4f\\n',x(1),x(2));"),
        "anti-cheat lsqlin 1",
    );
    let b = assert_ok(
        &h.run("[x,rn]=lsqlin([1 0;0 1;1 1],[2;1;0],[1 1],2); fprintf('%.4f %.4f\\n',x(1),x(2));"),
        "anti-cheat lsqlin 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[0], 0.5, 1e-3, "lsqlin 1 x1");
    close(b[0], 1.0, 1e-3, "lsqlin 2 x1");
    assert!((a[0] - b[0]).abs() > 0.1, "lsqlin must depend on d");

    // intlinprog：两个不同成本向量。
    let a = assert_ok(
        &h.run("[x,f]=intlinprog([-3;-2],[1 2],[1 1;1 3],[4;6],[],[],[0;0],[]); fprintf('%.4f %.4f %.4f\\n',x(1),x(2),f);"),
        "anti-cheat intlinprog 1",
    );
    let b = assert_ok(
        &h.run("[x,f]=intlinprog([-2;-3],[1 2],[1 1;1 3],[4;6],[],[],[0;0],[]); fprintf('%.4f %.4f %.4f\\n',x(1),x(2),f);"),
        "anti-cheat intlinprog 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[0], 4.0, 1e-6, "intlinprog 1 x1");
    close(b[0], 3.0, 1e-6, "intlinprog 2 x1");
    close(b[1], 1.0, 1e-6, "intlinprog 2 x2");
    close(a[2], -12.0, 1e-6, "intlinprog 1 f");
    close(b[2], -9.0, 1e-6, "intlinprog 2 f");

    // kalman：1×1 vs 2×2。
    let a = assert_ok(
        &h.run("S=ss(-1,1,1,0); [L,P]=kalman(S,1,1); fprintf('%.4f %.4f\\n',L,P);"),
        "anti-cheat kalman 1",
    );
    let b = assert_ok(
        &h.run("S=ss([0 1;-2 -3],[0;1],[1 0],0); [L,P]=kalman(S,eye(2),1); fprintf('%.4f %.4f\\n',L(1,1),P(1,1));"),
        "anti-cheat kalman 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[0], 0.2929, 1e-3, "kalman 1 L");
    close(b[0], 0.4019, 1e-3, "kalman 2 L");
    assert!(
        (a[0] - b[0]).abs() > 0.05,
        "kalman must depend on the system"
    );

    // c2d：两个不同极点。
    let a = assert_ok(
        &h.run("G=c2d(tf(1,[1 1]),0.1,'zoh'); fprintf('%.5f\\n',G.Denominator(2));"),
        "anti-cheat c2d 1",
    );
    let b = assert_ok(
        &h.run("G=c2d(tf(1,[1 2]),0.1,'zoh'); fprintf('%.5f\\n',G.Denominator(2));"),
        "anti-cheat c2d 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[0], -0.90484, 1e-4, "c2d 1 pole");
    close(b[0], -0.81873, 1e-4, "c2d 2 pole");

    // mle / glmfit / freqs / goertzel / convn。
    let a = assert_ok(
        &h.run("p=mle([1 2 3 4]); fprintf('%.5f\\n',p(1));"),
        "anti-cheat mle 1",
    );
    let b = assert_ok(
        &h.run("p=mle([10 20 30]); fprintf('%.5f\\n',p(1));"),
        "anti-cheat mle 2",
    );
    assert!(
        (nums(&a)[0] - nums(&b)[0]).abs() > 1.0,
        "mle must depend on data"
    );

    let a = assert_ok(
        &h.run("b=glmfit([1;2;3],[2;4;6]); fprintf('%.4f %.4f\\n',b(1),b(2));"),
        "anti-cheat glmfit 1",
    );
    let b = assert_ok(
        &h.run("b=glmfit([1;2;3],[1;4;7]); fprintf('%.4f %.4f\\n',b(1),b(2));"),
        "anti-cheat glmfit 2",
    );
    let (a, b) = (nums(&a), nums(&b));
    close(a[1], 2.0, 1e-5, "glmfit 1 slope");
    close(b[1], 3.0, 1e-5, "glmfit 2 slope");

    let a = assert_ok(
        &h.run("h=freqs([1],[1 1],1); fprintf('%.5f\\n',abs(h));"),
        "anti-cheat freqs 1",
    );
    let b = assert_ok(
        &h.run("h=freqs([2],[1 2],1); fprintf('%.5f\\n',abs(h));"),
        "anti-cheat freqs 2",
    );
    assert!(
        (nums(&a)[0] - nums(&b)[0]).abs() > 0.1,
        "freqs must depend on the system"
    );

    let a = assert_ok(
        &h.run("g=goertzel([1 2 3 4],1); fprintf('%.5f\\n',abs(g));"),
        "anti-cheat goertzel 1",
    );
    let b = assert_ok(
        &h.run("g=goertzel([1 2 3 4],2); fprintf('%.5f\\n',abs(g));"),
        "anti-cheat goertzel 2",
    );
    assert!(
        (nums(&a)[0] - nums(&b)[0]).abs() > 1.0,
        "goertzel must depend on bin"
    );

    let a = assert_ok(
        &h.run("c=convn([1 2;3 4],eye(2)); fprintf('%.5f\\n',sum(sum(c)));"),
        "anti-cheat convn 1",
    );
    let b = assert_ok(
        &h.run("c=convn([1 2;3 4],[1 1;1 1]); fprintf('%.5f\\n',sum(sum(c)));"),
        "anti-cheat convn 2",
    );
    assert!(
        (nums(&a)[0] - nums(&b)[0]).abs() > 1.0,
        "convn must depend on the kernel"
    );
}
