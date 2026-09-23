// Proves event loop + child_process + net in the embedded runtime:
// 1. spawnSync a real child (cmd /c echo), 2. run a TCP server + client
// through the WSAPOLL event loop, 3. setTimeout fires via loop timers.
import { spawnSync } from "child_process";
import * as net from "net";

// 1. child process (synchronous slice)
const child = spawnSync("cmd", ["/c", "echo", "child-ok"]);
console.log("child.status=" + child.status);
console.log("child.stdout=" + child.stdout.trim());

// 2. timers — the event loop's timer wheel
setTimeout(() => {
  console.log("timer-fired");

  // 3. TCP loopback over the event loop (WSAPOLL backend on Windows)
  const server = net.createServer((socket) => {
    socket.on("data", (chunk: Buffer) => {
      console.log("server-got=" + chunk.toString().trim());
      socket.write("pong");
      socket.end();
    });
  });
  server.listen(0, () => {
    const addr = server.address() as any;
    const port = addr.port as number;
    const client = net.connect(port, "127.0.0.1", () => {
      client.write("ping");
    });
    client.on("data", (chunk: Buffer) => {
      console.log("client-got=" + chunk.toString().trim());
      client.end();
      server.close();
    });
  });
}, 10);
