//! Kernel capability probe: `classdef` support requires a *path* request.
//!
//! RunMat 0.6.2 parses `classdef` and can execute classes when the source is
//! submitted as `SourceInput::Path` (sibling `.m` class files are discovered
//! automatically). `Engine::run` uses `SourceInput::Text`; `Engine::run_at`
//! passes a disk path as the Text source name, which keeps the buffer as the
//! executed text while still triggering sibling class discovery.

use runmat_core::abi::{ExecutionRequest, HostExecutionPolicy, SourceInput, WorkspaceBindingKey};
use runmat_core::RunMatSession;

#[test]
fn classdef_is_supported_for_path_requests() {
    let dir = std::env::temp_dir().join("tl_classdef_probe");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create probe dir");
    std::fs::write(
        dir.join("Vec2.m"),
        "classdef Vec2\n\
         properties\n\
           x\n\
           y\n\
         end\n\
         methods\n\
           function obj = Vec2(x, y)\n\
             obj.x = x;\n\
             obj.y = y;\n\
           end\n\
           function m = magnitude(obj)\n\
             m = sqrt(obj.x * obj.x + obj.y * obj.y);\n\
           end\n\
         end\n\
         end\n",
    )
    .expect("write class");
    std::fs::write(
        dir.join("main.m"),
        "p = Vec2(3, 4); cls = class(p); m = p.magnitude();",
    )
    .expect("write script");

    let mut session = RunMatSession::with_options(false, false).expect("session");
    let request = ExecutionRequest::for_source(
        SourceInput::Path(dir.join("main.m").to_string_lossy().to_string()),
        session.compat_mode(),
        HostExecutionPolicy::default(),
        session.workspace_handle(),
    );
    let response = futures::executor::block_on(session.execute_request(request));
    let outcome = match response.result {
        Ok(outcome) => outcome,
        Err(err) => panic!("path request failed: {err:?}"),
    };

    let mut class_name = None;
    let mut magnitude = None;
    for upsert in &outcome.workspace_delta.upserts {
        let name = match &upsert.key {
            WorkspaceBindingKey::Interactive { name, .. } => Some(name.0.as_str()),
            WorkspaceBindingKey::SourceBinding { binding, .. } => Some(binding.0.as_str()),
            _ => None,
        };
        match name {
            Some("cls") => class_name = Some(upsert.value.clone()),
            Some("m") => magnitude = Some(upsert.value.clone()),
            _ => {}
        }
    }

    assert_eq!(
        class_name,
        Some(runmat_value::Value::String("Vec2".into())),
        "class() must return Vec2"
    );
    assert_eq!(
        magnitude,
        Some(runmat_value::Value::Num(5.0)),
        "p.magnitude() must be 5"
    );
}

#[test]
fn run_at_loads_sibling_classdef_from_editor_buffer() {
    let dir = std::env::temp_dir().join("tl_classdef_run_at");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create probe dir");
    std::fs::write(
        dir.join("Vec2.m"),
        "classdef Vec2\n\
         properties\n\
           x\n\
           y\n\
         end\n\
         methods\n\
           function obj = Vec2(x, y)\n\
             obj.x = x;\n\
             obj.y = y;\n\
           end\n\
           function m = magnitude(obj)\n\
             m = sqrt(obj.x * obj.x + obj.y * obj.y);\n\
           end\n\
         end\n\
         end\n",
    )
    .expect("write class");
    let script = dir.join("main.m");
    // 磁盘内容故意与缓冲区不同，证明执行的是传入的 buffer。
    std::fs::write(&script, "placeholder on disk").expect("write script placeholder");

    let mut engine = tl_kernel::Engine::new().expect("engine");
    let report = engine.run_at(
        &script.to_string_lossy(),
        "p = Vec2(3, 4); cls = class(p); m = p.magnitude();",
    );
    assert!(report.error.is_none(), "error: {:?}", report.error);
    assert!(
        report.variables.iter().any(|v| v.name == "cls"),
        "class() result variable is missing"
    );
    let magnitude = report
        .variables
        .iter()
        .find(|v| v.name == "m")
        .expect("m variable");
    assert!(
        magnitude.preview.contains('5'),
        "preview: {}",
        magnitude.preview
    );
}
