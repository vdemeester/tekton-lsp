//! Documentation content for Tekton resources and fields.
//!
//! Provides Markdown documentation for hover tooltips.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Documentation lookup table for Tekton resources and fields.
static TEKTON_DOCS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut docs = HashMap::new();

    // Resource kinds
    docs.insert(
        "Pipeline",
        r#"# Pipeline

A Pipeline is a collection of Tasks that you define and arrange in a specific order of execution as part of your continuous integration flow.

Each Task in a Pipeline executes as a Pod on your Kubernetes cluster. You can configure various execution conditions to fit your business needs.

[Tekton Pipelines Documentation](https://tekton.dev/docs/pipelines/pipelines/)"#,
    );

    docs.insert(
        "Task",
        r#"# Task

A Task is a collection of Steps that you define and arrange in a specific order of execution as part of your continuous integration flow.

A Task executes as a Pod on your Kubernetes cluster. Each Step within a Task executes in its own container within the same Pod.

[Tekton Tasks Documentation](https://tekton.dev/docs/pipelines/tasks/)"#,
    );

    docs.insert(
        "PipelineRun",
        r#"# PipelineRun

A PipelineRun instantiates and executes a Pipeline on your cluster.

The PipelineRun references the Pipeline you want to execute and provides the necessary parameters, workspaces, and context.

[Tekton PipelineRuns Documentation](https://tekton.dev/docs/pipelines/pipelineruns/)"#,
    );

    docs.insert(
        "TaskRun",
        r#"# TaskRun

A TaskRun instantiates and executes a Task on your cluster.

The TaskRun references the Task you want to execute and provides the necessary parameters, workspaces, and context.

[Tekton TaskRuns Documentation](https://tekton.dev/docs/pipelines/taskruns/)"#,
    );

    // Common fields
    docs.insert(
        "tasks",
        r#"# tasks

Specifies the Tasks that comprise the Pipeline and the details of their execution.

Each PipelineTask must have:
- **name**: unique name for the task in the pipeline
- **taskRef** or **taskSpec**: reference to an existing Task or inline Task definition

Optional fields:
- **runAfter**: specify tasks that must complete before this task
- **params**: parameters to pass to the task
- **workspaces**: workspace bindings
- **when**: conditional execution expressions"#,
    );

    docs.insert(
        "steps",
        r#"# steps

Specifies one or more container images to run in the Task.

Each Step runs sequentially in the order specified. If a Step fails, subsequent Steps are not executed.

Required fields:
- **image**: container image to run

Common fields:
- **name**: step name
- **script**: script to execute in the container
- **command**: command and arguments to run
- **env**: environment variables
- **workingDir**: working directory"#,
    );

    docs.insert(
        "params",
        r#"# params

Specifies the execution parameters for the Pipeline/Task.

Parameters can be:
- **string**: simple string value
- **array**: list of string values
- **object**: structured data with properties

Each parameter can have:
- **name**: parameter name (required)
- **type**: string, array, or object
- **description**: human-readable description
- **default**: default value if not provided"#,
    );

    docs.insert(
        "workspaces",
        r#"# workspaces

Specifies paths to volumes required by the Pipeline/Task to execute.

Workspaces allow Tasks to share data and can be backed by:
- PersistentVolumeClaim
- emptyDir
- ConfigMap
- Secret

Each workspace has:
- **name**: workspace name (required)
- **description**: human-readable description
- **optional**: whether the workspace is optional
- **readOnly**: whether the workspace is read-only
- **mountPath**: path where the workspace is mounted"#,
    );

    docs.insert(
        "taskRef",
        r#"# taskRef

Reference to a Task that exists in the cluster.

Can reference a Task by:
- **name**: name of the Task in the same namespace
- **kind**: Task or ClusterTask (optional)
- **apiVersion**: API version (optional)

Example:
```yaml
taskRef:
  name: build-task
```"#,
    );

    docs.insert(
        "taskSpec",
        r#"# taskSpec

Inline Task specification embedded in the Pipeline.

Allows defining a Task directly without creating a separate Task resource.

Contains the same fields as a Task spec:
- **steps**: task steps (required)
- **params**: task parameters
- **workspaces**: task workspaces
- **results**: task results"#,
    );

    docs.insert(
        "results",
        r#"# results

Specifies the results that the Task/Pipeline will emit.

Results can be used to pass data between Tasks or output data from a Pipeline.

Each result has:
- **name**: result name (required)
- **description**: human-readable description
- **type**: string (default) or array

Tasks emit results by writing to:
```
$(results.<name>.path)
```"#,
    );

    docs.insert(
        "finally",
        r#"# finally

Specifies Tasks that run after all other Pipeline tasks complete.

Finally tasks run regardless of whether the Pipeline succeeded or failed. They are commonly used for:
- Cleanup operations
- Sending notifications
- Recording metrics

Finally tasks cannot have dependencies on regular Pipeline tasks and cannot use `runAfter`."#,
    );

    docs.insert(
        "runAfter",
        r#"# runAfter

Specifies the list of PipelineTask names that must complete before this task runs.

Example:
```yaml
- name: deploy
  taskRef:
    name: deploy-task
  runAfter:
    - build
    - test
```

This ensures proper ordering of task execution in the Pipeline."#,
    );

    // Metadata fields
    docs.insert(
        "metadata",
        r#"# metadata

Standard Kubernetes object metadata.

Required fields:
- **name**: resource name (must be unique in namespace)

Common fields:
- **namespace**: namespace (defaults to "default")
- **labels**: key-value labels for organizing resources
- **annotations**: key-value annotations for storing metadata"#,
    );

    docs.insert(
        "name",
        r#"# name

The name of the resource.

Must be unique within its namespace and follow Kubernetes naming conventions:
- Contain only lowercase alphanumeric characters, '-' or '.'
- Start with an alphanumeric character
- Be at most 253 characters

Example: `my-pipeline`, `build-task-v1`"#,
    );

    docs.insert(
        "namespace",
        r#"# namespace

The Kubernetes namespace where this resource exists.

If not specified, defaults to the `default` namespace or the namespace in the current context.

Example: `tekton-pipelines`, `my-project`"#,
    );

    docs.insert(
        "labels",
        r#"# labels

Key-value pairs used to organize and select resources.

Labels are used for:
- Organizing resources into groups
- Selecting resources with label selectors
- Filtering in CLI and UI

Example:
```yaml
labels:
  app: my-app
  environment: production
```"#,
    );

    docs.insert(
        "annotations",
        r#"# annotations

Key-value pairs for storing non-identifying metadata.

Annotations are used for:
- Build information
- Tool configuration
- Deployment details

Example:
```yaml
annotations:
  description: "Main build pipeline"
  author: "team@example.com"
```"#,
    );

    docs.insert(
        "spec",
        r#"# spec

Specification of the desired behavior of the resource.

Contains resource-specific fields that define how the Pipeline/Task/Run should execute."#,
    );

    // Step fields
    docs.insert(
        "image",
        r#"# image

The container image to use for this Step.

Must be a valid container image reference:
- `ubuntu:latest`
- `gcr.io/project/image:tag`
- `registry.example.com/image@sha256:...`

Example:
```yaml
- name: build
  image: golang:1.21
  script: |
    go build ./...
```"#,
    );

    docs.insert(
        "script",
        r#"# script

A script to execute in the container.

The script runs as the container's entrypoint. Use a shebang to specify the interpreter:

```yaml
script: |
  #!/usr/bin/env bash
  set -e
  echo "Building..."
  make build
```

If no shebang is provided, the default shell is used."#,
    );

    docs.insert(
        "command",
        r#"# command

The container entrypoint command.

Overrides the container image's ENTRYPOINT:

```yaml
command:
  - /bin/bash
  - -c
  - echo "Hello"
```

Use `args` to provide additional arguments."#,
    );

    docs.insert(
        "args",
        r#"# args

Arguments to the container entrypoint.

Overrides the container image's CMD:

```yaml
command:
  - python
args:
  - script.py
  - --verbose
```"#,
    );

    docs.insert(
        "env",
        r#"# env

Environment variables for the container.

Example:
```yaml
env:
  - name: MY_VAR
    value: "my-value"
  - name: SECRET_VAR
    valueFrom:
      secretKeyRef:
        name: my-secret
        key: password
```"#,
    );

    docs.insert(
        "workingDir",
        r#"# workingDir

The working directory for the container.

Defaults to the container image's WORKDIR. Can reference workspace paths:

```yaml
workingDir: $(workspaces.source.path)
```"#,
    );

    docs
});

/// Get documentation for a given key (field name or resource kind).
pub fn get_documentation(key: &str) -> Option<&'static str> {
    TEKTON_DOCS.get(key).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_documentation_for_pipeline() {
        let doc = get_documentation("Pipeline");
        assert!(doc.is_some());
        assert!(doc.unwrap().contains("Pipeline"));
    }

    #[test]
    fn test_get_documentation_for_tasks_field() {
        let doc = get_documentation("tasks");
        assert!(doc.is_some());
        assert!(doc.unwrap().contains("PipelineTask"));
    }

    #[test]
    fn test_get_documentation_unknown_key() {
        let doc = get_documentation("unknown_field_xyz");
        assert!(doc.is_none());
    }
}
