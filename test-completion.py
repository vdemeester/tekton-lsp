#!/usr/bin/env python3
"""Test completion via LSP protocol."""
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
  namespace: default
spec:
  params: []"""

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

# Request completion at metadata section (line 3, character 2)
send_lsp_message(proc, "textDocument/completion", {
    "textDocument": {"uri": "file:///tmp/test-pipeline.yaml"},
    "position": {"line": 3, "character": 2}
}, msg_id=2)
time.sleep(0.5)
print("✓ Completion requested")

# Shutdown
send_lsp_message(proc, "shutdown", {}, msg_id=3)
send_lsp_message(proc, "exit", {})

proc.stdin.close()
proc.wait(timeout=2)

stdout_output = proc.stdout.read().decode()

# Parse responses
messages = parse_lsp_responses(stdout_output)

# Find completion response
completion_response = None
for msg in messages:
    if msg.get("id") == 2:
        completion_response = msg
        break

print(f"\n📋 All messages received:")
for i, msg in enumerate(messages):
    msg_type = msg.get('method', f"id={msg.get('id')}")
    print(f"  {i}: {msg_type}")
    if msg.get("id") == 2:
        print(f"     Full message: {json.dumps(msg, indent=6)}")

if completion_response and "result" in completion_response:
    completions = completion_response["result"]
    if isinstance(completions, list):
        labels = [c["label"] for c in completions]
        print(f"\n✅ Received {len(completions)} completions:")
        for label in labels[:10]:  # Show first 10
            print(f"   - {label}")

        # Verify expected completions
        expected = ["name", "namespace", "labels", "annotations"]
        found = [e for e in expected if e in labels]
        if len(found) == len(expected):
            print(f"\n✅ All expected metadata fields found!")
            sys.exit(0)
        else:
            print(f"\n⚠ Only found {found}, expected {expected}")
            sys.exit(1)
    else:
        print(f"\n❌ Unexpected completion format: {completions}")
        sys.exit(1)
else:
    print(f"\n❌ No completion response found")
    if completion_response:
        print(f"Completion message exists but no result: {json.dumps(completion_response, indent=2)}")
    sys.exit(1)
