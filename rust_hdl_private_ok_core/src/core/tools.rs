pub fn find_ok_bus_collisions(vlog: &str) {
    let expr = regex::Regex::new(r#"\.ep_addr\(8'h(\w+)\)"#).unwrap();
    let mut addr_list = vec![];
    for capture in expr.captures_iter(vlog) {
        let port = capture.get(1).unwrap().as_str();
        assert!(
            !addr_list.contains(&port.to_string()),
            "Found duplicate port! {}",
            port
        );
        addr_list.push(port.to_owned());
    }
}
