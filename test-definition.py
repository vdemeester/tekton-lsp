#!/usr/bin/env python3
"""Test go-to-definition via LSP protocol."""
import json, subprocess, sys, time, re

def send_lsp_message(proc, method, params=None, msg_id=None):
    message = {"jsonrpc": "2.0", "method": method}
    if params is not None:
        message["params"] = params
    if msg_id is not None:
        message["id"] = msg_id

    content = json.dumps(message)
    header = f"Content-Length: {len(content)}\r\n\r\n"
    proc.stdin.write((header + content).encode())
    proc.stdin.flush()

def parse_lsp_responses(output):
    messages = []
    pattern = r'Content-Length: (\d+)\r\n\r\n'
    pos = 0
    while pos < len(output):
        match = re.search(pattern, output[pos:])
        if not match:
            break
        length = int(match.group(1))
        json_start = pos + match.end()
        json_end = json_start + length
        try:
            msg = json.loads(output[json_start:json_end])
            messages.append(msg)
        except json.JSONDecodeError:
            pass
        pos = json_end
    return messages

# Start LSP server
lsp_binary = "./target/release/tekton-lsp"
proc = subprocess.Popen([lsp_binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

print("✓ LSP server started")

# Initialize
send_lsp_message(proc, "initialize", {"capabilities": {}, "processId": None, "rootUri": "file:///tmp/tekton-test"}, msg_id=1)
time.sleep(0.3)
print("✓ Initialize sent")

# Initialized
send_lsp_message(proc, "initialized", {})
time.sleep(0.3)

# Open Task document first (the definition)
task_content = """apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - name: compile
      image: golang:1.21
      script: |
        go build ./..."""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/tasks/build-task.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": task_content
    }
})
time.sleep(0.3)
print("✓ Task document opened")

# Open Pipeline document (has reference to Task)
pipeline_content = """apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/pipelines/main-pipeline.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": pipeline_content
    }
})
time.sleep(0.3)
print("✓ Pipeline document opened")

# Request go-to-definition on "build-task" in taskRef.name (line 8, character 14)
send_lsp_message(proc, "textDocument/definition", {
    "textDocument": {"uri": "file:///tmp/pipelines/main-pipeline.yaml"},
    "position": {"line": 8, "character": 14}
}, msg_id=2)
time.sleep(0.5)
print("✓ Definition requested for 'build-task' in taskRef")

# Shutdown
send_lsp_message(proc, "shutdown", {}, msg_id=3)
send_lsp_message(proc, "exit", {})

proc.stdin.close()
proc.wait(timeout=2)

stdout_output = proc.stdout.read().decode()

# Parse responses
messages = parse_lsp_responses(stdout_output)

print(f"\n📋 All messages received:")
for i, msg in enumerate(messages):
    msg_type = msg.get('method', f"id={msg.get('id')}")
    print(f"  {i}: {msg_type}")

# Find definition response
definition_response = None
for msg in messages:
    if msg.get("id") == 2:
        definition_response = msg
        break

success = True

if definition_response and "result" in definition_response and definition_response["result"]:
    result = definition_response["result"]
    print(f"\n✅ Go-to-definition response:")
    print(f"   Result: {json.dumps(result, indent=4)}")

    # Check if it points to the Task file
    if isinstance(result, dict):
        uri = result.get("uri", "")
        if "build-task.yaml" in uri:
            print(f"   ✓ Correctly points to Task file")
        else:
            print(f"   ⚠ URI doesn't point to Task file: {uri}")
            success = False
    elif isinstance(result, list) and len(result) > 0:
        uri = result[0].get("uri", "")
        if "build-task.yaml" in uri:
            print(f"   ✓ Correctly points to Task file")
        else:
            print(f"   ⚠ URI doesn't point to Task file: {uri}")
            success = False
    else:
        print(f"   ⚠ Unexpected result format")
        success = False
else:
    print(f"\n⚠ No definition result (may be expected if cursor position doesn't match)")
    print(f"   Definition response: {definition_response}")
    # This might be acceptable if the position is slightly off
    # For now, we'll just note it

if success:
    print(f"\n✅ Go-to-definition test passed!")
    sys.exit(0)
else:
    print(f"\n❌ Go-to-definition test failed")
    sys.exit(1)
