#!/usr/bin/env python3
"""Test hover via LSP protocol."""
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

# Open document
test_content = """apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/test-pipeline.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": test_content
    }
})
time.sleep(0.3)
print("✓ Document opened")

# Request hover on "tasks" field (line 5, character 4)
send_lsp_message(proc, "textDocument/hover", {
    "textDocument": {"uri": "file:///tmp/test-pipeline.yaml"},
    "position": {"line": 5, "character": 4}
}, msg_id=2)
time.sleep(0.5)
print("✓ Hover requested for 'tasks' field")

# Request hover on "Pipeline" kind (line 1, character 7)
send_lsp_message(proc, "textDocument/hover", {
    "textDocument": {"uri": "file:///tmp/test-pipeline.yaml"},
    "position": {"line": 1, "character": 7}
}, msg_id=3)
time.sleep(0.5)
print("✓ Hover requested for 'Pipeline' kind")

# Shutdown
send_lsp_message(proc, "shutdown", {}, msg_id=4)
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

# Find hover responses
hover_tasks = None
hover_pipeline = None
for msg in messages:
    if msg.get("id") == 2:
        hover_tasks = msg
    elif msg.get("id") == 3:
        hover_pipeline = msg

success = True

# Check hover on 'tasks' field
if hover_tasks and "result" in hover_tasks and hover_tasks["result"]:
    result = hover_tasks["result"]
    content = result.get("contents", {})
    if isinstance(content, dict):
        value = content.get("value", "")
    else:
        value = str(content)

    print(f"\n✅ Hover on 'tasks' field:")
    print(f"   Content preview: {value[:100]}...")

    if "tasks" in value.lower() or "pipelinetask" in value.lower():
        print(f"   ✓ Contains relevant documentation")
    else:
        print(f"   ⚠ Documentation may be incomplete")
        success = False
else:
    print(f"\n❌ No hover result for 'tasks' field")
    success = False

# Check hover on 'Pipeline' kind
if hover_pipeline and "result" in hover_pipeline and hover_pipeline["result"]:
    result = hover_pipeline["result"]
    content = result.get("contents", {})
    if isinstance(content, dict):
        value = content.get("value", "")
    else:
        value = str(content)

    print(f"\n✅ Hover on 'Pipeline' kind:")
    print(f"   Content preview: {value[:100]}...")

    if "pipeline" in value.lower():
        print(f"   ✓ Contains Pipeline documentation")
    else:
        print(f"   ⚠ Documentation may be incomplete")
        success = False
else:
    print(f"\n❌ No hover result for 'Pipeline' kind")
    success = False

if success:
    print(f"\n✅ All hover tests passed!")
    sys.exit(0)
else:
    print(f"\n❌ Some hover tests failed")
    sys.exit(1)
