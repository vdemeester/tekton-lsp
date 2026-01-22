#!/usr/bin/env python3
"""Test document symbols via LSP protocol."""
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

# Open Pipeline document
pipeline_content = """apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  params:
    - name: version
    - name: environment
  workspaces:
    - name: source
  tasks:
    - name: build
      taskRef:
        name: build-task
    - name: test
      taskRef:
        name: test-task
  finally:
    - name: cleanup
      taskRef:
        name: cleanup-task"""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/pipeline.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": pipeline_content
    }
})
time.sleep(0.3)
print("✓ Pipeline document opened")

# Request document symbols
send_lsp_message(proc, "textDocument/documentSymbol", {
    "textDocument": {"uri": "file:///tmp/pipeline.yaml"}
}, msg_id=2)
time.sleep(0.5)
print("✓ Document symbols requested")

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

# Find document symbols response
symbols_response = None
for msg in messages:
    if msg.get("id") == 2:
        symbols_response = msg
        break

success = True

if symbols_response and "result" in symbols_response and symbols_response["result"]:
    result = symbols_response["result"]
    print(f"\n✅ Document symbols response received")

    # Should have at least one root symbol (the Pipeline)
    if isinstance(result, list) and len(result) > 0:
        root = result[0]
        print(f"   Root symbol: {root.get('name', 'unnamed')}")

        # Check root symbol is Pipeline
        if "Pipeline" in root.get("name", ""):
            print(f"   ✓ Root symbol is Pipeline")
        else:
            print(f"   ⚠ Root symbol is not Pipeline: {root.get('name')}")
            success = False

        # Check for children (metadata, spec)
        children = root.get("children", [])
        if children:
            print(f"   ✓ Has {len(children)} children")
            child_names = [c.get("name", "") for c in children]
            print(f"   Children: {child_names}")

            # Find spec
            spec = None
            for child in children:
                if child.get("name") == "spec":
                    spec = child
                    break

            if spec:
                spec_children = spec.get("children", [])
                spec_child_names = [c.get("name", "") for c in spec_children]
                print(f"   Spec children: {spec_child_names}")

                # Check for expected sections
                has_params = any("params" in name for name in spec_child_names)
                has_tasks = any("tasks" in name for name in spec_child_names)

                if has_params and has_tasks:
                    print(f"   ✓ Found params and tasks sections")
                else:
                    print(f"   ⚠ Missing expected sections (params: {has_params}, tasks: {has_tasks})")
                    success = False

                # Check tasks array has correct count
                for child in spec_children:
                    if "tasks" in child.get("name", "") and "(2)" in child.get("name", ""):
                        print(f"   ✓ Tasks array shows correct count (2)")
                        task_items = child.get("children", [])
                        if task_items:
                            task_names = [t.get("name", "") for t in task_items]
                            print(f"   Task items: {task_names}")
                            if "build" in task_names and "test" in task_names:
                                print(f"   ✓ Found expected task names")
                            else:
                                print(f"   ⚠ Missing expected task names")
                                success = False
                        break
        else:
            print(f"   ⚠ No children found")
            success = False
    else:
        print(f"   ⚠ Unexpected result format: {type(result)}")
        success = False
else:
    print(f"\n⚠ No symbols result")
    print(f"   Symbols response: {symbols_response}")
    success = False

if success:
    print(f"\n✅ Document symbols test passed!")
    sys.exit(0)
else:
    print(f"\n❌ Document symbols test failed")
    sys.exit(1)
