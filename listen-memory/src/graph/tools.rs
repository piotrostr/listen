use serde_json::json;

pub fn update_memory_tool_graph() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "update_graph_memory",
            "description": "Update the relationship key of an existing graph memory based on new information. This function should be called when there's a need to modify an existing relationship in the knowledge graph. The update should only be performed if the new information is more recent, more accurate, or provides additional context compared to the existing information. The source and destination nodes of the relationship must remain the same as in the existing graph memory; only the relationship itself can be updated.",
            "parameters": {
                    "type": "object",
                    "properties": {
                            "source": {
                                    "type": "string",
                                    "description": "The identifier of the source node in the relationship to be updated. This should match an existing node in the graph.",
                            },
                            "destination": {
                                    "type": "string",
                                    "description": "The identifier of the destination node in the relationship to be updated. This should match an existing node in the graph.",
                            },
                            "relationship": {
                                    "type": "string",
                                    "description": "The new or updated relationship between the source and destination nodes. This should be a concise, clear description of how the two nodes are connected.",
                            },
                    },
                    "required": ["source", "destination", "relationship"],
            },
    },
    })
}

pub fn add_memory_tool_graph() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "add_graph_memory",
            "description": "Add a new graph memory to the knowledge graph. This function creates a new relationship between two nodes, potentially creating new nodes if they don't exist.",
            "parameters": {
                    "type": "object",
                    "properties": {
                            "source": {
                                    "type": "string",
                                    "description": "The identifier of the source node in the new relationship. This can be an existing node or a new node to be created.",
                            },
                            "destination": {
                                    "type": "string",
                                    "description": "The identifier of the destination node in the new relationship. This can be an existing node or a new node to be created.",
                            },
                            "relationship": {
                                    "type": "string",
                                    "description": "The type of relationship between the source and destination nodes. This should be a concise, clear description of how the two nodes are connected.",
                            },
                            "source_type": {
                                    "type": "string",
                                    "description": "The type or category of the source node. This helps in classifying and organizing nodes in the graph.",
                            },
                            "destination_type": {
                                    "type": "string",
                                    "description": "The type or category of the destination node. This helps in classifying and organizing nodes in the graph.",
                            },
                    },
                    "required": [
                            "source",
                            "destination",
                            "relationship",
                            "source_type",
                            "destination_type",
                    ],
            },
    },
    })
}

pub fn noop_tool() -> serde_json::Value {
    json!({
        "type": "function",
    "function": {
            "name": "noop",
            "description": "No operation should be performed to the graph entities. This function is called when the system determines that no changes or additions are necessary based on the current input or context. It serves as a placeholder action when no other actions are required, ensuring that the system can explicitly acknowledge situations where no modifications to the graph are needed.",
            "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": [],
            },
    },
    })
}

pub fn relations_tool() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "establish_relationships",
            "description": "Establish relationships among the entities based on the provided text.",
            "parameters": {
                    "type": "object",
                    "properties": {
                            "entities": {
                                    "type": "array",
                                    "items": {
                                            "type": "object",
                                            "properties": {
                                                    "source": {"type": "string", "description": "The source entity of the relationship."},
                                                    "relationship": {
                                                            "type": "string",
                                                            "description": "The relationship between the source and destination entities.",
                                                    },
                                                    "destination": {
                                                            "type": "string",
                                                            "description": "The destination entity of the relationship.",
                                                    },
                                            },
                                            "required": [
                                                    "source",
                                                    "relationship",
                                                    "destination",
                                            ],
                                    },
                            }
                    },
                    "required": ["entities"],
            },
    },
    })
}

pub fn extract_entities_tool() -> serde_json::Value {
    json!({
        "type": "function",
    "function": {
            "name": "extract_entities",
            "description": "Extract entities and their types from the text.",
            "parameters": {
                    "type": "object",
                    "properties": {
                            "entities": {
                                    "type": "array",
                                    "items": {
                                            "type": "object",
                                            "properties": {
                                                    "entity": {"type": "string", "description": "The name or identifier of the entity."},
                                                    "entity_type": {"type": "string", "description": "The type or category of the entity."},
                                            },
                                            "required": ["entity", "entity_type"],
                                    },
                                    "description": "An array of entities with their types.",
                            }
                    },
                    "required": ["entities"],
            },
    },
    })
}

pub fn delete_memory_tool_graph() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "delete_graph_memory",
            "description": "Delete the relationship between two nodes. This function deletes the existing relationship.",
            "parameters": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "The identifier of the source node in the relationship.",
                    },
                    "relationship": {
                        "type": "string",
                        "description": "The existing relationship between the source and destination nodes that needs to be deleted.",
                    },
                    "destination": {
                        "type": "string",
                        "description": "The identifier of the destination node in the relationship.",
                    },
                },
                "required": [
                    "source",
                    "relationship",
                    "destination",
                ],
            },
        },
    })
}
