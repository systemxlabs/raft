fn main() {
    tonic_build::configure()
        // 给proto生成的rust类型加上派生宏
        .type_attribute(
            "LogEntry",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute("Server", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&["proto/raft.proto"], &["proto"])
        .unwrap();

    tonic_build::configure()
        .compile(&["proto/helloworld.proto"], &["proto"])
        .unwrap();
}
