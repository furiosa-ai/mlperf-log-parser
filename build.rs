fn main() {
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .generate_in_source_tree() // 소스 트리에 생성
        .always_use_colors() // 컬러 출력 사용
        .log_debug() // 자세한 로그 출력
        .emit_comments(true) // 라인 번호 정보 포함
        .process_file("src/log_summary/grammar.lalrpop")
        .expect("Failed to process src/log_summary/grammar.lalrpop file");

    // 파일이 변경되면 다시 빌드하도록 설정
    println!("cargo:rerun-if-changed=src/log_summary/grammar.lalrpop");
}
