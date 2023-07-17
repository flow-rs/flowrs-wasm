#[cfg(test)]
mod app_state {
    use flow::app_state::AppState;

    #[test]
    fn should_deserialize_empty_state() {
        let json_str = r#"
    {
        "nodes": [],
        "edges": []
    }
    "#;

        let json_data: AppState = serde_json::from_str(json_str).unwrap();
        assert!(json_data.nodes.len() == 0);
    }

    #[test]
    fn should_deserialize_non_empty_state() {
        let json_str = r#"
        {
            "nodes": [
                {
                    "name": "lhs",
                    "kind": "nodes.basic",
                    "props": {"value": {"I32": 12}}
                },
                {
                    "name": "rhs",
                    "kind": "nodes.basic",
                    "props": {"value": {"I32": 30}}
                },
                {
                    "name": "add",
                    "kind": "nodes.arithmetics.add",
                    "props": {"none": "Undefined"}
                }
            ],
            "edges": [
                {
                    "input": "lhs",
                    "output": "add",
                    "index": 0
                },
                {
                    "input": "rhs",
                    "output": "add",
                    "index": 1
                }
            ]
        }
        "#;

        let json_data: AppState = serde_json::from_str(json_str).unwrap();
        assert!(json_data.nodes.len() == 3);
        assert!(json_data.nodes.get("lhs").unwrap().output().take().len() == 1);
        assert!(json_data.nodes.get("rhs").unwrap().output().take().len() == 1);
        assert!(json_data.nodes.get("add").unwrap().output().take().len() == 0);
    }
}