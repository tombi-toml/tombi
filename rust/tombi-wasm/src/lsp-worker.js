import init, {
  ServerConfig,
  remove_workspace_file,
  serve,
  set_workspace_file,
} from "./tombi_lsp_wasm.js";

class AsyncByteQueue {
  #closed = false;
  #items = [];
  #waiters = [];

  push(item) {
    const waiter = this.#waiters.shift();
    if (waiter) waiter({ done: false, value: item });
    else this.#items.push(item);
  }

  close() {
    this.#closed = true;
    for (const waiter of this.#waiters.splice(0)) {
      waiter({ done: true, value: undefined });
    }
  }

  next() {
    const item = this.#items.shift();
    if (item) return Promise.resolve({ done: false, value: item });
    if (this.#closed) return Promise.resolve({ done: true, value: undefined });
    return new Promise((resolve) => this.#waiters.push(resolve));
  }

  [Symbol.asyncIterator]() {
    return this;
  }
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const input = new AsyncByteQueue();
let outputBuffer = new Uint8Array();
let ready = false;
const pendingMessages = [];

function encode(message) {
  const body = JSON.stringify(message);
  const contentLength = encoder.encode(body).byteLength;
  return encoder.encode(`Content-Length: ${contentLength}\r\n\r\n${body}`);
}

function emitMessages(chunk) {
  const next = new Uint8Array(outputBuffer.byteLength + chunk.byteLength);
  next.set(outputBuffer);
  next.set(chunk, outputBuffer.byteLength);
  outputBuffer = next;

  while (true) {
    const separator = decoder.decode(outputBuffer).indexOf("\r\n\r\n");
    if (separator < 0) return;

    const header = decoder.decode(outputBuffer.slice(0, separator));
    const length = Number.parseInt(header.match(/Content-Length:\s*(\d+)/i)?.[1] ?? "", 10);
    const bodyStart = separator + 4;
    if (!Number.isFinite(length) || outputBuffer.byteLength < bodyStart + length) return;

    const body = outputBuffer.slice(bodyStart, bodyStart + length);
    self.postMessage(JSON.parse(decoder.decode(body)));
    outputBuffer = outputBuffer.slice(bodyStart + length);
  }
}

function handleMessage(message) {
  if (message.method === "textDocument/didOpen") {
    const document = message.params?.textDocument;
    if (document?.uri?.startsWith("file:") && typeof document.text === "string") {
      set_workspace_file(document.uri, document.text);
    }
  } else if (message.method === "textDocument/didChange") {
    const uri = message.params?.textDocument?.uri;
    const change = message.params?.contentChanges?.findLast(
      (candidate) => candidate.range === undefined && typeof candidate.text === "string",
    );
    if (uri?.startsWith("file:") && change) {
      set_workspace_file(uri, change.text);
    }
  } else if (message.method === "workspace/didChangeWatchedFiles") {
    for (const change of message.params?.changes ?? []) {
      if (change.type === 3 && change.uri?.startsWith("file:")) {
        remove_workspace_file(change.uri);
      }
    }
  }
  input.push(encode(message));
}

self.addEventListener("message", (event) => {
  if (ready) handleMessage(event.data);
  else pendingMessages.push(event.data);
});

try {
  await init();
  const output = new WritableStream({ write: emitMessages });
  void serve(new ServerConfig(input, output)).catch((error) => {
    self.postMessage({
      type: "error",
      message: String(error),
    });
  });

  ready = true;
  for (const message of pendingMessages.splice(0)) handleMessage(message);
  self.postMessage({ type: "ready" });
} catch (error) {
  self.postMessage({
    type: "error",
    message: String(error),
  });
}
