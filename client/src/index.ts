import { client as WebSocketClient } from "websocket";

const client = new WebSocketClient();

client.on("connectFailed", error => {
    console.log("Connection failed", error);
});

client.on("connect", connection => {
    console.log("Connected");

    connection.on("message", message => {
        console.log("Received message", message);
    });

    connection.on("error", error => {
        console.log("Received error", error);
    });

    connection.on("close", (code, description) => {
        console.log(`Connection closed. Code: ${code}. Description: ${description}`);
    });

    const sayHello = () => {
        if (connection.connected) {
            connection.sendUTF("Hello from client!");
        }
    };
    sayHello();
});

client.connect("ws://localhost:8080/", "magic-protocol");
