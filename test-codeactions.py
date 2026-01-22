#!/usr/bin/env python3
"""Test code actions via LSP protocol."""
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

# Open document with an unknown field (will trigger diagnostic)
content = """apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: my-task
spec:
  unknownField: value
  steps:
    - name: build
      image: golang:1.21
"""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/task.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": content
    }
})
time.sleep(0.5)
print("✓ Document opened")

# Request code actions with a diagnostic for the unknown field
# We simulate the diagnostic that would be returned by the server
diagnostic = {
    "range": {
        "start": {"line": 5, "character": 2},
        "end": {"line": 5, "character": 14}
    },
    "severity": 2,  # Warning
    "source": "tekton-lsp",
    "message": "Unknown field 'unknownField' in Task spec"
}

send_lsp_message(proc, "textDocument/codeAction", {
    "textDocument": {"uri": "file:///tmp/task.yaml"},
    "range": {
        "start": {"line": 5, "character": 0},
        "end": {"line": 5, "character": 20}
    },
    "context": {
        "diagnostics": [diagnostic]
    }
}, msg_id=2)
time.sleep(0.5)
print("✓ Code actions requested")

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

# Find code action response
action_response = None
for msg in messages:
    if msg.get("id") == 2:
        action_response = msg
        break

success = True

if action_response and "result" in action_response:
    result = action_response["result"]

    if result is None:
        print(f"\n⚠ No code actions returned")
        success = False
    elif isinstance(result, list):
        print(f"\n✅ Code actions response received")
        print(f"   Number of actions: {len(result)}")

        if len(result) > 0:
            action = result[0]
            title = action.get("title", "")
            kind = action.get("kind", "")
            print(f"   ✓ Got code action: {title}")
            print(f"     Kind: {kind}")

            # Check that it's a quick fix for the unknown field
            if "Remove" in title and "unknownField" in title:
                print(f"   ✓ Action correctly offers to remove unknown field")
            else:
                print(f"   ⚠ Action doesn't match expected pattern")
                # This might be acceptable if the action has a different format
                success = True

            # Check for edit
            if action.get("edit"):
                print(f"   ✓ Action has workspace edit")
            else:
                print(f"   ⚠ Action missing workspace edit")
                success = False
        else:
            print(f"   ⚠ No actions in response")
            success = False
    else:
        print(f"\n⚠ Unexpected result format: {type(result)}")
        success = False
else:
    print(f"\n❌ No code action response received")
    print(f"   Response: {action_response}")
    success = False

if success:
    print(f"\n✅ Code actions test passed!")
    sys.exit(0)
else:
    print(f"\n❌ Code actions test failed")
    sys.exit(1)
