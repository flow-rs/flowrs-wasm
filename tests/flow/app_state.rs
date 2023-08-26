#[cfg(test)]
mod app_state {
    use flowrs_wasm::app_state::AppState;

    #[test]
    fn should_deserialize_empty_state() {
        let json_str = r#"
    {
        "threads": 1,
        "duration": 1,
        "nodes": [],
        "edges": []
    }
    "#;

        let json_data: AppState = serde_json::from_str(json_str).unwrap();
        assert_eq!(json_data.nodes.len(), 0);
    }

    #[test]
    fn should_deserialize_non_empty_state() {
        let json_str = r#"
        {
            "threads": 1,
            "duration": 3, 
            "nodes": [
                {
                    "name": "lhs",
                    "kind": "std.value",
                    "props": 12
                },
                {
                    "name": "rhs",
                    "kind": "std.value",
                    "props": 30
                },
                {
                    "name": "add",
                    "kind": "std.binops.add",
                    "props": {"none": "Undefined"}
                },
                {
                    "name": "debug",
                    "kind": "std.debug",
                    "props": {"none": "Undefined"}
                }
            ],
            "edges": [
                {
                    "source": {"node": "lhs", "index": 0},
                    "dest": {"node": "add", "index": 0}
                },
                {
                    "source": {"node": "rhs", "index": 0},
                    "dest": {"node": "add", "index": 1}
                },
                {
                    "source": {"node": "add", "index": 0},
                    "dest": {"node": "debug", "index": 0}
                }
            ]
        }
        "#;
        let app_state: AppState = serde_json::from_str(json_str).unwrap();
        assert_eq!(app_state.nodes.len(), 4);
        app_state.run();
    }

    #[test]
    #[should_panic = r#"Addition of JSON values of type String("string") and Number(30) is not supported."#]
    fn should_fail_on_invalid_types() {
        let json_str = r#"
        {
            "threads": 3,
            "duration": 3, 
            "nodes": [
                {
                    "name": "lhs",
                    "kind": "std.value",
                    "props": "string"
                },
                {
                    "name": "rhs",
                    "kind": "std.value",
                    "props": 30
                },
                {
                    "name": "add",
                    "kind": "std.binops.add",
                    "props": {"none": "Undefined"}
                },
                {
                    "name": "debug",
                    "kind": "std.debug",
                    "props": {"none": "Undefined"}
                }
            ],
            "edges": [
                {
                    "source": {"node": "lhs", "index": 0},
                    "dest": {"node": "add", "index": 0}
                },
                {
                    "source": {"node": "rhs", "index": 0},
                    "dest": {"node": "add", "index": 1}
                },
                {
                    "source": {"node": "add", "index": 0},
                    "dest": {"node": "debug", "index": 0}
                }
            ]
        }
        "#;

        let app_state: AppState = serde_json::from_str(json_str).unwrap();
        assert_eq!(app_state.nodes.len(), 4);
        app_state.run();
    }

    #[test]
    fn should_load_image() {
        let json_str = r#"
        {
            "threads": 1,
            "duration": 10, 
            "nodes": [
                {
                    "name": "bits",
                    "kind": "std.valueupdate",
                    "props": 255
                },
                {
                    "name": "transformer",
                    "kind": "std.transform.vec",
                    "props": 10
                },
                {
                    "name": "image",
                    "kind": "img.decode",
                    "props": {"none": "Undefined"}
                },
                {
                    "name": "debug",
                    "kind": "std.debug",
                    "props": {"none": "Undefined"}
                }
            ],
            "edges": [
                {
                    "source": {"node": "bits", "index": 0},
                    "dest": {"node": "transformer", "index": 0}
                },
                {
                    "source": {"node": "transformer", "index": 0},
                    "dest": {"node": "debug", "index": 0}
                }
            ]
        }
        "#;
        let app_state: AppState = serde_json::from_str(json_str).unwrap();
        assert_eq!(app_state.nodes.len(), 4);
        let err = app_state.run();
        assert!(err.is_err());
        assert!(err.is_err_and(|e| e.to_string() == r#"Errors occured while updating nodes: [NodeUpdateError { source: ConnectError { message: "Send Error: Failed to send item to output" }, node_id: Some(3), node_desc: None }]"#));
    }
}
