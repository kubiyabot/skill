# WIT Interface

The Skill Engine Component Model interface is defined in `wit/skill.wit`.

## Interface Definition

```wit
package kubiya:skill;

interface types {
    // A tool parameter definition
    record parameter {
        name: string,
        description: string,
        param-type: string, // string, number, boolean, etc.
        required: bool,
    }

    // A tool definition
    record tool {
        name: string,
        description: string,
        parameters: list<parameter>,
    }

    // Skill metadata
    record skill-metadata {
        name: string,
        version: string,
        description: string,
        author: option<string>,
    }

    // Result of a tool execution
    record execution-result {
        success: bool,
        output: string,
        error-message: option<string>,
    }
}

world skill {
    use types.{tool, skill-metadata, execution-result};

    // Exports required by every skill
    export get-metadata: func() -> skill-metadata;
    export get-tools: func() -> list<tool>;
    export execute-tool: func(name: string, args-json: string) -> execution-result;

    // Optional: Validate configuration
    export validate-config: func() -> result<_, string>;
}
```

## Implementation

To implement this interface in Rust:

```rust
use wit_bindgen::generate;

generate!({
    world: "skill",
    path: "wit/skill.wit",
});

struct MySkill;

impl Guest for MySkill {
    // Implement methods...
}

export!(MySkill);
```
